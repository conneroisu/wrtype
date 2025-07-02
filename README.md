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

## Compatibility

wrtype is compatible with the original C implementation of wtype and supports the same command-line interface and functionality.
