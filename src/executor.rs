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
            // Start with empty keymap - this will be populated on-demand as characters are encountered
            // This lazy approach avoids generating large keymaps for simple operations
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
        // SETUP PHASE: Upload initial empty keymap to establish protocol baseline
        // The Wayland virtual keyboard protocol requires a keymap before any key events can be sent
        // We start with an empty keymap and expand it dynamically as needed
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        // Roundtrip ensures the compositor has processed the keymap before we send events
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // EXECUTION PHASE: Execute all commands in the provided sequence
        // Commands are processed sequentially to maintain timing and ordering guarantees
        // Each command may modify the keymap, requiring re-upload to the compositor
        for command in commands {
            self.execute_command(command)?;
        }

        // CLEANUP PHASE: Release all modifiers to leave system in clean state
        // This prevents "sticky" modifiers that could affect other applications
        // Critical for system stability - modifiers left pressed can cause unexpected behavior
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
        // Dispatch to specialized handlers based on command type
        // Each handler encapsulates the specific logic and protocol interactions for that command
        match command {
            Command::Text { text, delay } => {
                // Type a complete string with character-by-character delay
                // This may require keymap updates for new Unicode characters
                self.type_text(&text, delay)?;
            }
            Command::ModPress(modifier) => {
                // Add modifier to current state using bitwise OR
                // Modifiers accumulate, allowing complex combinations
                self.press_modifier(modifier)?;
            }
            Command::ModRelease(modifier) => {
                // Remove modifier from current state using bitwise AND NOT
                // Safe to release non-pressed modifiers (no-op)
                self.release_modifier(modifier)?;
            }
            Command::KeyPress(key_name) => {
                // Press named key and leave it pressed until explicit release
                // Key name validation happens during keymap lookup
                self.press_key(&key_name)?;
            }
            Command::KeyRelease(key_name) => {
                // Release previously pressed key
                // Safe to release non-pressed keys (no-op)
                self.release_key(&key_name)?;
            }
            Command::Sleep(duration) => {
                // Simple sleep - no protocol interaction needed
                // Uses thread::sleep for precise timing control
                thread::sleep(duration);
            }
            Command::StdinText { delay } => {
                // Read and type text from stdin with UTF-8 boundary handling
                // More complex than regular text due to streaming nature
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
        // STEP 1: Pre-process the entire string to generate keycodes
        // This batch approach is more efficient than character-by-character keymap updates
        // The keymap builder caches lookups, so repeated characters are O(1)
        let keycodes = self.keymap.get_keycodes_for_text(text);

        // STEP 2: Upload updated keymap to compositor if new characters were added
        // The keymap may have grown to accommodate Unicode characters not seen before
        // We must upload the complete keymap before sending any events that reference new keycodes
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        // Roundtrip ensures compositor has processed and activated the new keymap
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // STEP 3: Type each character with appropriate inter-character delay
        // Using keycodes from step 1 ensures all characters are valid in the current keymap
        for keycode in keycodes {
            self.type_keycode(keycode)?;
            // Apply delay between characters for natural typing rhythm or application compatibility
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
        // PRESS PHASE: Send key press event
        self.wayland_state.press_key(keycode)?;
        // Roundtrip ensures the press event is processed before the release
        self.connection.roundtrip().context("Failed to roundtrip")?;
        // Small delay simulates natural key press duration (2ms is typical mechanical key travel time)
        thread::sleep(Duration::from_millis(2));

        // RELEASE PHASE: Send key release event
        self.wayland_state.release_key(keycode)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;
        // Small delay prevents key events from being too rapid for applications to process
        // Some applications have input rate limiting that can miss rapid-fire events
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
        // Read current modifier state and add the new modifier using bitwise OR
        // This allows multiple modifiers to be pressed simultaneously
        // Example: if Ctrl is already pressed (state=4), pressing Shift (1) results in state=5
        let current_mods = self.wayland_state.mod_state;
        let new_mods = current_mods | (modifier as u32);
        
        // Update both the Wayland state and our local tracking
        // The set_modifiers call handles the protocol details of depressed vs locked modifiers
        self.wayland_state.set_modifiers(new_mods)?;
        // Roundtrip ensures the modifier state is active before subsequent key events
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
        // Read current modifier state and remove the modifier using bitwise AND NOT
        // This preserves other pressed modifiers while removing only the specified one
        // Example: if Ctrl+Shift is pressed (state=5), releasing Shift (1) results in state=4
        let current_mods = self.wayland_state.mod_state;
        let new_mods = current_mods & !(modifier as u32);
        
        // Update modifier state - safe to release non-pressed modifiers (becomes no-op)
        self.wayland_state.set_modifiers(new_mods)?;
        // Roundtrip ensures the modifier release is processed
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
        // STEP 1: Resolve XKB key name to keycode (may add new keymap entry)
        // This validates the key name and assigns a keycode if it's not already in the keymap
        // Key name validation uses XKB's built-in keysym lookup with case-insensitive matching
        let keycode = self.keymap.get_keycode_for_key_name(key_name)?;

        // STEP 2: Upload updated keymap if we added a new key
        // The keymap may have grown to include the new key definition
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // STEP 3: Send only press event - key remains pressed until explicit release
        // This creates "sticky" key behavior useful for key combinations or sustained input
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
        // STEP 1: Resolve XKB key name to keycode (may add new keymap entry)
        // Even for release events, we need to ensure the key is defined in the keymap
        // This handles cases where release commands are given without corresponding press commands
        let keycode = self.keymap.get_keycode_for_key_name(key_name)?;

        // STEP 2: Upload updated keymap if we added a new key
        // Although unusual, this ensures consistency if the key wasn't previously defined
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // STEP 3: Send only release event
        // Safe to release keys that weren't pressed by wrtype - becomes a no-op at the compositor level
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
        // Initialize stdin reader and buffers for UTF-8 boundary handling
        let mut stdin = io::stdin();
        let mut buffer = [0u8; 8];  // Small buffer for incremental reading
        let mut incomplete_char = Vec::new();  // Accumulates bytes across buffer boundaries

        loop {
            // STEP 1: Read more bytes, starting after any incomplete character bytes
            // This preserves incomplete UTF-8 sequences across read boundaries
            // We read into the remaining space in the buffer after any incomplete bytes
            let bytes_read = stdin
                .read(&mut buffer[incomplete_char.len()..])
                .context("Failed to read from stdin")?;

            if bytes_read == 0 {
                break; // EOF reached - no more input available
            }

            // STEP 2: Combine any incomplete character bytes with newly read bytes
            // This creates a continuous byte stream that may span multiple read operations
            incomplete_char.extend_from_slice(&buffer[..bytes_read]);

            let mut processed = 0;
            
            // STEP 3: Process as many complete UTF-8 characters as possible
            while processed < incomplete_char.len() {
                // Attempt to decode UTF-8 characters from the accumulated bytes
                // We try to decode from the current position to handle partial characters
                match std::str::from_utf8(&incomplete_char[processed..]) {
                    Ok(s) => {
                        // Valid UTF-8 string found - process the first character
                        if let Some(ch) = s.chars().next() {
                            let char_len = ch.len_utf8();  // UTF-8 characters can be 1-4 bytes
                            self.type_character(ch, delay)?;
                            processed += char_len;
                        } else {
                            break; // No more complete characters in this segment
                        }
                    }
                    Err(error) => {
                        // UTF-8 decode error - handle partial sequences gracefully
                        if error.valid_up_to() > 0 {
                            // Some bytes form valid UTF-8, process those first
                            // This handles cases where valid UTF-8 is followed by incomplete sequences
                            let valid_str = std::str::from_utf8(
                                &incomplete_char[processed..processed + error.valid_up_to()],
                            )
                            .unwrap();  // Safe unwrap - we know this range is valid UTF-8
                            
                            // Process all valid characters in this segment
                            for ch in valid_str.chars() {
                                self.type_character(ch, delay)?;
                            }
                            processed += error.valid_up_to();
                        } else {
                            // Invalid UTF-8 at start - might need more bytes for complete character
                            // This is normal for multi-byte UTF-8 sequences split across reads
                            break;
                        }
                    }
                }
            }

            // STEP 4: Remove processed bytes, keep unprocessed ones for next iteration
            // This maintains incomplete UTF-8 sequences for the next read cycle
            incomplete_char.drain(..processed);

            // STEP 5: Safety mechanism - prevent infinite loops with invalid input
            // UTF-8 characters are at most 4 bytes, so more than 4 unprocessed bytes
            // indicates corrupted input that will never form a valid character
            if incomplete_char.len() > 4 {
                // Skip the first byte and continue - this handles binary data or corruption
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
        // STEP 1: Convert Unicode character to keycode (may add new keymap entry)
        // This handles the XKB keysym mapping and allocates a keycode if needed
        // The keymap builder caches lookups for performance on repeated characters
        let keycode = self.keymap.get_keycode_for_char(ch);

        // STEP 2: Upload updated keymap if we added a new character
        // Since this is called per-character from stdin, the keymap may grow frequently
        // The compositor needs the updated keymap before events using new keycodes
        let keymap_data = self.keymap.generate_keymap();
        self.wayland_state.upload_keymap(&keymap_data)?;
        // Roundtrip ensures keymap activation before key events
        self.connection.roundtrip().context("Failed to roundtrip")?;

        // STEP 3: Type the character using standard press+release sequence
        // This creates a complete key press event with proper timing
        self.type_keycode(keycode)?;

        // STEP 4: Apply character delay if specified
        // This delay comes after the key press, creating spacing between characters
        // Useful for applications that need time to process each character
        if !delay.is_zero() {
            thread::sleep(delay);
        }

        Ok(())
    }
}
