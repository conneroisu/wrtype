// XKB keymap generation and management for wrtype
//
// This module provides dynamic keymap generation for typing arbitrary Unicode characters
// and named keys. It builds XKB keymaps on-demand by:
// - Converting Unicode characters to XKB keysyms
// - Handling special character mappings (newline, tab, escape)
// - Generating complete XKB keymap files
// - Managing keycode allocation and caching

use anyhow::Result;
use std::collections::HashMap;
use xkbcommon::xkb;
use xkbcommon::xkb::keysyms::*;

/// Dynamic keymap builder that creates XKB keymaps for arbitrary characters and keys.
/// 
/// The builder maintains a growing collection of keymap entries and generates
/// complete XKB keymap files that can be uploaded to the Wayland compositor.
/// It provides caching to avoid duplicate entries and ensures stable keycode
/// assignments across multiple operations.
pub struct KeymapBuilder {
    /// All keymap entries in order (keycode = index + 1)
    entries: Vec<KeymapEntry>,
    /// Fast lookup cache: character -> keycode
    char_to_keycode: HashMap<char, u32>,
    /// Fast lookup cache: keysym -> keycode  
    symbol_to_keycode: HashMap<xkb::Keysym, u32>,
}

/// A single entry in the keymap defining the relationship between
/// keycode, keysym, and optional Unicode character.
#[derive(Debug, Clone)]
pub struct KeymapEntry {
    /// XKB keycode (1-based, offset by 8 for Linux keycodes)
    pub keycode: u32,
    /// XKB keysym identifier
    pub keysym: xkb::Keysym,
    /// Associated Unicode character (if any)
    pub character: Option<char>,
}

impl KeymapBuilder {
    /// Create a new empty keymap builder.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            char_to_keycode: HashMap::new(),
            symbol_to_keycode: HashMap::new(),
        }
    }

    /// Get or create a keycode for a Unicode character.
    /// 
    /// This method handles the mapping from Unicode characters to XKB keysyms
    /// and assigns stable keycodes. Special characters like newline, tab, and
    /// escape are mapped to their corresponding XKB keysyms.
    ///
    /// # Arguments
    /// * `ch` - Unicode character to get keycode for
    ///
    /// # Returns
    /// * Keycode (1-based) that can be used with the virtual keyboard
    pub fn get_keycode_for_char(&mut self, ch: char) -> u32 {
        // Check cache first for fast lookup
        if let Some(&keycode) = self.char_to_keycode.get(&ch) {
            return keycode;
        }

        // Handle special character remapping to appropriate XKB keysyms
        // These characters need special treatment as they don't map directly
        let keysym = match ch {
            '\n' => xkb::Keysym::from(KEY_Return),     // Newline -> Return key
            '\t' => xkb::Keysym::from(KEY_Tab),        // Tab -> Tab key  
            '\x1b' => xkb::Keysym::from(KEY_Escape),   // ESC -> Escape key
            // For all other characters, use XKB's Unicode-to-keysym conversion
            _ => xkb::utf32_to_keysym(ch as u32),
        };

        // Add new entry to keymap and return assigned keycode
        self.add_entry(keysym, Some(ch))
    }

    /// Get or create a keycode for an XKB keysym.
    /// 
    /// This method is used for named keys that don't necessarily correspond
    /// to printable characters (like arrow keys, function keys, etc.).
    ///
    /// # Arguments
    /// * `keysym` - XKB keysym to get keycode for
    ///
    /// # Returns  
    /// * Keycode (1-based) that can be used with the virtual keyboard
    pub fn get_keycode_for_keysym(&mut self, keysym: xkb::Keysym) -> u32 {
        // Check cache first for fast lookup
        if let Some(&keycode) = self.symbol_to_keycode.get(&keysym) {
            return keycode;
        }

        // Add new entry without associated character
        self.add_entry(keysym, None)
    }

    /// Get or create a keycode for a named key.
    /// 
    /// This method converts key names (like "Return", "Left", "F1") to
    /// XKB keysyms and assigns keycodes. Key names are case-insensitive.
    ///
    /// # Arguments
    /// * `name` - XKB key name (e.g., "Return", "Left", "space")
    ///
    /// # Returns
    /// * `Ok(keycode)` - Successfully resolved keycode
    /// * `Err` - Unknown or invalid key name
    pub fn get_keycode_for_key_name(&mut self, name: &str) -> Result<u32> {
        // Convert key name to keysym using XKB lookup
        let keysym = xkb::keysym_from_name(name, xkb::KEYSYM_CASE_INSENSITIVE);
        
        // Check if the key name was valid
        if keysym == xkb::Keysym::from(KEY_NoSymbol) {
            anyhow::bail!("Unknown key name: {}", name);
        }
        
        Ok(self.get_keycode_for_keysym(keysym))
    }

    /// Add a new entry to the keymap and update caches.
    /// 
    /// This method allocates the next available keycode, creates a keymap entry,
    /// and updates the lookup caches for future fast access.
    ///
    /// # Arguments
    /// * `keysym` - XKB keysym for this entry
    /// * `character` - Optional Unicode character associated with this keysym
    ///
    /// # Returns
    /// * Newly allocated keycode (1-based)
    fn add_entry(&mut self, keysym: xkb::Keysym, character: Option<char>) -> u32 {
        // Keycodes start at 1 (XKB convention)
        let keycode = self.entries.len() as u32 + 1;
        
        // Create new keymap entry
        let entry = KeymapEntry {
            keycode,
            keysym,
            character,
        };
        
        // Add to entries list
        self.entries.push(entry);
        
        // Update lookup caches for fast future access
        if let Some(ch) = character {
            self.char_to_keycode.insert(ch, keycode);
        }
        self.symbol_to_keycode.insert(keysym, keycode);
        
        keycode
    }

    /// Generate a complete XKB keymap file in text format.
    /// 
    /// This method creates a full XKB keymap that can be uploaded to the
    /// Wayland compositor. The keymap includes all necessary sections:
    /// - keycodes: Maps symbolic names to numeric keycodes
    /// - types: Key behavior definitions (uses standard types)
    /// - compatibility: Modifier behavior (uses standard compat)
    /// - symbols: Maps keycodes to keysyms
    ///
    /// The generated keymap follows XKB conventions:
    /// - Keycodes start at 8 (Linux kernel offset)
    /// - Uses standard type and compatibility rules
    /// - Each key maps to exactly one keysym (no modifier variants)
    ///
    /// # Returns
    /// * Complete XKB keymap as a string, ready for upload to compositor
    pub fn generate_keymap(&self) -> String {
        let mut keymap = String::new();
        
        // Start of complete XKB keymap
        keymap.push_str("xkb_keymap {\n");
        
        // Generate keycodes section - maps symbolic names to numbers
        keymap.push_str("xkb_keycodes \"(unnamed)\" {\n");
        keymap.push_str("minimum = 8;\n"); // Linux keycode offset
        keymap.push_str(&format!("maximum = {};\n", self.entries.len() + 8 + 1));
        
        // Define keycode mappings: <K1> = 9, <K2> = 10, etc.
        // XKB keycodes are offset by 8 from our 1-based internal keycodes
        for (i, _entry) in self.entries.iter().enumerate() {
            keymap.push_str(&format!("<K{}> = {};\n", i + 1, i + 8 + 1));
        }
        keymap.push_str("};\n");
        
        // Include standard type definitions for key behavior
        // This provides standard key press/release semantics
        keymap.push_str("xkb_types \"(unnamed)\" { include \"complete\" };\n");
        
        // Include standard compatibility rules for modifier behavior
        // This ensures modifiers work as expected
        keymap.push_str("xkb_compatibility \"(unnamed)\" { include \"complete\" };\n");
        
        // Generate symbols section - maps keycodes to keysyms
        keymap.push_str("xkb_symbols \"(unnamed)\" {\n");
        for (i, _entry) in self.entries.iter().enumerate() {
            // Get the symbolic name for this keysym
            let keysym_name = xkb::keysym_get_name(_entry.keysym);
            // Define key mapping: key <K1> {[Return]};
            keymap.push_str(&format!("key <K{}> {{[{}]}};\n", i + 1, keysym_name));
        }
        keymap.push_str("};\n");
        
        // End of complete XKB keymap
        keymap.push_str("};\n");
        
        keymap
    }

    /// Convert a text string to a sequence of keycodes.
    /// 
    /// This is a convenience method that processes an entire string and
    /// returns the keycodes needed to type it. Each character is processed
    /// individually and may result in new keymap entries being created.
    ///
    /// # Arguments
    /// * `text` - String to convert to keycodes
    ///
    /// # Returns
    /// * Vector of keycodes in the same order as the input characters
    pub fn get_keycodes_for_text(&mut self, text: &str) -> Vec<u32> {
        text.chars()
            .map(|ch| self.get_keycode_for_char(ch))
            .collect()
    }
}