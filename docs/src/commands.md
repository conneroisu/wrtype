# Command Reference

Complete reference for all wrtype command-line options and usage patterns.

## Synopsis

```
wrtype [OPTIONS] [TEXT]...
```

## Arguments

### `[TEXT]...`

Text to type. Multiple text arguments are processed in order.

```bash
wrtype "First" "Second" "Third"
# Types: FirstSecondThird

wrtype "Hello" " " "World"  
# Types: Hello World
```

**Special Cases:**
- Use `--` to prevent text from being interpreted as options
- Use `-` as a placeholder for stdin input

```bash
wrtype -- "-starts-with-dash"
wrtype "Before" - "After" < input.txt
```

## Options

### Text Input

#### `--stdin`
Read text from standard input instead of (or in addition to) command line arguments.

```bash
echo "Hello" | wrtype --stdin
cat file.txt | wrtype --stdin -d 50
```

### Timing Control

#### `-d, --delay <TIME>`
Set delay in milliseconds between keystrokes. Default: 0.

```bash
wrtype -d 100 "Slow typing"
wrtype --delay 50 "Medium speed"
```

#### `-s, --sleep <TIME>`
Sleep for specified milliseconds before processing subsequent options. Can be used multiple times.

```bash
wrtype -P space -s 1000 -p space    # Hold space for 1 second
wrtype "Text" -s 500 "More text"    # 500ms pause between
```

### Modifier Keys

#### `-M, --press-modifier <MOD>`
Press (hold down) a modifier key. Can be used multiple times.

**Available Modifiers:**
- `shift` - Shift key
- `ctrl` - Control key  
- `alt` - Alt key
- `logo` / `win` - Windows/Super/Cmd key
- `altgr` - AltGr (right Alt) key
- `capslock` - Caps Lock key

```bash
wrtype -M ctrl -M shift t -m shift -m ctrl  # Ctrl+Shift+T
wrtype -M alt Tab -m alt                    # Alt+Tab
```

#### `-m, --release-modifier <MOD>`
Release a previously pressed modifier key.

```bash
# Hold Ctrl, type 'a', release Ctrl
wrtype -M ctrl a -m ctrl

# Multiple modifiers
wrtype -M ctrl -M shift -k Home -m shift -m ctrl
```

### Named Keys

#### `-P, --press-key <KEY>`
Press and hold a named key. Key remains pressed until released with `-p`.

```bash
wrtype -P shift            # Hold Shift
wrtype -P space -s 1000    # Hold Space for timing
```

#### `-p, --release-key <KEY>`
Release a previously pressed key.

```bash
wrtype -P Left -s 100 -p Left    # Brief Left arrow press
wrtype -P shift a -p shift       # Shift+A (capital A)
```

#### `-k, --type-key <KEY>`
Press and immediately release a key (equivalent to `-P <KEY> -p <KEY>`).

```bash
wrtype -k Return        # Press Enter
wrtype -k Tab -k Tab    # Press Tab twice
wrtype -k F5            # Press F5
```

**Common Key Names:**
- `Return` / `Enter` - Enter key
- `space` - Space bar
- `Tab` - Tab key
- `Escape` - Escape key
- `BackSpace` - Backspace
- `Delete` - Delete key
- `Left`, `Right`, `Up`, `Down` - Arrow keys
- `Home`, `End` - Navigation keys
- `Page_Up`, `Page_Down` - Page navigation
- `F1`-`F12` - Function keys
- `Insert` - Insert key

### Help and Information

#### `-h, --help`
Show help message and exit.

#### `-V, --version`
Show version information and exit.

```bash
wrtype --version
# Output: wrtype 0.1.0
```

## Command Processing Order

Commands are processed in a specific order regardless of their position on the command line:

1. **Text arguments** (including `-` stdin placeholders)
2. **Modifier press** (`-M`) commands
3. **Modifier release** (`-m`) commands  
4. **Key press** (`-P`) commands
5. **Key release** (`-p`) commands
6. **Type key** (`-k`) commands
7. **Sleep** (`-s`) commands
8. **Stdin flag** (`--stdin`)

```bash
# These are equivalent:
wrtype -s 100 -M ctrl "text" -m ctrl
wrtype "text" -M ctrl -m ctrl -s 100

# Processing order: text, press ctrl, release ctrl, sleep 100ms
```

## Usage Patterns

### Simple Text Typing

```bash
wrtype "Hello, World!"
wrtype -d 100 "Slow text"
echo "Piped" | wrtype --stdin
```

### Key Combinations

```bash
# Ctrl+C
wrtype -M ctrl c -m ctrl

# Ctrl+Shift+N  
wrtype -M ctrl -M shift n -m shift -m ctrl

# Alt+F4
wrtype -M alt -k F4 -m alt
```

### Precise Key Control

```bash
# Hold Shift, type 'hello', release Shift
wrtype -P shift hello -p shift

# Navigate with arrows
wrtype -k Left -k Left -k Down -k Right
```

### Timing Sequences

```bash
# Type, wait, type more
wrtype "First" -s 1000 "Second"

# Hold key with precise timing
wrtype -P space -s 500 -p space
```

### Mixed Input

```bash
# Combine stdin with other text
wrtype "Prefix: " - " :Suffix" < data.txt

# Complex automation
wrtype -M ctrl a -m ctrl -s 100 "New content" -k Return
```

## Exit Codes

- `0` - Success
- `1` - Invalid arguments or usage error
- Other - Runtime error (Wayland connection, protocol error, etc.)

## Environment Variables

### `WAYLAND_DISPLAY`
Specifies the Wayland display to connect to. Usually set automatically.

```bash
WAYLAND_DISPLAY=wayland-1 wrtype "Hello"
```

## Error Handling

wrtype provides clear error messages for common issues:

```bash
# Unknown key name
wrtype -k InvalidKey
# Error: Unknown key name: InvalidKey

# Invalid modifier  
wrtype -M badmod a -m badmod
# Error: Invalid modifier name: badmod

# No Wayland display
WAYLAND_DISPLAY= wrtype "test"
# Error: Failed to connect to Wayland display

# Unsupported protocol
wrtype "test"  # On compositor without virtual keyboard support
# Error: Compositor does not support the virtual keyboard protocol
```

## Performance Notes

- **Keymap Updates**: Adding new characters requires keymap regeneration
- **Protocol Overhead**: Each key event requires a Wayland roundtrip
- **Timing Accuracy**: Sleep timing is subject to system scheduler precision
- **Memory Usage**: Keymap grows with unique characters/keys used

For high-performance scenarios, consider batching operations and minimizing unique character sets.