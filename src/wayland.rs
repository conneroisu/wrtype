// Wayland virtual keyboard protocol implementation for wrtype
//
// This module handles the low-level Wayland protocol communication, including:
// - Connection establishment and protocol discovery
// - Virtual keyboard creation and management
// - Keymap upload and keyboard event generation
// - Modifier state tracking and management

use anyhow::{Context, Result};
use std::os::unix::io::{AsFd, OwnedFd};
use tempfile::NamedTempFile;
use wayland_client::protocol::{wl_keyboard, wl_registry, wl_seat};
use wayland_client::{Connection, Dispatch, QueueHandle};

/// Virtual keyboard protocol bindings generated from the Wayland XML protocol definition.
///
/// This module contains the auto-generated protocol bindings for the
/// zwp_virtual_keyboard_unstable_v1 protocol extension. The protocol allows
/// clients to create virtual keyboards that can send keyboard events to the compositor.
pub mod virtual_keyboard {
    use wayland_client;
    use wayland_client::protocol::*;

    /// Protocol interface definitions generated from XML
    pub mod __interfaces {
        use wayland_client::protocol::__interfaces::*;
        wayland_scanner::generate_interfaces!("wtype/protocol/virtual-keyboard-unstable-v1.xml");
    }

    use self::__interfaces::*;
    wayland_scanner::generate_client_code!("wtype/protocol/virtual-keyboard-unstable-v1.xml");
}

use self::virtual_keyboard::{
    zwp_virtual_keyboard_manager_v1::ZwpVirtualKeyboardManagerV1,
    zwp_virtual_keyboard_v1::ZwpVirtualKeyboardV1,
};

/// Central state management for Wayland virtual keyboard functionality.
///
/// This struct maintains references to all the Wayland objects needed for
/// virtual keyboard operation and tracks the current modifier state.
pub struct WaylandState {
    /// The Wayland seat object - represents an input device collection
    seat: Option<wl_seat::WlSeat>,
    /// The virtual keyboard manager - factory for creating virtual keyboards
    manager: Option<ZwpVirtualKeyboardManagerV1>,
    /// The virtual keyboard instance - sends actual key events
    keyboard: Option<ZwpVirtualKeyboardV1>,
    /// Current modifier state bitmask (shift, ctrl, alt, etc.)
    pub mod_state: u32,
}

impl Default for WaylandState {
    fn default() -> Self {
        Self::new()
    }
}

impl WaylandState {
    /// Create a new WaylandState with empty/uninitialized objects.
    /// Objects will be populated during the Wayland connection process.
    pub fn new() -> Self {
        Self {
            // Initialize all Wayland objects as None - they'll be populated during registry discovery
            seat: None,                // Will hold the wl_seat object (input device manager)
            manager: None,             // Will hold the virtual keyboard manager factory
            keyboard: None,            // Will hold the actual virtual keyboard instance
            mod_state: 0,             // Start with no modifiers pressed (clean state)
        }
    }

    /// Create a virtual keyboard instance using the manager and seat.
    ///
    /// This must be called after both seat and manager have been discovered
    /// through the registry global announcements.
    ///
    /// # Arguments
    /// * `qh` - Queue handle for registering the new keyboard object
    ///
    /// # Returns
    /// * `Ok(())` - Virtual keyboard created successfully
    /// * `Err` - Missing seat or manager objects
    pub fn create_keyboard(&mut self, qh: &QueueHandle<Self>) -> Result<()> {
        // Verify that both required objects were discovered during registry enumeration
        let seat = self.seat.as_ref().context("No seat available")?;
        let manager = self
            .manager
            .as_ref()
            .context("No virtual keyboard manager available")?;

        // Create the virtual keyboard instance using the manager factory
        // This sends a create_virtual_keyboard request to the compositor
        // The seat parameter associates this keyboard with the input device collection
        self.keyboard = Some(manager.create_virtual_keyboard(seat, qh, ()));
        Ok(())
    }

    /// Upload an XKB keymap to the virtual keyboard.
    ///
    /// The keymap defines the mapping between keycodes and keysyms/characters.
    /// This must be called before sending any key events. The keymap is sent
    /// as a file descriptor to avoid size limitations in Wayland messages.
    ///
    /// # Arguments
    /// * `keymap_data` - Complete XKB keymap in text format
    ///
    /// # Returns  
    /// * `Ok(())` - Keymap uploaded successfully
    /// * `Err` - Virtual keyboard not available or file I/O error
    ///
    /// # Examples
    /// ```rust,no_run
    /// use wrtype::{connect_wayland, KeymapBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (connection, wayland_state) = connect_wayland()?;
    ///
    /// // Create a keymap with some characters
    /// let mut keymap_builder = KeymapBuilder::new();
    /// keymap_builder.get_keycode_for_char('a');
    /// keymap_builder.get_keycode_for_char('b');
    /// let keymap_data = keymap_builder.generate_keymap();
    ///
    /// // Upload to compositor
    /// wayland_state.upload_keymap(&keymap_data)?;
    /// connection.roundtrip()?; // Ensure keymap is processed
    ///
    /// println!("Keymap uploaded successfully");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Protocol Flow
    /// ```text
    /// Client                     Compositor
    ///   |                            |
    ///   |-- create temp file --------|
    ///   |-- write keymap data -------|
    ///   |                            |
    ///   |-- virtual_keyboard.keymap -| (send fd + size)
    ///   |                            |
    ///   |                            |-- receive fd
    ///   |                            |-- read keymap
    ///   |                            |-- parse XKB
    ///   |                            |-- activate keymap
    /// ```
    ///
    /// # Keymap Requirements
    /// The keymap must be a valid XKB keymap containing:
    /// - `xkb_keycodes` section with keycode definitions
    /// - `xkb_types` section (can use `include "complete"`)
    /// - `xkb_compatibility` section (can use `include "complete"`)
    /// - `xkb_symbols` section with key-to-symbol mappings
    pub fn upload_keymap(&self, keymap_data: &str) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;

        // STEP 1: Create a temporary file to hold the keymap data
        // Wayland protocol requires keymaps to be sent as file descriptors for efficiency
        // Large keymaps can't fit in Wayland messages, so shared memory (via FD) is used
        let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
        
        // STEP 2: Write the complete XKB keymap to the temporary file
        std::io::Write::write_all(&mut temp_file, keymap_data.as_bytes())
            .context("Failed to write keymap data")?;
        // XKB specification requires keymaps to be null-terminated C strings
        std::io::Write::write_all(&mut temp_file, b"\0")
            .context("Failed to write null terminator")?;

        // STEP 3: Convert to owned file descriptor for sending over Wayland
        // The temporary file is converted to a regular File, then to an OwnedFd
        // This allows us to send the FD while maintaining ownership semantics
        let fd = temp_file.into_file();
        let owned_fd = OwnedFd::from(fd);

        // STEP 4: Send the keymap to the compositor via the virtual keyboard protocol
        // The compositor will read the keymap from the FD and activate it for this keyboard
        keyboard.keymap(
            wl_keyboard::KeymapFormat::XkbV1.into(),  // Standard XKB format
            owned_fd.as_fd(),                         // File descriptor containing keymap
            keymap_data.len() as u32 + 1,            // Size including null terminator
        );

        Ok(())
    }

    /// Send a key press event to the compositor.
    ///
    /// The keycode must exist in the currently uploaded keymap. The key
    /// will remain pressed until a corresponding release event is sent.
    ///
    /// # Arguments
    /// * `keycode` - Linux keycode (XKB keycode + 8 offset)
    ///
    /// # Returns
    /// * `Ok(())` - Key press sent successfully  
    /// * `Err` - Virtual keyboard not available
    ///
    /// # Examples
    /// ```rust,no_run
    /// use wrtype::{connect_wayland, KeymapBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (connection, mut wayland_state) = connect_wayland()?;
    /// let mut keymap_builder = KeymapBuilder::new();
    ///
    /// // Add character to keymap and get its keycode
    /// let keycode = keymap_builder.get_keycode_for_char('a');
    /// let keymap_data = keymap_builder.generate_keymap();
    /// wayland_state.upload_keymap(&keymap_data)?;
    /// connection.roundtrip()?;
    ///
    /// // Press the key (it stays pressed)
    /// wayland_state.press_key(keycode)?;
    /// connection.roundtrip()?;
    ///
    /// // Key is now pressed - release it later
    /// std::thread::sleep(std::time::Duration::from_millis(100));
    /// wayland_state.release_key(keycode)?;
    /// connection.roundtrip()?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Protocol Message
    /// ```text
    /// virtual_keyboard.key(
    ///     serial: 0,           // Event serial (0 for virtual events)
    ///     keycode: u32,        // Linux keycode (XKB + 8)
    ///     state: Pressed       // Key state
    /// )
    /// ```
    pub fn press_key(&self, keycode: u32) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        
        // Send a key press event to the compositor
        // Parameters: serial (0 for virtual events), keycode (Linux format), state (pressed)
        // The keycode must exist in the currently active keymap or it will be ignored
        keyboard.key(
            0,                                    // Serial number (0 for virtual keyboard events)
            keycode,                             // Linux keycode (XKB keycode + 8 offset)
            wl_keyboard::KeyState::Pressed.into() // Key state: pressed
        );
        Ok(())
    }

    /// Send a key release event to the compositor.
    ///
    /// Should be paired with a corresponding press event. Releasing a key
    /// that wasn't pressed has no effect but is not an error.
    ///
    /// # Arguments
    /// * `keycode` - Linux keycode (XKB keycode + 8 offset)
    ///
    /// # Returns
    /// * `Ok(())` - Key release sent successfully
    /// * `Err` - Virtual keyboard not available
    ///
    /// # Examples
    /// ```rust,no_run
    /// use wrtype::{connect_wayland, KeymapBuilder};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (connection, mut wayland_state) = connect_wayland()?;
    /// let mut keymap_builder = KeymapBuilder::new();
    ///
    /// // Setup keymap
    /// let space_keycode = keymap_builder.get_keycode_for_key_name("space")?;
    /// let keymap_data = keymap_builder.generate_keymap();
    /// wayland_state.upload_keymap(&keymap_data)?;
    /// connection.roundtrip()?;
    ///
    /// // Complete key press sequence
    /// wayland_state.press_key(space_keycode)?;
    /// connection.roundtrip()?;
    ///
    /// std::thread::sleep(std::time::Duration::from_millis(50));
    ///
    /// wayland_state.release_key(space_keycode)?;
    /// connection.roundtrip()?;
    ///
    /// println!("Space key pressed and released");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Protocol Message
    /// ```text
    /// virtual_keyboard.key(
    ///     serial: 0,           // Event serial (0 for virtual events)
    ///     keycode: u32,        // Linux keycode (XKB + 8)
    ///     state: Released      // Key state
    /// )
    /// ```
    pub fn release_key(&self, keycode: u32) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        
        // Send a key release event to the compositor
        // This should typically be paired with a corresponding press event
        // Safe to release keys that weren't pressed (becomes a no-op)
        keyboard.key(
            0,                                      // Serial number (0 for virtual keyboard events)
            keycode,                               // Linux keycode (must match the press event)
            wl_keyboard::KeyState::Released.into() // Key state: released
        );
        Ok(())
    }

    /// Update the modifier state and send to compositor.
    ///
    /// Modifiers are split into different categories:
    /// - Depressed: Currently held modifiers (shift, ctrl, alt, etc.)  
    /// - Locked: Toggle modifiers (caps lock, num lock)
    /// - Latched: One-shot modifiers (not used here)
    /// - Group: Layout group (not used here)
    ///
    /// # Arguments
    /// * `mods` - New modifier state bitmask
    ///
    /// # Returns
    /// * `Ok(())` - Modifier state updated successfully
    /// * `Err` - Virtual keyboard not available
    ///
    /// # Examples
    /// ```rust,no_run
    /// use wrtype::{connect_wayland, Modifier};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let (connection, mut wayland_state) = connect_wayland()?;
    ///
    /// // Press Ctrl modifier
    /// let ctrl_bit = Modifier::Ctrl as u32;
    /// wayland_state.set_modifiers(ctrl_bit)?;
    /// connection.roundtrip()?;
    ///
    /// // Add Shift to the existing modifiers
    /// let shift_bit = Modifier::Shift as u32;
    /// let ctrl_shift = ctrl_bit | shift_bit;
    /// wayland_state.set_modifiers(ctrl_shift)?;
    /// connection.roundtrip()?;
    ///
    /// // Release all modifiers
    /// wayland_state.set_modifiers(0)?;
    /// connection.roundtrip()?;
    ///
    /// println!("Modifier states updated");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Protocol Message
    /// ```text
    /// virtual_keyboard.modifiers(
    ///     depressed: u32,      // Currently held modifiers (ctrl, shift, alt)
    ///     latched: u32,        // One-shot modifiers (always 0)
    ///     locked: u32,         // Toggle modifiers (caps lock)
    ///     group: u32           // Layout group (always 0)
    /// )
    /// ```
    ///
    /// # Modifier Combinations
    /// ```rust
    /// use wrtype::Modifier;
    ///
    /// // Individual modifiers
    /// let ctrl = Modifier::Ctrl as u32;       // 4
    /// let shift = Modifier::Shift as u32;     // 1
    /// let alt = Modifier::Alt as u32;         // 8
    ///
    /// // Combinations using bitwise OR
    /// let ctrl_c = ctrl;                      // Just Ctrl
    /// let ctrl_shift = ctrl | shift;          // Ctrl+Shift
    /// let ctrl_alt_del = ctrl | alt;          // Ctrl+Alt (+ Delete key)
    /// ```
    pub fn set_modifiers(&mut self, mods: u32) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        
        // Update our local modifier state tracking
        self.mod_state = mods;

        // MODIFIER CLASSIFICATION: Split modifiers into different categories per XKB protocol
        // XKB distinguishes between different types of modifiers for proper handling:
        
        // Depressed modifiers: Currently held down (shift, ctrl, alt, etc.)
        // Caps lock (bit 1, value 2) is special - it's a toggle, not a hold modifier
        let depressed = mods & !2; // Everything except capslock (use bitwise AND NOT)
        
        // Locked modifiers: Toggle state modifiers (caps lock, num lock, scroll lock)
        // Only caps lock is supported in our current implementation
        let locked = if mods & 2 != 0 { 2 } else { 0 }; // Extract only the caps lock bit

        // Send the modifier state to the compositor
        // Parameters: depressed, latched, locked, group
        // - depressed: modifiers currently held down
        // - latched: one-shot modifiers (not used in our implementation)
        // - locked: toggle modifiers like caps lock
        // - group: keyboard layout group (not used in our implementation)
        keyboard.modifiers(
            depressed, // Currently held modifiers (shift, ctrl, alt, logo, altgr)
            0,         // Latched modifiers (none in our implementation)
            locked,    // Locked modifiers (caps lock only)
            0          // Layout group (single layout in our implementation)
        );
        Ok(())
    }
}

/// Event handler for Wayland registry global announcements.
///
/// The registry announces available global objects (protocols) when we connect.
/// We look for the seat (input device collection) and virtual keyboard manager.
impl Dispatch<wl_registry::WlRegistry, ()> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        // Handle registry global announcements - the compositor tells us what protocols are available
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                // CORE PROTOCOL: Bind to the first available seat - represents input device collection
                // A seat is a group of input devices (keyboard, mouse, touch) that work together
                // Most systems have exactly one seat, but multi-user systems can have multiple
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(
                        name,                           // Global object name assigned by compositor
                        std::cmp::min(version, 7),     // Use min of our support (7) and compositor's version
                        qh,                             // Queue handle for receiving events
                        (),                             // User data (none needed)
                    );
                    state.seat = Some(seat);
                }
                // EXTENSION PROTOCOL: Bind to virtual keyboard manager - factory for virtual keyboards
                // This is the zwp_virtual_keyboard_unstable_v1 protocol extension
                // Not all compositors support this - check for None after registry discovery
                "zwp_virtual_keyboard_manager_v1" => {
                    let manager = registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(
                        name,                           // Global object name
                        1,                              // This protocol only has version 1
                        qh,                             // Queue handle
                        (),                             // User data
                    );
                    state.manager = Some(manager);
                }
                _ => {} // Ignore other protocols - we only need seat and virtual keyboard manager
            }
        }
    }
}

/// Event handler for seat events.
///
/// Seats can announce capabilities (keyboard, pointer, touch) but we don't
/// need to handle these events for virtual keyboard functionality.
impl Dispatch<wl_seat::WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _seat: &wl_seat::WlSeat,
        _event: wl_seat::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Seat events (capabilities announcements) are not needed for virtual keyboard operation
        // The seat object is used only as a parameter when creating the virtual keyboard
        // Real keyboard implementations would listen for capabilities events to know
        // when physical keyboards/mice/touch devices are added/removed
    }
}

/// Event handler for virtual keyboard manager events.
///
/// The manager doesn't send any events in the current protocol version,
/// so this implementation is empty.
impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _manager: &ZwpVirtualKeyboardManagerV1,
        _event: virtual_keyboard::zwp_virtual_keyboard_manager_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // The virtual keyboard manager protocol doesn't define any events in version 1
        // This is a factory object - it only receives requests to create virtual keyboards
        // Future protocol versions might add events for capability announcements
    }
}

/// Event handler for virtual keyboard events.
///
/// Virtual keyboards don't send events back to the client in normal operation,
/// so this implementation is empty.
impl Dispatch<ZwpVirtualKeyboardV1, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _keyboard: &ZwpVirtualKeyboardV1,
        _event: virtual_keyboard::zwp_virtual_keyboard_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Virtual keyboard protocol doesn't define any events in version 1
        // This is a send-only interface - we send key events to the compositor
        // but don't receive any events back from the virtual keyboard object
        // Real keyboards would receive events like repeat rate changes
    }
}

/// Establish connection to Wayland and set up virtual keyboard protocol.
///
/// This function performs the complete initialization sequence:
/// 1. Connect to the Wayland display server
/// 2. Create an event queue for handling protocol messages
/// 3. Discover available global objects (protocols)
/// 4. Bind to required objects (seat and virtual keyboard manager)
/// 5. Create a virtual keyboard instance
///
/// # Returns
/// * `Ok((Connection, WaylandState))` - Ready-to-use connection and state
/// * `Err(anyhow::Error)` - Various failure modes:
///   - No Wayland display available
///   - Missing required protocols
///   - Protocol negotiation failure
///
/// # Protocol Requirements
/// The compositor must support:
/// - `wl_seat` (core Wayland protocol)
/// - `zwp_virtual_keyboard_manager_v1` (virtual keyboard extension)
///
/// # Examples
/// ```rust,no_run
/// use wrtype::connect_wayland;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Connect to Wayland display (usually via WAYLAND_DISPLAY env var)
/// let (connection, mut wayland_state) = connect_wayland()?;
///
/// // Connection is ready for virtual keyboard operations
/// println!("Connected to Wayland compositor");
/// println!("Virtual keyboard protocol available");
///
/// // The connection can be used for protocol communication
/// connection.roundtrip()?; // Flush any pending messages
/// # Ok(())
/// # }
/// ```
///
/// # Protocol Flow
/// ```text
/// Client                     Compositor
///   |                            |
///   |-- wl_display.get_registry -|
///   |                            |
///   |<- wl_registry.global  ------| (announces wl_seat)
///   |<- wl_registry.global  ------| (announces zwp_virtual_keyboard_manager_v1)
///   |                            |
///   |-- wl_registry.bind  -------| (bind to wl_seat)
///   |-- wl_registry.bind  -------| (bind to manager)
///   |                            |
///   |-- manager.create_virtual_keyboard |
///   |                            |
///   |<- virtual_keyboard  -------| (ready for events)
/// ```
///
/// # Error Handling
/// ```rust,no_run
/// use wrtype::connect_wayland;
///
/// match connect_wayland() {
///     Ok((conn, state)) => {
///         println!("Successfully connected to Wayland");
///         // Use connection...
///     }
///     Err(e) => {
///         eprintln!("Failed to connect: {}", e);
///         // Common causes:
///         // - No WAYLAND_DISPLAY environment variable
///         // - Compositor doesn't support virtual keyboard protocol
///         // - Permission denied to Wayland socket
///     }
/// }
/// ```
pub fn connect_wayland() -> Result<(Connection, WaylandState)> {
    // PHASE 1: Connect to Wayland display server
    // This uses the WAYLAND_DISPLAY environment variable (usually "wayland-0")
    // If WAYLAND_DISPLAY is not set, it defaults to "wayland-0"
    let conn = Connection::connect_to_env().context("Failed to connect to Wayland display")?;
    let mut state = WaylandState::new();

    // PHASE 2: Set up event processing infrastructure
    // The display object represents the connection to the compositor
    let display = conn.display();
    // Create an event queue for handling protocol events
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();

    // PHASE 3: Request registry of available global objects
    // The registry announces what protocols and interfaces the compositor supports
    let _registry = display.get_registry(&qh, ());
    
    // PHASE 4: Process registry announcements to discover protocols
    // This roundtrip ensures we receive all global announcements before proceeding
    // The registry events will populate our seat and manager fields
    event_queue
        .roundtrip(&mut state)
        .context("Failed to get globals")?;

    // PHASE 5: Verify required protocols are available
    // We need both a seat (core protocol) and virtual keyboard manager (extension)
    if state.seat.is_none() {
        anyhow::bail!("No seat found - compositor may not support input devices");
    }
    if state.manager.is_none() {
        anyhow::bail!("Compositor does not support the virtual keyboard protocol (zwp_virtual_keyboard_unstable_v1)");
    }

    // PHASE 6: Create virtual keyboard instance
    // This uses the manager factory to create a virtual keyboard associated with the seat
    state.create_keyboard(&qh)?;

    // Return the connection and fully initialized state
    // The connection is used for roundtrips, the state holds all Wayland objects
    Ok((conn, state))
}
