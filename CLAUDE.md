# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

wrtype is a Rust implementation of wtype - a Wayland virtual keyboard tool that types text and sends key events to applications. It's designed as a cross-platform replacement for xdotool on Wayland compositors.

## Key Commands

### Development and Building

```bash
# Using Nix (Recommended)
nix develop                     # Enter development environment
nix develop -c cargo build --release  # Build optimized release version
nix develop -c cargo run -- "Hello World"  # Run with text input

# Using Cargo directly (requires system dependencies)
nix develop -c cargo build                # Build debug version
nix develop -c cargo build --release      # Build release version
nix develop -c cargo test                 # Run all tests
nix develop -c cargo check                # Check code without building
nix develop -c cargo clippy               # Run linter
nix develop -c cargo fmt                  # Format code
```

### Testing and Quality

```bash
# Run tests
nix develop -c cargo test                 # All tests
nix develop -c cargo test -- --nocapture # Tests with output
nix develop -c cargo test test_name      # Specific test

# Code quality
nix develop -c cargo fmt --check         # Check formatting
nix develop -c cargo clippy -- -D warnings  # Linting with warnings as errors
nix develop -c env RUST_BACKTRACE=1 cargo run  # Run with stack traces
```

### Formatting

```bash
# Format all code using treefmt
nix fmt                         # Format Nix and Rust files
nix develop -c cargo fmt        # Format Rust code only
nix develop -c alejandra .      # Format Nix files only
```

### Package Building

```bash
# Build the package using Nix
nix build                       # Build wrtype package
nix build .#wrtype             # Build specific package
nix run                        # Build and run wrtype
nix run . -- "Hello World"     # Run with arguments
```

### Usage Examples

```bash
# Type text
nix develop -c cargo run -- "Hello World"

# Press Ctrl+C
nix develop -c cargo run -- -M ctrl c -m ctrl

# Type with delays
nix develop -c cargo run -- foo -d 120 bar

# Read from stdin
echo "text" | nix develop -c cargo run -- --stdin -d 12
```

## Architecture Overview

### Core Module Structure

The application follows a modular pipeline architecture:

1. **`main.rs`** - CLI parsing, command sequencing, and application lifecycle
2. **`wayland.rs`** - Wayland protocol implementation and connection management  
3. **`keymap.rs`** - Dynamic XKB keymap generation for Unicode support
4. **`executor.rs`** - Sequential command execution with timing control

### Data Flow Pipeline

```
CLI Input → Command Parser → Command Executor → Keymap Builder → Wayland Protocol → Compositor
```

### Key Design Principles

- **Dynamic Keymap Generation**: On-demand XKB keymaps support arbitrary Unicode characters
- **Protocol-First Design**: Direct Wayland virtual keyboard protocol implementation
- **Synchronous Execution**: Predictable command ordering with explicit timing control
- **UTF-8 First**: Full Unicode support throughout the pipeline

## Important Implementation Details

### Command System

The `Command` enum represents all possible actions:
- `Text` - Type strings with configurable delays
- `ModPress/ModRelease` - Modifier key control
- `KeyPress/KeyRelease` - Named key control  
- `Sleep` - Timing control
- `StdinText` - Read from stdin

### Wayland Integration

The `WaylandState` manages:
- Virtual keyboard protocol binding
- Seat discovery and management
- Modifier state tracking
- Protocol object lifecycle

### Keymap Management

The `KeymapBuilder` provides:
- Dynamic character-to-keysym mapping
- Efficient keycode allocation and caching
- XKB-compliant keymap generation
- Unicode character support

### Build System

- **Protocol Generation**: `build.rs` handles Wayland protocol code generation
- **Nix Integration**: `flake.nix` provides reproducible development environment
- **Cross-Platform**: Supports Linux distributions with different package managers

## Development Environment

### Nix Development Shell

The flake provides a comprehensive development environment with:

**Core Tools:**
- Rust stable toolchain via rust-overlay
- Cargo and standard Rust development tools

**Wayland Dependencies:**
- `wayland` - Core Wayland libraries
- `wayland-protocols` - Protocol definitions
- `libxkbcommon` - XKB keymap support
- `pkg-config` - Build configuration

**Development Tools:**
- `alejandra` - Nix code formatter
- `nixd` - Nix language server
- `statix` - Nix linter
- `deadnix` - Dead code detector for Nix
- `just` - Command runner

**Code Quality:**
- `rustfmt` - Rust code formatter (via treefmt)
- `cargo clippy` - Rust linter
- Integrated formatting pipeline

### Testing Strategy

- Unit tests for individual components
- Integration tests for protocol interaction
- Manual testing with different Wayland compositors
- Unicode and timing edge case validation

### Protocol Dependencies

- Requires Wayland compositor with `zwp_virtual_keyboard_manager_v1` support
- Uses standard XKB keymap format for character mapping
- Implements proper modifier state management

### Common Development Tasks

- **Adding new commands**: Extend the `Command` enum and update parser/executor
- **Protocol debugging**: Use `WAYLAND_DEBUG=1` environment variable
- **Keymap issues**: Check XKB keysym mappings in `keymap.rs`
- **Timing problems**: Adjust delays in command execution logic

### Performance Considerations

- Keymap lookup is O(1) for cached characters
- Memory usage grows linearly with unique characters used
- Protocol overhead is minimal per-event
- Large stdin inputs could benefit from streaming optimizations

## System Dependencies (if not using Nix)

If you need to develop outside the Nix environment:

**Ubuntu/Debian:**
```bash
sudo apt install libwayland-dev wayland-protocols libxkbcommon-dev pkg-config build-essential
```

**Fedora:**
```bash
sudo dnf install wayland-devel wayland-protocols-devel libxkbcommon-devel pkgconf-pkg-config gcc
```