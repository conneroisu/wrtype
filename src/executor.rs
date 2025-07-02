// Command execution engine for wrtype
//
// This module orchestrates the execution of parsed commands by coordinating between
// the keymap builder, Wayland protocol state, and timing requirements. It handles:
// - Sequential command execution with proper timing
// - Dynamic keymap updates and synchronization
// - Modifier state management
// - UTF-8 text processing from stdin
// - Key press/release sequencing with appropriate delays

use crate::keymap::KeymapBuilder;
use crate::wayland::WaylandState;
use crate::{Command, Modifier};
use anyhow::{Context, Result};
use std::io::{self, Read};
use std::thread;
use std::time::Duration;
use wayland_client::Connection;

/// Central command execution engine that coordinates all wrtype operations.
/// 
/// The executor maintains the complete state needed for virtual keyboard operation:
/// - Dynamic keymap builder for Unicode and named key support
/// - Wayland protocol state for communication with compositor
/// - Connection for protocol message synchronization
/// 
/// Commands are executed sequentially with proper timing and protocol synchronization.
pub struct CommandExecutor {
    /// Dynamic keymap builder - grows as new characters/keys are needed
    keymap: KeymapBuilder,
    /// Wayland virtual keyboard state and protocol objects
    wayland_state: WaylandState,
    /// Wayland connection for protocol roundtrips and synchronization
    connection: Connection,
}

impl CommandExecutor {
    /// Create a new command executor with initialized Wayland connection and state.
    /// 
    /// The executor starts with an empty keymap that will grow dynamically as
    /// characters and keys are encountered during command execution.
    ///
    /// # Arguments
    /// * `connection` - Active Wayland connection for protocol communication
    /// * `wayland_state` - Initialized virtual keyboard state with all required objects
    pub fn new(connection: Connection, wayland_state: WaylandState) -> Self {
        Self {
            keymap: KeymapBuilder::new(),
            wayland_state,
            connection,
        }
    }

    /// Execute a sequence of commands with proper setup and cleanup.
    /// 
    /// This method performs the complete execution cycle:
    /// 1. Upload initial empty keymap to establish protocol state
    /// 2. Execute all commands sequentially in the provided order
    /// 3. Clean up by releasing all pressed modifiers
    /// 
    /// Each command execution may update the keymap, requiring re-upload to
    /// the compositor. The method ensures proper protocol synchronization
    /// throughout the process.
    ///
    /// # Arguments
    /// * `commands` - Sequence of commands to execute in order
    ///
    /// # Returns
    /// * `Ok(())` - All commands executed successfully with cleanup complete
    /// * `Err` - Command execution or protocol communication failure
    pub fn execute_commands(&mut self, commands: Vec<Command>) -> Result<()> {
        // Upload initial empty keymap to establish protocol baseline
        // Even an empty keymap is required before sending any key events
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // Execute all commands in the provided sequence
        for command in commands {
            self.execute_command(command)?;
        }

        // Clean up: release all modifiers to leave system in clean state
        // This ensures no modifiers remain stuck after wrtype exits
        self.wayland_state.set_modifiers(0)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        Ok(())
    }

    /// Execute a single command with appropriate timing and protocol handling.
    /// 
    /// This method dispatches to the appropriate specialized handler based on
    /// command type. Each handler manages the specific protocol interactions
    /// and timing requirements for that command type.
    ///
    /// # Arguments
    /// * `command` - Single command to execute
    ///
    /// # Returns
    /// * `Ok(())` - Command executed successfully
    /// * `Err` - Command execution failure (protocol, timing, or I/O error)
    fn execute_command(&mut self, command: Command) -> Result<()> {
        match command {
            Command::Text { text, delay } => {
                self.type_text(&text, delay)?;
            }
            Command::ModPress(modifier) => {
                self.press_modifier(modifier)?;
            }
            Command::ModRelease(modifier) => {
                self.release_modifier(modifier)?;
            }
            Command::KeyPress(key_name) => {
                self.press_key(&key_name)?;
            }
            Command::KeyRelease(key_name) => {
                self.release_key(&key_name)?;
            }
            Command::Sleep(duration) => {
                // Simple sleep - no protocol interaction needed
                thread::sleep(duration);
            }
            Command::StdinText { delay } => {
                self.type_stdin(delay)?;
            }
        }
        Ok(())
    }

    /// Type a complete text string with specified inter-character delay.
    /// 
    /// This method processes the entire string to generate keycodes, updates
    /// the keymap if new characters were encountered, and then types each
    /// character sequentially with the specified delay between keystrokes.
    ///
    /// # Arguments
    /// * `text` - Text string to type
    /// * `delay` - Duration to wait between each character
    ///
    /// # Returns
    /// * `Ok(())` - Text typed successfully
    /// * `Err` - Keymap generation, protocol communication, or timing failure
    fn type_text(&mut self, text: &str, delay: Duration) -> Result<()> {
        // Convert all characters to keycodes (may add new keymap entries)
        let keycodes = self.keymap.get_keycodes_for_text(text);
        
        // Upload updated keymap to compositor if new characters were added
        // This must happen before typing any of the new characters
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // Type each character with appropriate delay
        for keycode in keycodes {
            self.type_keycode(keycode)?;
            if !delay.is_zero() {
                thread::sleep(delay);
            }
        }
        
        Ok(())
    }

    /// Type a single keycode with press+release sequence and minimal timing.
    /// 
    /// This method performs the fundamental key typing operation:
    /// 1. Press the key (send press event)
    /// 2. Small delay for natural timing
    /// 3. Release the key (send release event)  
    /// 4. Small delay before next operation
    ///
    /// # Arguments
    /// * `keycode` - Keycode to type (must exist in current keymap)
    ///
    /// # Returns
    /// * `Ok(())` - Key typed successfully
    /// * `Err` - Protocol communication failure
    fn type_keycode(&mut self, keycode: u32) -> Result<()> {
        // Send key press event
        self.wayland_state.press_key(keycode)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        // Small delay for natural key press timing
        thread::sleep(Duration::from_millis(2));
        
        // Send key release event
        self.wayland_state.release_key(keycode)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        // Small delay before next operation
        thread::sleep(Duration::from_millis(2));
        
        Ok(())
    }

    /// Press a modifier key by adding it to the current modifier state.
    /// 
    /// Modifier keys use bitwise OR to combine with existing modifiers,
    /// allowing multiple modifiers to be pressed simultaneously (e.g., Ctrl+Shift).
    ///
    /// # Arguments
    /// * `modifier` - Modifier to press (add to state)
    ///
    /// # Returns
    /// * `Ok(())` - Modifier pressed successfully
    /// * `Err` - Protocol communication failure
    fn press_modifier(&mut self, modifier: Modifier) -> Result<()> {
        let current_mods = self.wayland_state.mod_state;
        let new_mods = current_mods | (modifier as u32);
        self.wayland_state.set_modifiers(new_mods)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        Ok(())
    }

    /// Release a modifier key by removing it from the current modifier state.
    /// 
    /// Modifier keys use bitwise AND with NOT to remove from existing modifiers,
    /// while preserving other pressed modifiers.
    ///
    /// # Arguments
    /// * `modifier` - Modifier to release (remove from state)
    ///
    /// # Returns
    /// * `Ok(())` - Modifier released successfully
    /// * `Err` - Protocol communication failure
    fn release_modifier(&mut self, modifier: Modifier) -> Result<()> {
        let current_mods = self.wayland_state.mod_state;
        let new_mods = current_mods & !(modifier as u32);
        self.wayland_state.set_modifiers(new_mods)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        Ok(())
    }

    /// Press a named key and leave it pressed until explicitly released.
    /// 
    /// This method resolves the key name to a keycode, updates the keymap if
    /// the key is new, and sends only a press event. The key remains pressed
    /// until a corresponding release command is executed.
    ///
    /// # Arguments
    /// * `key_name` - XKB key name (e.g., "Return", "Left", "F1")
    ///
    /// # Returns
    /// * `Ok(())` - Key pressed successfully
    /// * `Err` - Unknown key name, keymap update failure, or protocol error
    fn press_key(&mut self, key_name: &str) -> Result<()> {
        // Resolve key name to keycode (may add new keymap entry)
        let keycode = self.keymap.get_keycode_for_key_name(key_name)?;
        
        // Upload updated keymap if we added a new key
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // Send only press event - key remains pressed
        self.wayland_state.press_key(keycode)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        Ok(())
    }

    /// Release a named key that was previously pressed.
    /// 
    /// This method resolves the key name to a keycode and sends only a release
    /// event. It should be paired with a corresponding press command to avoid
    /// releasing keys that weren't pressed by wrtype.
    ///
    /// # Arguments
    /// * `key_name` - XKB key name (e.g., "Return", "Left", "F1")
    ///
    /// # Returns
    /// * `Ok(())` - Key released successfully
    /// * `Err` - Unknown key name, keymap update failure, or protocol error
    fn release_key(&mut self, key_name: &str) -> Result<()> {
        // Resolve key name to keycode (may add new keymap entry)
        let keycode = self.keymap.get_keycode_for_key_name(key_name)?;
        
        // Upload updated keymap if we added a new key
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // Send only release event
        self.wayland_state.release_key(keycode)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        Ok(())
    }

    /// Read and type text from stdin with UTF-8 character boundary handling.
    /// 
    /// This method performs robust UTF-8 decoding from stdin by:
    /// 1. Reading bytes in chunks to handle partial character sequences
    /// 2. Maintaining incomplete character state across reads
    /// 3. Properly decoding multi-byte UTF-8 sequences
    /// 4. Typing each complete character with specified delay
    /// 
    /// The implementation handles edge cases like incomplete UTF-8 sequences
    /// at buffer boundaries and invalid UTF-8 input gracefully.
    ///
    /// # Arguments
    /// * `delay` - Duration to wait after typing each character
    ///
    /// # Returns
    /// * `Ok(())` - All stdin text processed successfully
    /// * `Err` - I/O error reading stdin or protocol communication failure
    fn type_stdin(&mut self, delay: Duration) -> Result<()> {
        let mut stdin = io::stdin();
        let mut buffer = [0u8; 8];
        let mut incomplete_char = Vec::new();

        loop {
            // Read more bytes, starting after any incomplete character bytes
            let bytes_read = stdin.read(&mut buffer[incomplete_char.len()..])
                .context("Failed to read from stdin")?;
            
            if bytes_read == 0 {
                break; // EOF reached
            }

            // Combine any incomplete character bytes with newly read bytes
            incomplete_char.extend_from_slice(&buffer[..bytes_read]);

            let mut processed = 0;
            while processed < incomplete_char.len() {
                // Attempt to decode UTF-8 characters from the accumulated bytes
                match std::str::from_utf8(&incomplete_char[processed..]) {
                    Ok(s) => {
                        // Valid UTF-8 string - process characters
                        if let Some(ch) = s.chars().next() {
                            let char_len = ch.len_utf8();
                            self.type_character(ch, delay)?;
                            processed += char_len;
                        } else {
                            break; // No more characters
                        }
                    }
                    Err(error) => {
                        if error.valid_up_to() > 0 {
                            // Some bytes form valid UTF-8, process those first
                            let valid_str = std::str::from_utf8(&incomplete_char[processed..processed + error.valid_up_to()])
                                .unwrap();
                            for ch in valid_str.chars() {
                                self.type_character(ch, delay)?;
                            }
                            processed += error.valid_up_to();
                        } else {
                            // Invalid UTF-8 at start - might need more bytes for complete character
                            break;
                        }
                    }
                }
            }

            // Remove processed bytes, keep unprocessed ones for next iteration
            incomplete_char.drain(..processed);
            
            // Safety: if we accumulate too many bytes without a valid character,
            // skip the first byte to avoid infinite loops with invalid input
            if incomplete_char.len() > 4 {
                incomplete_char.remove(0);
            }
        }

        Ok(())
    }

    /// Type a single Unicode character with keymap update and timing.
    /// 
    /// This method handles the complete process for typing one character:
    /// 1. Convert character to keycode (may update keymap)
    /// 2. Upload updated keymap to compositor if needed
    /// 3. Type the character using press+release sequence
    /// 4. Apply specified delay after typing
    ///
    /// # Arguments
    /// * `ch` - Unicode character to type
    /// * `delay` - Duration to wait after typing this character
    ///
    /// # Returns
    /// * `Ok(())` - Character typed successfully
    /// * `Err` - Keymap update failure or protocol communication error
    fn type_character(&mut self, ch: char, delay: Duration) -> Result<()> {
        // Convert character to keycode (may add new keymap entry)
        let keycode = self.keymap.get_keycode_for_char(ch);
        
        // Upload updated keymap if we added a new character
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // Type the character using standard press+release sequence
        self.type_keycode(keycode)?;
        
        // Apply character delay if specified
        if !delay.is_zero() {
            thread::sleep(delay);
        }
        
        Ok(())
    }
}