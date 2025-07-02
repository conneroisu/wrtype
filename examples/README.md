# wrtype Library Examples

This directory contains examples demonstrating how to use the `wrtype` Rust library for programmatic text input and key event simulation on Wayland.

## Examples

### 1. Basic Typing (`basic_typing.rs`)

The simplest example showing how to type text using the library.

**Features demonstrated:**
- Creating a wrtype client
- Typing basic text
- Typing Unicode characters (emojis)
- Character-by-character typing with delays

**Run with:**
```bash
cargo run --example basic_typing
```

### 2. Keyboard Shortcuts (`shortcuts.rs`)

Demonstrates sending common keyboard shortcuts and key combinations.

**Features demonstrated:**
- High-level shortcut API (`send_shortcut`)
- Common shortcuts (Ctrl+A, Ctrl+C, Ctrl+V)
- Complex combinations (Ctrl+Shift+T)
- Manual modifier control
- Function keys, arrow keys, and special keys

**Run with:**
```bash
cargo run --example shortcuts
```

### 3. Stdin Processing (`stdin_processing.rs`)

Shows how to process text from stdin and pipe it through wrtype.

**Features demonstrated:**
- Reading text from stdin
- Processing piped input
- Using the Command API
- Handling different input scenarios

**Run with:**
```bash
# With piped input
echo "Hello from stdin!" | cargo run --example stdin_processing

# With file input
cargo run --example stdin_processing < some_file.txt

# Without input (shows demonstration)
cargo run --example stdin_processing
```

### 4. Advanced Key Sequences (`advanced_sequences.rs`)

Complex automation scenarios and sophisticated key sequence handling.

**Features demonstrated:**
- Text editor workflow simulation
- Form filling automation
- Complex modifier combinations
- Gaming-style key sequences (WASD)
- Precise timing control
- Sequential command execution

**Run with:**
```bash
cargo run --example advanced_sequences
```

## API Overview

The examples showcase these main library components:

### WrtypeClient

High-level interface for common operations:
- `type_text()` - Type strings
- `type_text_with_delay()` - Type with character delays
- `press_key()` / `release_key()` - Individual key control
- `send_shortcut()` - Keyboard shortcuts
- `execute_commands()` - Execute command sequences

### Command API

Lower-level command interface for precise control:
- `Command::Text` - Type text with delays
- `Command::KeyPress` / `Command::KeyRelease` - Key events
- `Command::ModPress` / `Command::ModRelease` - Modifier keys
- `Command::Sleep` - Timing control
- `Command::StdinText` - Read from stdin

### Modifier Types

Supported modifier keys:
- `Modifier::Shift`
- `Modifier::Ctrl` 
- `Modifier::Alt`
- `Modifier::Logo` (Super/Windows key)
- `Modifier::AltGr`
- `Modifier::CapsLock`

## Requirements

- Wayland compositor with virtual-keyboard protocol support
- Rust with cargo
- Access to Wayland session (WAYLAND_DISPLAY set)

## Building

Build all examples:
```bash
cargo build --examples
```

Build specific example:
```bash
cargo build --example basic_typing
```

## Integration

These examples can be used as starting points for:
- Test automation
- Accessibility tools  
- Gaming macros
- Text processing pipelines
- Desktop automation scripts
- Development tools

## Error Handling

All examples include proper error handling patterns. The library uses `anyhow::Result` for comprehensive error reporting including:
- Wayland connection failures
- Protocol errors
- Invalid key names
- Timing issues