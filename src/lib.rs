//! # wrtype - Rust implementation of wtype for Wayland
//!
//! This crate provides a library interface for programmatically typing text and sending
//! key events through the Wayland virtual keyboard protocol. It's designed as a Rust
//! alternative to wtype/xdotool for Wayland compositors.
//!
//! ## Features
//!
//! - Type arbitrary Unicode text through Wayland virtual keyboard
//! - Press and release individual keys and modifiers
//! - Support for complex key sequences with timing control
//! - Dynamic XKB keymap generation for Unicode characters
//! - Compatible with any Wayland compositor supporting virtual-keyboard protocol
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use wrtype::{WrtypeClient, Command};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a client and connect to Wayland
//! let mut client = WrtypeClient::new()?;
//!
//! // Type some text
//! client.type_text("Hello, Wayland!")?;
//!
//! // Execute a key sequence
//! let commands = vec![
//!     Command::KeyPress("ctrl".to_string()),
//!     Command::KeyPress("c".to_string()),
//!     Command::KeyRelease("c".to_string()),
//!     Command::KeyRelease("ctrl".to_string()),
//! ];
//! client.execute_commands(commands)?;
//! # Ok(())
//! # }
//! ```

pub mod executor;
pub mod keymap;
pub mod wayland;

pub use executor::CommandExecutor;
pub use keymap::KeymapBuilder;
pub use wayland::{connect_wayland, WaylandState};

use anyhow::Result;
use std::time::Duration;

/// Internal command representation after parsing command-line arguments.
/// Each command represents a single action to be executed in sequence.
///
/// # Examples
///
/// ```rust
/// use wrtype::Command;
/// use std::time::Duration;
///
/// // Type "Hello" with 100ms delay between characters
/// let text_cmd = Command::Text {
///     text: "Hello".to_string(),
///     delay: Duration::from_millis(100),
/// };
///
/// // Create Ctrl+C shortcut sequence
/// let ctrl_c = vec![
///     Command::ModPress(wrtype::Modifier::Ctrl),
///     Command::KeyPress("c".to_string()),
///     Command::KeyRelease("c".to_string()),
///     Command::ModRelease(wrtype::Modifier::Ctrl),
/// ];
///
/// // Press and hold Escape key
/// let hold_escape = Command::KeyPress("Escape".to_string());
///
/// // Add timing delay in sequence
/// let pause = Command::Sleep(Duration::from_millis(500));
/// ```
#[derive(Debug, Clone)]
pub enum Command {
    /// Type a string of text with specified delay between characters
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::Command;
    /// # use std::time::Duration;
    /// // Type "Hello, World!" with 50ms between each character
    /// let cmd = Command::Text {
    ///     text: "Hello, World!".to_string(),
    ///     delay: Duration::from_millis(50),
    /// };
    /// ```
    Text { text: String, delay: Duration },

    /// Press a modifier key (adds to current modifier state)
    ///
    /// Multiple modifiers can be pressed simultaneously by calling this multiple times.
    /// The modifier remains pressed until explicitly released with `ModRelease`.
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::{Command, Modifier};
    /// // Press Ctrl (remains held)
    /// let press_ctrl = Command::ModPress(Modifier::Ctrl);
    /// // Press Shift while Ctrl is still held
    /// let press_shift = Command::ModPress(Modifier::Shift);
    /// ```
    ModPress(Modifier),

    /// Release a modifier key (removes from current modifier state)
    ///
    /// Should be paired with corresponding `ModPress` commands to avoid
    /// releasing modifiers that weren't pressed by wrtype.
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::{Command, Modifier};
    /// // Complete Ctrl+Shift+T sequence
    /// let sequence = vec![
    ///     Command::ModPress(Modifier::Ctrl),
    ///     Command::ModPress(Modifier::Shift),
    ///     Command::KeyPress("t".to_string()),
    ///     Command::KeyRelease("t".to_string()),
    ///     Command::ModRelease(Modifier::Shift),
    ///     Command::ModRelease(Modifier::Ctrl),
    /// ];
    /// ```
    ModRelease(Modifier),

    /// Press a named key (key stays pressed until released)
    ///
    /// Uses XKB key names. The key remains pressed until a corresponding
    /// `KeyRelease` command is executed.
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::Command;
    /// // Press and hold the Return key
    /// let hold_return = Command::KeyPress("Return".to_string());
    /// // Press arrow keys
    /// let press_left = Command::KeyPress("Left".to_string());
    /// let press_f1 = Command::KeyPress("F1".to_string());
    /// ```
    KeyPress(String),

    /// Release a named key
    ///
    /// Should be paired with corresponding `KeyPress` commands.
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::Command;
    /// // Complete key press sequence
    /// let key_tap = vec![
    ///     Command::KeyPress("space".to_string()),
    ///     Command::KeyRelease("space".to_string()),
    /// ];
    /// ```
    KeyRelease(String),

    /// Sleep for specified duration (for timing control)
    ///
    /// Useful for creating precise timing in key sequences or waiting
    /// for applications to respond.
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::Command;
    /// # use std::time::Duration;
    /// // Wait 1 second
    /// let pause = Command::Sleep(Duration::from_secs(1));
    /// // Short pause between rapid keystrokes
    /// let micro_pause = Command::Sleep(Duration::from_millis(50));
    /// ```
    Sleep(Duration),

    /// Read and type text from stdin with specified delay
    ///
    /// Processes UTF-8 text from standard input and types each character
    /// with the specified delay between them.
    ///
    /// # Example
    /// ```rust
    /// # use wrtype::Command;
    /// # use std::time::Duration;
    /// // Read from stdin with 10ms delay per character
    /// let stdin_cmd = Command::StdinText {
    ///     delay: Duration::from_millis(10),
    /// };
    /// ```
    StdinText { delay: Duration },
}

/// Modifier keys with their corresponding bit values for Wayland protocol.
/// These values match the modifier mask constants used in XKB and Wayland.
///
/// Modifiers can be combined by pressing multiple at once. The protocol uses
/// bitwise OR to combine modifier states, allowing complex combinations like
/// Ctrl+Shift+Alt.
///
/// # Examples
///
/// ```rust
/// use wrtype::Modifier;
///
/// // Individual modifiers for common shortcuts
/// let ctrl = Modifier::Ctrl;      // Ctrl+C, Ctrl+V
/// let shift = Modifier::Shift;    // Shift+Tab, Shift+Arrow
/// let alt = Modifier::Alt;        // Alt+Tab, Alt+F4
///
/// // Special keys
/// let super_key = Modifier::Logo; // Super/Windows key
/// let altgr = Modifier::AltGr;    // AltGr for international chars
/// let caps = Modifier::CapsLock;  // Toggle caps lock state
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modifier {
    /// Shift key - bit 0 (value 1)
    ///
    /// Used for capital letters, symbols, and navigation shortcuts.
    ///
    /// # Examples
    /// - `Shift + letter` → Capital letter
    /// - `Shift + Tab` → Reverse tab navigation  
    /// - `Shift + Arrow` → Text selection
    /// - `Shift + F10` → Context menu
    Shift = 1,

    /// Caps Lock - bit 1 (value 2) - handled as locked modifier
    ///
    /// Toggle modifier that affects letter case. Unlike other modifiers,
    /// CapsLock toggles state rather than being held down.
    ///
    /// # Examples
    /// - Press once → Enable caps lock
    /// - Press again → Disable caps lock
    /// - Affects typing: `a` becomes `A` when active
    CapsLock = 2,

    /// Control key - bit 2 (value 4)
    ///
    /// Primary modifier for application shortcuts and system commands.
    ///
    /// # Examples
    /// - `Ctrl + C` → Copy
    /// - `Ctrl + V` → Paste
    /// - `Ctrl + Z` → Undo
    /// - `Ctrl + Shift + T` → Reopen closed tab
    /// - `Ctrl + Alt + Delete` → System interrupt
    Ctrl = 4,

    /// Alt key - bit 3 (value 8)
    ///
    /// Alternative modifier for menu access and application shortcuts.
    ///
    /// # Examples
    /// - `Alt + Tab` → Application switcher
    /// - `Alt + F4` → Close window
    /// - `Alt + Enter` → Properties/Fullscreen
    /// - `Alt + letter` → Menu access (underlined letters)
    Alt = 8,

    /// Logo/Super/Windows key - bit 6 (value 64)
    ///
    /// System-level modifier typically used for desktop environment shortcuts.
    /// Also known as Super key on Linux or Windows key on Windows.
    ///
    /// # Examples
    /// - `Logo + L` → Lock screen
    /// - `Logo + D` → Show desktop
    /// - `Logo + R` → Run dialog
    /// - `Logo + number` → Launch taskbar applications
    Logo = 64,

    /// AltGr (right Alt) key - bit 7 (value 128)
    ///
    /// Alternative Graphics modifier for typing international characters
    /// and symbols not available on the base keyboard layout.
    ///
    /// # Examples
    /// - `AltGr + e` → é (on many European layouts)
    /// - `AltGr + c` → ç (on many European layouts)
    /// - `AltGr + 4` → € (Euro symbol on many layouts)
    /// - `AltGr + 2` → @ (on some international layouts)
    AltGr = 128,
}

impl Modifier {
    /// Convert string modifier name to enum value.
    ///
    /// Accepts both "logo" and "win" for the Windows/Super key.
    /// Case-insensitive matching for user convenience.
    ///
    /// # Arguments
    /// * `name` - String name of the modifier (case-insensitive)
    ///
    /// # Returns
    /// * `Some(Modifier)` - If the name matches a known modifier
    /// * `None` - If the name is not recognized
    ///
    /// # Examples
    /// ```rust
    /// use wrtype::Modifier;
    ///
    /// // Standard modifier names (case-insensitive)
    /// assert_eq!(Modifier::from_name("shift"), Some(Modifier::Shift));
    /// assert_eq!(Modifier::from_name("CTRL"), Some(Modifier::Ctrl));
    /// assert_eq!(Modifier::from_name("Alt"), Some(Modifier::Alt));
    ///
    /// // Alternative names
    /// assert_eq!(Modifier::from_name("win"), Some(Modifier::Logo));
    /// assert_eq!(Modifier::from_name("logo"), Some(Modifier::Logo));
    ///
    /// // Invalid names return None
    /// assert_eq!(Modifier::from_name("super"), None);
    /// assert_eq!(Modifier::from_name("command"), None);
    /// assert_eq!(Modifier::from_name(""), None);
    /// ```
    ///
    /// # Accepted Names
    /// - `"shift"` → `Modifier::Shift`
    /// - `"capslock"` → `Modifier::CapsLock`
    /// - `"ctrl"` → `Modifier::Ctrl`
    /// - `"alt"` → `Modifier::Alt`
    /// - `"logo"` or `"win"` → `Modifier::Logo`
    /// - `"altgr"` → `Modifier::AltGr`
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "shift" => Some(Self::Shift),
            "capslock" => Some(Self::CapsLock),
            "ctrl" => Some(Self::Ctrl),
            "alt" => Some(Self::Alt),
            "logo" | "win" => Some(Self::Logo),
            "altgr" => Some(Self::AltGr),
            _ => None,
        }
    }
}

/// High-level client interface for wrtype functionality
///
/// This provides a simplified API for common use cases while still allowing
/// access to the lower-level Command system for complex scenarios.
///
/// # Architecture
/// 
/// WrtypeClient implements the Facade design pattern, providing a clean, simple
/// interface that hides the complexity of the underlying layers:
/// - Command parsing and sequencing
/// - Wayland protocol communication
/// - XKB keymap generation and management
/// - Timing and synchronization
///
/// # Design Patterns Used
/// - **Facade Pattern**: Simplifies the complex subsystem interaction
/// - **Command Pattern**: Operations are encapsulated as Command objects
/// - **Builder Pattern**: KeymapBuilder constructs XKB keymaps incrementally
/// - **Strategy Pattern**: Different command types use different execution strategies
///
/// # Examples
///
/// ```rust,no_run
/// use wrtype::{WrtypeClient, Modifier};
/// use std::time::Duration;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a client
/// let mut client = WrtypeClient::new()?;
///
/// // Type some text
/// client.type_text("Hello, Wayland!")?;
///
/// // Send a keyboard shortcut
/// client.send_shortcut(&[Modifier::Ctrl], "c")?; // Ctrl+C
///
/// // Complex sequence with timing
/// client.press_modifier(Modifier::Alt)?;
/// client.type_key("Tab")?;
/// client.sleep(Duration::from_millis(100))?;
/// client.release_modifier(Modifier::Alt)?;
/// # Ok(())
/// # }
/// ```
pub struct WrtypeClient {
    // COMPOSITION PATTERN: WrtypeClient owns a CommandExecutor
    // This encapsulates all the low-level implementation details
    executor: CommandExecutor,
}

impl WrtypeClient {
    /// Create a new wrtype client and establish Wayland connection
    ///
    /// This method automatically connects to the Wayland display server
    /// and sets up the virtual keyboard protocol. It will fail if the
    /// compositor doesn't support virtual keyboards.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use wrtype::WrtypeClient;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Connect to Wayland
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Now ready to send keyboard events
    /// client.type_text("Ready to type!")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// Returns an error if:
    /// - Cannot connect to Wayland compositor (no WAYLAND_DISPLAY)
    /// - Virtual keyboard protocol is not supported by compositor
    /// - Required Wayland objects cannot be created
    /// - Insufficient permissions to access Wayland socket
    pub fn new() -> Result<Self> {
        // ARCHITECTURAL PATTERN: Dependency injection via constructor
        // The WrtypeClient depends on CommandExecutor, which depends on Wayland connection
        // We establish the dependency chain here and encapsulate it
        let (connection, wayland_state) = connect_wayland()?;
        let executor = CommandExecutor::new(connection, wayland_state);
        
        // DESIGN PATTERN: Facade pattern - WrtypeClient provides a simplified interface
        // hiding the complexity of the executor, keymap, and Wayland protocol layers
        Ok(Self { executor })
    }

    /// Type a string of text with optional delay between characters
    ///
    /// # Arguments
    /// * `text` - The text to type
    /// * `delay` - Optional delay between each character (default: no delay)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::WrtypeClient;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    /// client.type_text("Hello, World!")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        self.type_text_with_delay(text, Duration::from_millis(0))
    }

    /// Type a string of text with specified delay between characters
    ///
    /// Useful for applications that need time to process each keystroke
    /// or when simulating human-like typing speed.
    ///
    /// # Arguments
    /// * `text` - The text to type (supports full UTF-8)
    /// * `delay` - Delay between each character
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::WrtypeClient;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Slow typing for screen recording
    /// client.type_text_with_delay("Hello, World!", Duration::from_millis(100))?;
    ///
    /// // Unicode support
    /// client.type_text_with_delay("café 🚀 résumé", Duration::from_millis(50))?;
    ///
    /// // Fast typing for automation
    /// client.type_text_with_delay("rapid input", Duration::from_millis(10))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn type_text_with_delay(&mut self, text: &str, delay: Duration) -> Result<()> {
        // DESIGN PATTERN: Command pattern - encapsulate the operation as a Command object
        // This allows for consistent handling, queuing, and execution of different operation types
        let command = Command::Text {
            text: text.to_string(),
            delay,
        };
        // ARCHITECTURAL PATTERN: Delegation - pass the command to the executor layer
        // The client layer focuses on API convenience, execution layer handles implementation
        self.executor.execute_commands(vec![command])
    }

    /// Press a key (key remains pressed until explicitly released)
    ///
    /// The key will remain in pressed state until `release_key()` is called
    /// with the same key name. Useful for key combinations or sustained input.
    ///
    /// # Arguments
    /// * `key` - XKB key name (case-sensitive)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::WrtypeClient;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Hold down arrow key for sustained movement
    /// client.press_key("Right")?;
    /// client.sleep(Duration::from_millis(500));
    /// client.release_key("Right")?;
    ///
    /// // Gaming: hold space for jump
    /// client.press_key("space")?;
    /// client.sleep(Duration::from_millis(200));
    /// client.release_key("space")?;
    ///
    /// // Function keys
    /// client.press_key("F11")?; // Fullscreen toggle
    /// client.release_key("F11")?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Common Key Names
    /// - Letters: `"a"`, `"b"`, `"z"` (lowercase)
    /// - Numbers: `"1"`, `"2"`, `"0"`
    /// - Special: `"space"`, `"Return"`, `"Tab"`, `"Escape"`
    /// - Arrows: `"Left"`, `"Right"`, `"Up"`, `"Down"`
    /// - Function: `"F1"`, `"F2"`, ..., `"F12"`
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        let command = Command::KeyPress(key.to_string());
        self.executor.execute_commands(vec![command])
    }

    /// Release a previously pressed key
    ///
    /// Should be paired with a corresponding `press_key()` call.
    /// Releasing a key that wasn't pressed by wrtype has no effect.
    ///
    /// # Arguments
    /// * `key` - XKB key name to release (must match press_key name exactly)
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::WrtypeClient;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Proper press/release pairing
    /// client.press_key("shift")?;
    /// client.press_key("Tab")?;      // Shift+Tab for reverse navigation
    /// client.release_key("Tab")?;
    /// client.release_key("shift")?;
    ///
    /// // Long key hold
    /// client.press_key("Delete")?;
    /// // Key stays pressed until released
    /// std::thread::sleep(std::time::Duration::from_millis(1000));
    /// client.release_key("Delete")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_key(&mut self, key: &str) -> Result<()> {
        let command = Command::KeyRelease(key.to_string());
        self.executor.execute_commands(vec![command])
    }

    /// Type a key (press and immediately release)
    ///
    /// Equivalent to calling `press_key()` followed immediately by `release_key()`.
    /// This is the most common way to simulate a single key press.
    ///
    /// # Arguments
    /// * `key` - XKB key name to type
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::WrtypeClient;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Navigation
    /// client.type_key("Return")?;     // Press Enter
    /// client.type_key("Tab")?;        // Tab to next field
    /// client.type_key("Escape")?;     // Cancel dialog
    ///
    /// // Function keys
    /// client.type_key("F5")?;         // Refresh
    /// client.type_key("F12")?;        // Developer tools
    ///
    /// // Media keys (if supported)
    /// client.type_key("XF86AudioPlay")?;  // Play/pause
    /// client.type_key("XF86AudioNext")?;  // Next track
    /// # Ok(())
    /// # }
    /// ```
    pub fn type_key(&mut self, key: &str) -> Result<()> {
        let commands = vec![
            Command::KeyPress(key.to_string()),
            Command::KeyRelease(key.to_string()),
        ];
        self.executor.execute_commands(commands)
    }

    /// Press a modifier key
    ///
    /// The modifier remains active until released with `release_modifier()`.
    /// Multiple modifiers can be pressed simultaneously.
    ///
    /// # Arguments
    /// * `modifier` - Modifier to press
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::{WrtypeClient, Modifier};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Simple modifier usage
    /// client.press_modifier(Modifier::Ctrl)?;
    /// client.type_key("c")?;  // Ctrl+C
    /// client.release_modifier(Modifier::Ctrl)?;
    ///
    /// // Multiple modifiers
    /// client.press_modifier(Modifier::Ctrl)?;
    /// client.press_modifier(Modifier::Shift)?;
    /// client.type_key("z")?;  // Ctrl+Shift+Z (Redo)
    /// client.release_modifier(Modifier::Shift)?;
    /// client.release_modifier(Modifier::Ctrl)?;
    ///
    /// // System shortcuts
    /// client.press_modifier(Modifier::Logo)?;
    /// client.type_key("d")?;  // Show desktop
    /// client.release_modifier(Modifier::Logo)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn press_modifier(&mut self, modifier: Modifier) -> Result<()> {
        let command = Command::ModPress(modifier);
        self.executor.execute_commands(vec![command])
    }

    /// Release a modifier key
    ///
    /// Should be paired with corresponding `press_modifier()` calls.
    /// Order matters when releasing multiple modifiers.
    ///
    /// # Arguments
    /// * `modifier` - Modifier to release
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::{WrtypeClient, Modifier};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Proper release order (reverse of press order)
    /// client.press_modifier(Modifier::Ctrl)?;
    /// client.press_modifier(Modifier::Alt)?;
    /// client.type_key("Delete")?;  // Ctrl+Alt+Delete
    /// client.release_modifier(Modifier::Alt)?;   // Release last pressed first
    /// client.release_modifier(Modifier::Ctrl)?;
    ///
    /// // Toggle caps lock
    /// client.press_modifier(Modifier::CapsLock)?;
    /// client.release_modifier(Modifier::CapsLock)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn release_modifier(&mut self, modifier: Modifier) -> Result<()> {
        let command = Command::ModRelease(modifier);
        self.executor.execute_commands(vec![command])
    }

    /// Sleep for specified duration
    ///
    /// Useful for timing control in complex sequences or waiting
    /// for applications to respond to input.
    ///
    /// # Arguments
    /// * `duration` - How long to sleep
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::WrtypeClient;
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Wait for application to load
    /// client.type_key("F5")?;  // Refresh
    /// client.sleep(Duration::from_secs(2))?;  // Wait for page load
    /// client.type_text("Now the page is loaded")?;
    ///
    /// // Precise timing for games
    /// client.type_key("space")?;  // Jump
    /// client.sleep(Duration::from_millis(16))?;  // 60 FPS timing
    /// client.type_key("space")?;  // Double jump
    ///
    /// // Micro delays for sensitive applications
    /// client.type_text("username")?;
    /// client.sleep(Duration::from_millis(50))?;
    /// client.type_key("Tab")?;
    /// client.sleep(Duration::from_millis(50))?;
    /// client.type_text("password")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sleep(&mut self, duration: Duration) -> Result<()> {
        let command = Command::Sleep(duration);
        self.executor.execute_commands(vec![command])
    }

    /// Execute a sequence of commands
    ///
    /// This is the most flexible interface, allowing complex key sequences
    /// with precise timing control. Commands are executed in order.
    ///
    /// # Arguments
    /// * `commands` - Vector of commands to execute in order
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::{WrtypeClient, Command, Modifier};
    /// # use std::time::Duration;
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    ///
    /// // Complex login sequence
    /// let login_sequence = vec![
    ///     Command::Text { text: "username".to_string(), delay: Duration::from_millis(50) },
    ///     Command::KeyPress("Tab".to_string()),
    ///     Command::KeyRelease("Tab".to_string()),
    ///     Command::Sleep(Duration::from_millis(100)),
    ///     Command::Text { text: "password".to_string(), delay: Duration::from_millis(50) },
    ///     Command::KeyPress("Return".to_string()),
    ///     Command::KeyRelease("Return".to_string()),
    /// ];
    /// client.execute_commands(login_sequence)?;
    ///
    /// // Text editing with shortcuts
    /// let edit_sequence = vec![
    ///     Command::ModPress(Modifier::Ctrl),
    ///     Command::KeyPress("a".to_string()),  // Select all
    ///     Command::KeyRelease("a".to_string()),
    ///     Command::ModRelease(Modifier::Ctrl),
    ///     Command::Text { text: "New content".to_string(), delay: Duration::ZERO },
    /// ];
    /// client.execute_commands(edit_sequence)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn execute_commands(&mut self, commands: Vec<Command>) -> Result<()> {
        self.executor.execute_commands(commands)
    }

    /// Convenience method for common keyboard shortcuts
    ///
    /// # Arguments
    /// * `modifiers` - Slice of modifiers to hold
    /// * `key` - Key to press while modifiers are held
    ///
    /// # Examples
    /// ```rust,no_run
    /// # use wrtype::{WrtypeClient, Modifier};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut client = WrtypeClient::new()?;
    /// // Ctrl+C
    /// client.send_shortcut(&[Modifier::Ctrl], "c")?;
    /// // Ctrl+Shift+T
    /// client.send_shortcut(&[Modifier::Ctrl, Modifier::Shift], "t")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn send_shortcut(&mut self, modifiers: &[Modifier], key: &str) -> Result<()> {
        let mut commands = Vec::new();

        // PHASE 1: Press all modifiers in forward order
        // This builds up the modifier state incrementally
        for &modifier in modifiers {
            commands.push(Command::ModPress(modifier));
        }

        // PHASE 2: Press and release the key while modifiers are held
        // This creates the actual shortcut key press
        commands.push(Command::KeyPress(key.to_string()));
        commands.push(Command::KeyRelease(key.to_string()));

        // PHASE 3: Release all modifiers in reverse order
        // DESIGN PATTERN: Stack discipline for proper nesting
        // Last-pressed modifier is first-released, maintaining proper order
        // This prevents modifier state corruption in complex sequences
        for &modifier in modifiers.iter().rev() {
            commands.push(Command::ModRelease(modifier));
        }

        // ARCHITECTURAL PATTERN: Command pattern - we build a sequence of commands
        // and execute them atomically, ensuring consistency
        self.executor.execute_commands(commands)
    }
}
