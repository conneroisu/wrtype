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

impl Default for KeymapBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KeymapBuilder {
    /// Create a new empty keymap builder.
    pub fn new() -> Self {
        Self {
            // Start with empty collections - we use lazy allocation for efficiency
            // The Vec grows as keys are added, while HashMaps provide O(1) lookups
            entries: Vec::new(),
            // Cache maps for fast lookup - avoids repeated XKB keysym resolution
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
    ///
    /// # Examples
    /// ```rust
    /// # use wrtype::KeymapBuilder;
    /// let mut builder = KeymapBuilder::new();
    ///
    /// // Basic ASCII characters
    /// let a_key = builder.get_keycode_for_char('a');
    /// let space_key = builder.get_keycode_for_char(' ');
    /// let num_key = builder.get_keycode_for_char('5');
    ///
    /// // Unicode characters
    /// let emoji_key = builder.get_keycode_for_char('\u{1F600}'); // 😀
    /// let accented_key = builder.get_keycode_for_char('\u{00E9}'); // é
    ///
    /// // Special characters (mapped to XKB keys)
    /// let enter_key = builder.get_keycode_for_char('\n');   // Maps to Return
    /// let tab_key = builder.get_keycode_for_char('\t');     // Maps to Tab
    /// let escape_key = builder.get_keycode_for_char('\x1b'); // Maps to Escape
    ///
    /// // Cached lookups are fast
    /// let same_a_key = builder.get_keycode_for_char('a');
    /// assert_eq!(a_key, same_a_key);
    /// ```
    pub fn get_keycode_for_char(&mut self, ch: char) -> u32 {
        // FAST PATH: Check cache first for O(1) lookup
        // This is critical for performance when typing repeated characters
        if let Some(&keycode) = self.char_to_keycode.get(&ch) {
            return keycode;
        }

        // SLOW PATH: Handle special character remapping to appropriate XKB keysyms
        // These control characters need special treatment as they don't map directly to Unicode keysyms
        // The XKB protocol defines specific keysyms for common control characters
        let keysym = match ch {
            '\n' => xkb::Keysym::from(KEY_Return), // Newline -> Return key (standard mapping)
            '\t' => xkb::Keysym::from(KEY_Tab),    // Tab -> Tab key (standard mapping)
            '\x1b' => xkb::Keysym::from(KEY_Escape), // ESC -> Escape key (standard mapping)
            // For all other characters, use XKB's Unicode-to-keysym conversion
            // This handles the full Unicode range including emoji, accented characters, etc.
            _ => xkb::utf32_to_keysym(ch as u32),
        };

        // Add new entry to keymap and return assigned keycode
        // This updates both the entries list and the lookup caches
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
        // FAST PATH: Check cache first for O(1) lookup
        // Keysym lookups are less common than character lookups but still benefit from caching
        if let Some(&keycode) = self.symbol_to_keycode.get(&keysym) {
            return keycode;
        }

        // SLOW PATH: Add new entry without associated character
        // This is used for named keys (like F1, arrows) that don't correspond to printable characters
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
    ///
    /// # Examples
    /// ```rust
    /// # use wrtype::KeymapBuilder;
    /// let mut builder = KeymapBuilder::new();
    ///
    /// // Navigation keys
    /// let return_key = builder.get_keycode_for_key_name("Return").unwrap();
    /// let left_key = builder.get_keycode_for_key_name("Left").unwrap();
    /// let right_key = builder.get_keycode_for_key_name("Right").unwrap();
    /// let up_key = builder.get_keycode_for_key_name("Up").unwrap();
    /// let down_key = builder.get_keycode_for_key_name("Down").unwrap();
    ///
    /// // Function keys
    /// let f1_key = builder.get_keycode_for_key_name("F1").unwrap();
    /// let f12_key = builder.get_keycode_for_key_name("F12").unwrap();
    ///
    /// // Special keys
    /// let space_key = builder.get_keycode_for_key_name("space").unwrap();
    /// let tab_key = builder.get_keycode_for_key_name("Tab").unwrap();
    /// let escape_key = builder.get_keycode_for_key_name("Escape").unwrap();
    ///
    /// // Case insensitive (using valid XKB key names)
    /// let return_key1 = builder.get_keycode_for_key_name("return").unwrap();
    /// let return_key2 = builder.get_keycode_for_key_name("RETURN").unwrap();
    /// let return_key3 = builder.get_keycode_for_key_name("Return").unwrap();
    /// assert_eq!(return_key1, return_key2);
    /// assert_eq!(return_key2, return_key3);
    ///
    /// // Invalid key names return errors
    /// assert!(builder.get_keycode_for_key_name("InvalidKey").is_err());
    /// assert!(builder.get_keycode_for_key_name("").is_err());
    /// ```
    pub fn get_keycode_for_key_name(&mut self, name: &str) -> Result<u32> {
        // Convert key name to keysym using XKB's built-in lookup table
        // This uses the standard XKB keysym database with case-insensitive matching
        // Examples: "Return" -> Return keysym, "F1" -> F1 keysym, "space" -> space keysym
        let keysym = xkb::keysym_from_name(name, xkb::KEYSYM_CASE_INSENSITIVE);

        // Check if the key name was valid according to XKB standards
        // KEY_NoSymbol is the sentinel value returned for unknown key names
        if keysym == xkb::Keysym::from(KEY_NoSymbol) {
            anyhow::bail!("Unknown key name: {}", name);
        }

        // Convert the validated keysym to our internal keycode representation
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
        // Allocate next available keycode - XKB convention starts at 1
        // Our internal keycodes are 1-based, but will be offset by 8 for Linux kernel compatibility
        let keycode = self.entries.len() as u32 + 1;

        // Create new keymap entry with all required fields
        // This represents a single key definition in the XKB keymap
        let entry = KeymapEntry {
            keycode,
            keysym,
            character,
        };

        // Add to the ordered list of entries
        // The index in this Vec corresponds to the keycode (offset by 1)
        self.entries.push(entry);

        // Update lookup caches for fast future access
        // These HashMaps provide O(1) lookup time for repeated key usage
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
    ///
    /// # Examples
    /// ```rust
    /// # use wrtype::KeymapBuilder;
    /// let mut builder = KeymapBuilder::new();
    ///
    /// // Add some characters to the keymap
    /// builder.get_keycode_for_char('a');
    /// builder.get_keycode_for_char('b');
    /// builder.get_keycode_for_char('1');
    /// builder.get_keycode_for_key_name("Return").unwrap();
    ///
    /// // Generate the complete XKB keymap
    /// let keymap = builder.generate_keymap();
    ///
    /// // The keymap contains all required XKB sections
    /// assert!(keymap.contains("xkb_keymap {"));
    /// assert!(keymap.contains("xkb_keycodes"));
    /// assert!(keymap.contains("xkb_types"));
    /// assert!(keymap.contains("xkb_compatibility"));
    /// assert!(keymap.contains("xkb_symbols"));
    ///
    /// // Keycode mappings are present
    /// assert!(keymap.contains("<K1> = 9;"));  // First key
    /// assert!(keymap.contains("minimum = 8;"));  // Linux offset
    ///
    /// // Symbol mappings for our characters
    /// assert!(keymap.contains("key <K1>"));  // Key definitions
    ///
    /// println!("Generated keymap:");
    /// println!("{}", keymap);
    /// ```
    ///
    /// # Generated Keymap Structure
    /// ```text
    /// xkb_keymap {
    ///   xkb_keycodes "(unnamed)" {
    ///     minimum = 8;
    ///     maximum = 12;
    ///     <K1> = 9;
    ///     <K2> = 10;
    ///     // ... more keycodes
    ///   };
    ///   xkb_types "(unnamed)" { include "complete" };
    ///   xkb_compatibility "(unnamed)" { include "complete" };
    ///   xkb_symbols "(unnamed)" {
    ///     key <K1> {[a]};
    ///     key <K2> {[Return]};
    ///     // ... more symbols
    ///   };
    /// };
    /// ```
    pub fn generate_keymap(&self) -> String {
        let mut keymap = String::new();

        // Start of complete XKB keymap - this is the root container for all sections
        keymap.push_str("xkb_keymap {\n");

        // SECTION 1: Generate keycodes section - maps symbolic names to numeric keycodes
        // This section defines the available keycodes and their symbolic names
        keymap.push_str("xkb_keycodes \"(unnamed)\" {\n");
        
        // Linux kernel requires keycodes to start at 8 (historical X11 compatibility)
        keymap.push_str("minimum = 8;\n"); 
        // Maximum keycode is our highest entry plus the Linux offset plus safety margin
        keymap.push_str(&format!("maximum = {};\n", self.entries.len() + 8 + 1));

        // Define keycode mappings: <K1> = 9, <K2> = 10, etc.
        // The symbolic names <K1>, <K2> will be referenced in the symbols section
        // XKB keycodes are offset by 8 from our 1-based internal keycodes
        for (i, _entry) in self.entries.iter().enumerate() {
            keymap.push_str(&format!("<K{}> = {};\n", i + 1, i + 8 + 1));
        }
        keymap.push_str("};\n");

        // SECTION 2: Include standard type definitions for key behavior
        // Types define how keys behave (normal keys, modifier keys, etc.)
        // We use the standard "complete" types which handle most use cases
        keymap.push_str("xkb_types \"(unnamed)\" { include \"complete\" };\n");

        // SECTION 3: Include standard compatibility rules for modifier behavior
        // Compatibility rules define how modifiers affect key behavior and LED states
        // The "complete" compatibility provides standard modifier semantics
        keymap.push_str("xkb_compatibility \"(unnamed)\" { include \"complete\" };\n");

        // SECTION 4: Generate symbols section - maps keycodes to keysyms
        // This is where we define what each key actually produces when pressed
        keymap.push_str("xkb_symbols \"(unnamed)\" {\n");
        for (i, _entry) in self.entries.iter().enumerate() {
            // Get the symbolic name for this keysym (e.g., "Return", "space", "a")
            let keysym_name = xkb::keysym_get_name(_entry.keysym);
            // Define key mapping: key <K1> {[Return]}; - maps symbolic keycode to keysym
            // The square brackets indicate this is the base level (no modifiers)
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
    ///
    /// # Examples
    /// ```rust
    /// # use wrtype::KeymapBuilder;
    /// let mut builder = KeymapBuilder::new();
    ///
    /// // Convert simple text
    /// let keycodes = builder.get_keycodes_for_text("hello");
    /// assert_eq!(keycodes.len(), 5);
    ///
    /// // Each character gets a unique keycode
    /// let h_code = builder.get_keycode_for_char('h');
    /// let e_code = builder.get_keycode_for_char('e');
    /// let l_code = builder.get_keycode_for_char('l');
    /// let o_code = builder.get_keycode_for_char('o');
    /// assert_eq!(keycodes, vec![h_code, e_code, l_code, l_code, o_code]);
    ///
    /// // Unicode strings work too
    /// let unicode_codes = builder.get_keycodes_for_text("café 🎉");
    /// assert_eq!(unicode_codes.len(), 6); // c, a, f, é, space, 🎉
    ///
    /// // Empty string returns empty vector
    /// let empty_codes = builder.get_keycodes_for_text("");
    /// assert_eq!(empty_codes, vec![]);
    ///
    /// // Special characters are handled
    /// let special_codes = builder.get_keycodes_for_text("line1\nline2\t");
    /// // \n becomes Return key, \t becomes Tab key
    /// ```
    ///
    /// # Use Cases
    /// - **Text typing**: Convert user input to keyboard events
    /// - **Bulk processing**: Handle entire sentences or paragraphs
    /// - **Template expansion**: Convert text templates to key sequences
    /// - **Automation scripts**: Type predefined text blocks
    pub fn get_keycodes_for_text(&mut self, text: &str) -> Vec<u32> {
        // Process each Unicode character in the string and convert to keycodes
        // This handles multi-byte UTF-8 sequences correctly via Rust's char iterator
        // The keymap builder will cache repeated characters for efficiency
        text.chars()
            .map(|ch| self.get_keycode_for_char(ch))
            .collect()
    }
}
