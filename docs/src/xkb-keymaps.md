# XKB Keymaps

wrtype uses XKB (X Keyboard Extension) keymaps to define the relationship between keycodes and the characters or actions they produce. This document explains how wrtype generates and manages XKB keymaps dynamically.

## Overview

XKB keymaps are text files that describe keyboard layouts and behavior. wrtype generates these dynamically to support arbitrary Unicode characters and named keys.

## Keymap Structure

### Complete Keymap Format

```xkb
xkb_keymap {
    xkb_keycodes "wrtype" {
        minimum = 8;
        maximum = 15;
        <K1> = 9;
        <K2> = 10;
        <K3> = 11;
    };
    
    xkb_types "wrtype" { 
        include "complete" 
    };
    
    xkb_compatibility "wrtype" { 
        include "complete" 
    };
    
    xkb_symbols "wrtype" {
        key <K1> {[Return]};
        key <K2> {[space]};
        key <K3> {[a]};
    };
};
```

### Sections Explained

1. **keycodes**: Maps symbolic names (`<K1>`) to numeric keycodes (`9`)
2. **types**: Defines key behavior (wrtype uses standard types)
3. **compatibility**: Defines modifier behavior (wrtype uses standard compatibility)
4. **symbols**: Maps keycodes to keysyms (`Return`, `space`, `a`)

## Dynamic Generation Process

### 1. Character to Keysym Conversion

```rust
let keysym = match ch {
    '\n' => xkb::Keysym::from(KEY_Return),
    '\t' => xkb::Keysym::from(KEY_Tab),
    '\x1b' => xkb::Keysym::from(KEY_Escape),
    _ => xkb::utf32_to_keysym(ch as u32),
};
```

wrtype converts characters to XKB keysyms:
- **Special Characters**: Direct mapping (newline → Return)
- **Unicode Characters**: UTF-32 to keysym conversion
- **Named Keys**: String name to keysym lookup

### 2. Keycode Allocation

```rust
let keycode = self.entries.len() as u32 + 1; // 1-based keycodes
```

Keycodes are allocated sequentially:
- Start at 1 (XKB convention)
- Increment for each new character/key
- Cache assignments to avoid duplicates

### 3. Keymap Assembly

The keymap is built incrementally as new characters are encountered:

```rust
pub fn generate_keymap(&self) -> String {
    let mut keymap = String::new();
    
    // Header
    keymap.push_str("xkb_keymap {\n");
    
    // Keycodes section
    self.generate_keycodes_section(&mut keymap);
    
    // Standard sections
    keymap.push_str("xkb_types \"(unnamed)\" { include \"complete\" };\n");
    keymap.push_str("xkb_compatibility \"(unnamed)\" { include \"complete\" };\n");
    
    // Symbols section
    self.generate_symbols_section(&mut keymap);
    
    // Footer
    keymap.push_str("};\n");
    
    keymap
}
```

## Keysym Reference

### Common Keysyms

| Character | Keysym | Description |
|-----------|--------|-------------|
| `a`-`z` | `a`-`z` | Lowercase letters |
| `A`-`Z` | `A`-`Z` | Uppercase letters |
| `0`-`9` | `0`-`9` | Digits |
| ` ` | `space` | Space character |
| `\n` | `Return` | Newline/Enter |
| `\t` | `Tab` | Tab character |
| `\x1b` | `Escape` | Escape key |

### Special Keys

| Key Name | Keysym | Usage |
|----------|--------|--------|
| `Return` | `Return` | Enter key |
| `Tab` | `Tab` | Tab key |
| `space` | `space` | Space bar |
| `BackSpace` | `BackSpace` | Backspace |
| `Delete` | `Delete` | Delete key |
| `Left` | `Left` | Left arrow |
| `Right` | `Right` | Right arrow |
| `Up` | `Up` | Up arrow |
| `Down` | `Down` | Down arrow |
| `Home` | `Home` | Home key |
| `End` | `End` | End key |
| `Page_Up` | `Page_Up` | Page Up |
| `Page_Down` | `Page_Down` | Page Down |
| `F1`-`F12` | `F1`-`F12` | Function keys |

### Unicode Support

wrtype supports the full Unicode range through XKB's Unicode keysym space:

```rust
// Convert Unicode codepoint to keysym
let keysym = xkb::utf32_to_keysym(ch as u32);
```

Examples:
- `é` → `U00e9` (Latin small letter e with acute)
- `∀` → `U2200` (For all quantifier)
- `🚀` → `U1f680` (Rocket emoji)

## Keycode Mapping

### Linux Keycodes

XKB keycodes are offset by 8 from Linux keycodes:

| XKB Keycode | Linux Keycode | wrtype Internal |
|-------------|---------------|-----------------|
| 9 | 1 | 1 |
| 10 | 2 | 2 |
| 11 | 3 | 3 |
| ... | ... | ... |

### Allocation Strategy

```rust
fn add_entry(&mut self, keysym: xkb::Keysym, character: Option<char>) -> u32 {
    let keycode = self.entries.len() as u32 + 1; // 1-based
    
    let entry = KeymapEntry {
        keycode,          // Internal keycode (1-based)
        keysym,          // XKB keysym
        character,       // Optional Unicode character
    };
    
    self.entries.push(entry);
    // ... update caches ...
    
    keycode
}
```

Benefits:
- **Simple**: Sequential allocation
- **Predictable**: Same input always gets same keycode
- **Efficient**: O(1) allocation
- **Cacheable**: Fast lookup for repeated characters

## Optimization Strategies

### Caching

```rust
pub struct KeymapBuilder {
    entries: Vec<KeymapEntry>,
    char_to_keycode: HashMap<char, u32>,     // Character lookup
    symbol_to_keycode: HashMap<xkb::Keysym, u32>, // Keysym lookup
}
```

Two-level caching:
1. **Character Cache**: Direct character → keycode mapping
2. **Keysym Cache**: Keysym → keycode mapping

### Batch Updates

```rust
// Process entire text string before uploading keymap
let keycodes = self.keymap.get_keycodes_for_text(text);

// Single keymap upload for all new characters
let keymap_data = self.keymap.generate_keymap();
self.wayland_state.upload_keymap(&keymap_data)?;
```

Benefits:
- Reduces protocol roundtrips
- Minimizes compositor overhead
- Better performance for long text

### Memory Management

- **Incremental Growth**: Keymap grows only as needed
- **No Cleanup**: Keycodes remain stable throughout session
- **Bounded Size**: Practical limit on unique characters

## Advanced Features

### Custom Key Names

```rust
pub fn get_keycode_for_key_name(&mut self, name: &str) -> Result<u32> {
    let keysym = xkb::keysym_from_name(name, xkb::KEYSYM_CASE_INSENSITIVE);
    if keysym == xkb::Keysym::from(KEY_NoSymbol) {
        anyhow::bail!("Unknown key name: {}", name);
    }
    Ok(self.get_keycode_for_keysym(keysym))
}
```

Supports full XKB key name vocabulary:
- Standard keys: `Return`, `Tab`, `space`
- Navigation: `Left`, `Right`, `Up`, `Down`
- Function keys: `F1`, `F2`, ..., `F12`
- Modifiers: `Shift_L`, `Control_L`, `Alt_L`
- Special: `Print`, `Scroll_Lock`, `Pause`

### Error Handling

```rust
// Invalid key name
wrtype -k InvalidKeyName
// Error: Unknown key name: InvalidKeyName

// Unicode conversion failure (rare)
// Fallback to NoSymbol or error
```

### Debugging

View generated keymaps:

```rust
// Enable keymap debugging
WRTYPE_DEBUG_KEYMAP=1 wrtype "test"

// Outputs generated keymap to stderr
```

Example output:
```xkb
xkb_keymap {
xkb_keycodes "(unnamed)" {
minimum = 8;
maximum = 12;
<K1> = 9;
<K2> = 10;
<K3> = 11;
<K4> = 12;
};
xkb_types "(unnamed)" { include "complete" };
xkb_compatibility "(unnamed)" { include "complete" };
xkb_symbols "(unnamed)" {
key <K1> {[t]};
key <K2> {[e]};
key <K3> {[s]};
key <K4> {[t]};
};
};
```

## Best Practices

### Performance

1. **Batch Character Processing**: Process strings before individual characters
2. **Reuse Keymaps**: Cache generated keymaps when possible
3. **Minimize Updates**: Only regenerate when new characters added

### Compatibility

1. **Standard Keysyms**: Use well-known keysyms when possible
2. **Fallback Handling**: Handle unknown characters gracefully
3. **Validator Testing**: Test with strict XKB validators

### Debugging

1. **Keymap Inspection**: Save generated keymaps for analysis
2. **Keysym Verification**: Verify keysym names are correct
3. **Protocol Monitoring**: Watch Wayland protocol traffic

This XKB keymap system provides wrtype with the flexibility to handle any Unicode character while maintaining compatibility with standard keyboard layouts and compositor expectations.