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
pub use wayland::{WaylandState, connect_wayland};

use anyhow::Result;
use std::time::Duration;

/// Internal command representation after parsing command-line arguments.
/// Each command represents a single action to be executed in sequence.
#[derive(Debug, Clone)]
pub enum Command {
    /// Type a string of text with specified delay between characters
    Text { text: String, delay: Duration },
    /// Press a modifier key (adds to current modifier state)
    ModPress(Modifier),
    /// Release a modifier key (removes from current modifier state)
    ModRelease(Modifier),
    /// Press a named key (key stays pressed until released)
    KeyPress(String),
    /// Release a named key
    KeyRelease(String),
    /// Sleep for specified duration (for timing control)
    Sleep(Duration),
    /// Read and type text from stdin with specified delay
    StdinText { delay: Duration },
}

/// Modifier keys with their corresponding bit values for Wayland protocol.
/// These values match the modifier mask constants used in XKB and Wayland.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Modifier {
    /// Shift key - bit 0 (value 1)
    Shift = 1,
    /// Caps Lock - bit 1 (value 2) - handled as locked modifier
    CapsLock = 2,
    /// Control key - bit 2 (value 4)
    Ctrl = 4,
    /// Alt key - bit 3 (value 8)
    Alt = 8,
    /// Logo/Super/Windows key - bit 6 (value 64)
    Logo = 64,
    /// AltGr (right Alt) key - bit 7 (value 128)
    AltGr = 128,
}

impl Modifier {
    /// Convert string modifier name to enum value.
    /// Accepts both "logo" and "win" for the Windows/Super key.
    /// Case-insensitive matching for user convenience.
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
pub struct WrtypeClient {
    executor: CommandExecutor,
}

impl WrtypeClient {
    /// Create a new wrtype client and establish Wayland connection
    ///
    /// # Errors
    /// Returns an error if:
    /// - Cannot connect to Wayland compositor
    /// - Virtual keyboard protocol is not supported
    /// - Required Wayland objects cannot be created
    pub fn new() -> Result<Self> {
        let (connection, wayland_state) = connect_wayland()?;
        let executor = CommandExecutor::new(connection, wayland_state);
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
    /// # Arguments
    /// * `text` - The text to type
    /// * `delay` - Delay between each character
    pub fn type_text_with_delay(&mut self, text: &str, delay: Duration) -> Result<()> {
        let command = Command::Text {
            text: text.to_string(),
            delay,
        };
        self.executor.execute_commands(vec![command])
    }

    /// Press a key (key remains pressed until explicitly released)
    ///
    /// # Arguments
    /// * `key` - XKB key name (e.g., "Return", "space", "ctrl", "a")
    pub fn press_key(&mut self, key: &str) -> Result<()> {
        let command = Command::KeyPress(key.to_string());
        self.executor.execute_commands(vec![command])
    }

    /// Release a previously pressed key
    ///
    /// # Arguments
    /// * `key` - XKB key name to release
    pub fn release_key(&mut self, key: &str) -> Result<()> {
        let command = Command::KeyRelease(key.to_string());
        self.executor.execute_commands(vec![command])
    }

    /// Type a key (press and immediately release)
    ///
    /// # Arguments
    /// * `key` - XKB key name to type
    pub fn type_key(&mut self, key: &str) -> Result<()> {
        let commands = vec![
            Command::KeyPress(key.to_string()),
            Command::KeyRelease(key.to_string()),
        ];
        self.executor.execute_commands(commands)
    }

    /// Press a modifier key
    ///
    /// # Arguments
    /// * `modifier` - Modifier to press (shift, ctrl, alt, etc.)
    pub fn press_modifier(&mut self, modifier: Modifier) -> Result<()> {
        let command = Command::ModPress(modifier);
        self.executor.execute_commands(vec![command])
    }

    /// Release a modifier key
    ///
    /// # Arguments
    /// * `modifier` - Modifier to release
    pub fn release_modifier(&mut self, modifier: Modifier) -> Result<()> {
        let command = Command::ModRelease(modifier);
        self.executor.execute_commands(vec![command])
    }

    /// Sleep for specified duration
    ///
    /// # Arguments
    /// * `duration` - How long to sleep
    pub fn sleep(&mut self, duration: Duration) -> Result<()> {
        let command = Command::Sleep(duration);
        self.executor.execute_commands(vec![command])
    }

    /// Execute a sequence of commands
    ///
    /// This is the most flexible interface, allowing complex key sequences
    /// with precise timing control.
    ///
    /// # Arguments
    /// * `commands` - Vector of commands to execute in order
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

        // Press all modifiers
        for &modifier in modifiers {
            commands.push(Command::ModPress(modifier));
        }

        // Press and release the key
        commands.push(Command::KeyPress(key.to_string()));
        commands.push(Command::KeyRelease(key.to_string()));

        // Release all modifiers (in reverse order)
        for &modifier in modifiers.iter().rev() {
            commands.push(Command::ModRelease(modifier));
        }

        self.executor.execute_commands(commands)
    }
}