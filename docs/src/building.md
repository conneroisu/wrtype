# Building from Source

This guide covers building wrtype from source code for development, customization, or contributing to the project.

## Prerequisites

### System Requirements

- **Rust** 1.70.0 or later
- **Git** for source control
- **pkg-config** for dependency discovery
- **Wayland development libraries**

### Rust Installation

Install Rust through rustup (recommended):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update
```

### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt install \
    git \
    pkg-config \
    libwayland-dev \
    wayland-protocols \
    libxkbcommon-dev \
    build-essential
```

**Fedora/CentOS:**
```bash
sudo dnf install \
    git \
    pkgconf-pkg-config \
    wayland-devel \
    wayland-protocols-devel \
    libxkbcommon-devel \
    gcc
```

**Arch Linux:**
```bash
sudo pacman -S \
    git \
    pkgconf \
    wayland \
    wayland-protocols \
    libxkbcommon \
    base-devel
```

## Getting the Source

Clone the repository:

```bash
git clone https://github.com/conneroisu/wrtype.git
cd wrtype
```

### Repository Structure

```
wrtype/
├── src/           # Rust source code
│   ├── main.rs    # CLI and orchestration
│   ├── wayland.rs # Wayland protocol implementation
│   ├── keymap.rs  # XKB keymap generation
│   └── executor.rs# Command execution engine
├── docs/          # Documentation source
├── wtype/         # Original C implementation (reference)
├── Cargo.toml     # Rust dependencies and metadata
├── flake.nix      # Nix development environment
└── README.md      # Project overview
```

## Building with Cargo

### Development Build

```bash
# Build debug version (faster compilation, slower runtime)
cargo build

# Run directly without installing
cargo run -- "Hello from development build!"
```

### Release Build

```bash
# Build optimized release version
cargo build --release

# Binary will be at target/release/wrtype
./target/release/wrtype --version
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Generate documentation
cargo doc --open
```

## Building with Nix (Recommended)

### Development Environment

```bash
# Enter development shell with all dependencies
nix develop

# Or with direnv (if configured)
direnv allow
```

### Available Commands in Nix Shell

```bash
# Build the project
cargo build --release

# Run with hot reloading
cargo watch -x run

# Format all code
cargo fmt

# Check for issues
cargo clippy
```

### Building Documentation

```bash
# Build documentation site
nix build .#docs

# View built documentation
firefox result/index.html
```

## Development Workflow

### 1. Setup Development Environment

```bash
git clone https://github.com/conneroisu/wrtype.git
cd wrtype
nix develop  # or install dependencies manually
```

### 2. Make Changes

```bash
# Edit source files
$EDITOR src/main.rs

# Check compilation
cargo check
```

### 3. Test Changes

```bash
# Run tests
cargo test

# Test manually
cargo run -- "test input"

# Test with different options
cargo run -- -d 100 "slow typing"
```

### 4. Quality Checks

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy

# Run full test suite
cargo test --release
```

### 5. Documentation

```bash
# Generate and view docs
cargo doc --open

# Build documentation site
cd docs && mdbook build
```

## Custom Builds

### Feature Flags

Currently wrtype doesn't use feature flags, but they can be added for optional functionality:

```toml
# In Cargo.toml
[features]
default = []
debug-protocol = []  # Enable protocol debugging
experimental = []    # Enable experimental features
```

### Cross Compilation

Build for different architectures:

```bash
# Install target
rustup target add aarch64-unknown-linux-gnu

# Cross compile (requires cross-compilation toolchain)
cargo build --target aarch64-unknown-linux-gnu --release
```

### Optimization Profiles

Customize build profiles in `Cargo.toml`:

```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
panic = "abort"      # Smaller binary
strip = true         # Remove debug symbols
```

## Debugging Builds

### Debug Information

```bash
# Build with debug info
cargo build

# Run with debugger
gdb target/debug/wrtype
rust-gdb target/debug/wrtype
```

### Environment Variables

```bash
# Enable Rust backtraces
RUST_BACKTRACE=1 cargo run -- "test"

# Enable full backtraces
RUST_BACKTRACE=full cargo run -- "test"

# Enable logging (if implemented)
RUST_LOG=debug cargo run -- "test"
```

### Wayland Debugging

```bash
# Enable Wayland debugging
WAYLAND_DEBUG=1 cargo run -- "test"

# Check available protocols
wayland-info | grep virtual_keyboard
```

## Common Build Issues

### Missing Dependencies

**Error:** `pkg-config` not found
```bash
# Install pkg-config
sudo apt install pkg-config  # Ubuntu/Debian
sudo dnf install pkgconf     # Fedora
```

**Error:** Wayland libraries not found
```bash
# Install Wayland development packages
sudo apt install libwayland-dev wayland-protocols
```

### Rust Version Issues

**Error:** Minimum supported Rust version
```bash
# Update Rust
rustup update

# Check version
rustc --version
```

### Linker Issues

**Error:** Cannot find linker
```bash
# Install build tools
sudo apt install build-essential  # Ubuntu/Debian
sudo dnf install gcc             # Fedora
```

## Performance Builds

### Maximum Optimization

```bash
# Use highest optimization level
RUSTFLAGS="-C target-cpu=native" cargo build --release

# Profile-guided optimization (advanced)
cargo build --release
# Run with representative workload to generate profile data
RUSTFLAGS="-C profile-use=profile.profdata" cargo build --release
```

### Size Optimization

```bash
# Optimize for size
RUSTFLAGS="-C opt-level=z" cargo build --release

# Use system allocator (smaller binary)
# Add to main.rs:
# #[global_allocator]
# static ALLOC: std::alloc::System = std::alloc::System;
```

## Contributing Builds

### Before Submitting

```bash
# Run full test suite
cargo test --release

# Check formatting
cargo fmt --check

# Check for clippy warnings
cargo clippy -- -D warnings

# Verify documentation builds
cargo doc --no-deps

# Test on multiple Wayland compositors if possible
```

### Continuous Integration

The project uses GitHub Actions for CI. Local verification:

```bash
# Run CI-equivalent checks
cargo check --all-targets
cargo test --all
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Packaging

### Creating Release Binary

```bash
# Build optimized release
cargo build --release

# Strip debug symbols
strip target/release/wrtype

# Verify binary
ldd target/release/wrtype
file target/release/wrtype
```

### Debian Package

```bash
# Install cargo-deb
cargo install cargo-deb

# Create .deb package
cargo deb
```

### RPM Package

```bash
# Install cargo-rpm
cargo install cargo-rpm

# Create .rpm package
cargo rpm build
```

This comprehensive build guide should help you successfully build wrtype from source for any purpose, whether development, customization, or distribution.