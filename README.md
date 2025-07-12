# wrtype

A Rust implementation of wtype - xdotool type for Wayland.

## Features

- **Text Input**: Type unicode characters
- **Modifier Keys**: Press/release modifiers (shift, capslock, ctrl, logo, win, alt, altgr)
- **Named Keys**: Press/release named keys using XKB key names
- **Delays**: Configurable delays between keystrokes and sleep commands
- **Stdin Support**: Read text from stdin for piped input

## Usage

```bash
# Type unicode characters
wrtype ∇⋅∇ψ = ρ

# Press Ctrl+C
wrtype -M ctrl c -m ctrl

# Delay between keystrokes
wrtype foo -d 120 bar

# Read from stdin with delay
echo "everything" | wrtype --stdin -d 12

# Press and release the Left key
wrtype -P left -p left

# Hold the Right key for 1000ms
wrtype -P right -s 1000 -p right
```

## Building

### Using Nix (Recommended)

```bash
nix develop
cargo build --release
```

### Using Cargo

Ensure you have the following system dependencies:
- libxkbcommon-dev
- wayland-dev
- wayland-protocols

```bash
cargo build --release
```

## Options

- `-M <MOD>`: Press modifier (shift, capslock, ctrl, logo, win, alt, altgr)
- `-m <MOD>`: Release modifier
- `-P <KEY>`: Press key
- `-p <KEY>`: Release key
- `-k <KEY>`: Type (press and release) key
- `-d <TIME>`: Sleep for TIME milliseconds between keystrokes
- `-s <TIME>`: Sleep for TIME milliseconds before interpreting following options
- `--stdin`: Read text from stdin

## Architecture

The implementation consists of several modules:

- **Command Line Interface**: Uses clap for argument parsing
- **Wayland Protocol**: Implements virtual keyboard protocol using wayland-client
- **Keymap Generation**: Dynamic XKB keymap generation for unicode support
- **Command Execution**: Sequentially executes typing commands with proper timing

## Library Usage

wrtype can be used as a Rust library for programmatic text input and automation:

```rust
use wrtype::{WrtypeClient, Modifier};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = WrtypeClient::new()?;
    
    // Type text
    client.type_text("Hello, Wayland!")?;
    
    // Send keyboard shortcuts
    client.send_shortcut(&[Modifier::Ctrl], "c")?;
    
    Ok(())
}
```

Add to your `Cargo.toml`:
```toml
[dependencies]
wrtype = { git = "https://github.com/conneroisu/wrtype" }
```

## Examples

The `examples/` directory contains comprehensive examples showing different use cases:

### Quick Start

```bash
# Run the interactive example runner
./run-examples.sh all

# Or run specific examples
./run-examples.sh basic      # Basic text typing
./run-examples.sh shortcuts  # Keyboard shortcuts
./run-examples.sh stdin      # Stdin processing
./run-examples.sh advanced   # Complex sequences
```

### Using just (if available)

```bash
# Quick start guide
just quick-start

# Run specific examples
just example-basic
just example-shortcuts
just examples-all

# Development commands
just build
just test
just dev
```

### Available Examples

1. **Basic Typing** (`basic_typing.rs`) - Simple text input with Unicode support
2. **Shortcuts** (`shortcuts.rs`) - Common keyboard shortcuts and combinations
3. **Stdin Processing** (`stdin_processing.rs`) - Reading and processing piped input
4. **Advanced Sequences** (`advanced_sequences.rs`) - Complex automation workflows

Run any example with:
```bash
cargo run --example basic_typing
# or using nix
nix develop -c cargo run --example basic_typing
```

## Compatibility

wrtype is compatible with the original C implementation of wtype and supports the same command-line interface and functionality.
