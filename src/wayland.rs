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
        use wayland_backend::protocol::*;
        wayland_scanner::generate_interfaces!("wtype/protocol/virtual-keyboard-unstable-v1.xml");
    }
    
    use self::__interfaces::*;
    /// Client-side protocol implementation generated from XML  
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

impl WaylandState {
    /// Create a new WaylandState with empty/uninitialized objects.
    /// Objects will be populated during the Wayland connection process.
    pub fn new() -> Self {
        Self {
            seat: None,
            manager: None,
            keyboard: None,
            mod_state: 0,
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
        let seat = self.seat.as_ref().context("No seat available")?;
        let manager = self.manager.as_ref().context("No virtual keyboard manager available")?;
        
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
    pub fn upload_keymap(&self, keymap_data: &str) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        
        // Create a temporary file to hold the keymap data
        // Wayland requires keymaps to be sent as file descriptors
        let mut temp_file = NamedTempFile::new().context("Failed to create temporary file")?;
        std::io::Write::write_all(&mut temp_file, keymap_data.as_bytes())
            .context("Failed to write keymap data")?;
        // XKB keymaps must be null-terminated
        std::io::Write::write_all(&mut temp_file, b"\0")
            .context("Failed to write null terminator")?;
        
        // Convert to owned file descriptor for sending over Wayland
        let fd = temp_file.into_file();
        let owned_fd = OwnedFd::from(fd);
        
        // Send the keymap to the compositor
        keyboard.keymap(
            wl_keyboard::KeymapFormat::XkbV1.into(),
            owned_fd.as_fd(),
            keymap_data.len() as u32 + 1, // +1 for null terminator
        );
        
        Ok(())
    }

    /// Send a key press event to the compositor.
    ///
    /// # Arguments
    /// * `keycode` - Linux keycode (offset by 8 from XKB keycode)
    ///
    /// # Returns
    /// * `Ok(())` - Key press sent successfully  
    /// * `Err` - Virtual keyboard not available
    pub fn press_key(&self, keycode: u32) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        keyboard.key(0, keycode, wl_keyboard::KeyState::Pressed.into());
        Ok(())
    }

    /// Send a key release event to the compositor.
    ///
    /// # Arguments
    /// * `keycode` - Linux keycode (offset by 8 from XKB keycode)
    ///
    /// # Returns
    /// * `Ok(())` - Key release sent successfully
    /// * `Err` - Virtual keyboard not available
    pub fn release_key(&self, keycode: u32) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        keyboard.key(0, keycode, wl_keyboard::KeyState::Released.into());
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
    pub fn set_modifiers(&mut self, mods: u32) -> Result<()> {
        let keyboard = self.keyboard.as_ref().context("No virtual keyboard")?;
        self.mod_state = mods;
        
        // Split modifiers: depressed (regular mods) vs locked (capslock)
        // Caps lock (bit 1, value 2) is handled as a locked modifier
        let depressed = mods & !2; // Everything except capslock
        let locked = if mods & 2 != 0 { 2 } else { 0 }; // Only capslock
        
        keyboard.modifiers(depressed, 0, locked, 0);
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
        if let wl_registry::Event::Global { name, interface, version } = event {
            match interface.as_str() {
                // Bind to the first available seat - represents input devices
                "wl_seat" => {
                    let seat = registry.bind::<wl_seat::WlSeat, _, _>(
                        name,
                        std::cmp::min(version, 7), // Use supported version, max 7
                        qh,
                        (),
                    );
                    state.seat = Some(seat);
                }
                // Bind to virtual keyboard manager - allows creating virtual keyboards
                "zwp_virtual_keyboard_manager_v1" => {
                    let manager = registry.bind::<ZwpVirtualKeyboardManagerV1, _, _>(
                        name,
                        1, // This protocol only has version 1
                        qh,
                        (),
                    );
                    state.manager = Some(manager);
                }
                _ => {} // Ignore other protocols
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
        // No events need handling for virtual keyboard operation
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
        // No events defined in this protocol version
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
        // No events defined in this protocol version
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
pub fn connect_wayland() -> Result<(Connection, WaylandState)> {
    // Connect to Wayland display server (usually via WAYLAND_DISPLAY env var)
    let conn = Connection::connect_to_env().context("Failed to connect to Wayland display")?;
    let mut state = WaylandState::new();
    
    // Set up event processing
    let display = conn.display();
    let mut event_queue = conn.new_event_queue();
    let qh = event_queue.handle();
    
    // Request registry of available global objects
    let _registry = display.get_registry(&qh, ());
    // Process registry announcements to discover protocols
    event_queue.roundtrip(&mut state).context("Failed to get globals")?;
    
    // Verify required protocols are available
    if state.seat.is_none() {
        anyhow::bail!("No seat found");
    }
    if state.manager.is_none() {
        anyhow::bail!("Compositor does not support the virtual keyboard protocol");
    }
    
    // Create virtual keyboard instance
    state.create_keyboard(&qh)?;
    
    Ok((conn, state))
}