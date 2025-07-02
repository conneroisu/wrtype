# wrtype development commands

# Default recipe lists all available commands
default:
    @just --list

# Build the project
build:
    cargo build

# Build with release optimizations
build-release:
    cargo build --release

# Build all examples
build-examples:
    cargo build --examples

# Run all tests
test:
    cargo test

# Check code without building
check:
    cargo check

# Run linting
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Run a specific example
run-example name:
    cargo run --example {{name}}

# Run basic typing example
example-basic:
    cargo run --example basic_typing

# Run shortcuts example  
example-shortcuts:
    cargo run --example shortcuts

# Run stdin processing example
example-stdin:
    cargo run --example stdin_processing

# Run advanced sequences example
example-advanced:
    cargo run --example advanced_sequences

# Run all examples interactively
examples-all:
    ./run-examples.sh all

# Build and run examples using nix development environment
nix-example name:
    nix develop -c cargo run --example {{name}}

# Build using nix
nix-build:
    nix develop -c cargo build

# Run tests using nix
nix-test:
    nix develop -c cargo test

# Format using nix treefmt
nix-fmt:
    nix fmt

# Clean build artifacts
clean:
    cargo clean

# Generate documentation
docs:
    cargo doc --open

# Generate documentation for library users (no private items)
docs-lib:
    cargo doc --lib --no-deps --open

# Check if examples compile
check-examples:
    cargo check --examples

# Quick development cycle: format, lint, test, build
dev: fmt lint test build

# Nix development cycle
nix-dev:
    nix develop -c just dev

# Show example usage help
examples-help:
    ./run-examples.sh help

# Build examples and show how to run them
examples-setup: build-examples
    @echo "Examples built! Run them with:"
    @echo "  just example-basic     # Basic text typing"
    @echo "  just example-shortcuts # Keyboard shortcuts" 
    @echo "  just example-stdin     # Stdin processing"
    @echo "  just example-advanced  # Advanced sequences"
    @echo "  just examples-all      # Run all interactively"
    @echo ""
    @echo "Or use the script directly:"
    @echo "  ./run-examples.sh [basic|shortcuts|stdin|advanced|all]"

# Install the binary locally
install:
    cargo install --path .

# Show project information
info:
    @echo "Project: wrtype"
    @echo "Description: Rust implementation of wtype for Wayland"
    @echo ""
    @echo "Available examples:"
    @echo "  basic_typing       - Simple text input"
    @echo "  shortcuts          - Keyboard shortcuts"
    @echo "  stdin_processing   - Pipe and stdin handling"
    @echo "  advanced_sequences - Complex automation"
    @echo ""
    @echo "Requirements:"
    @echo "  - Wayland compositor with virtual-keyboard support"
    @echo "  - WAYLAND_DISPLAY environment variable set"

# Quick start for new users
quick-start: build-examples
    @echo "🚀 wrtype Quick Start"
    @echo ""
    @echo "1. Make sure you're in a Wayland session:"
    @echo "   echo \$WAYLAND_DISPLAY"
    @echo ""
    @echo "2. Try the basic example:"
    @echo "   just example-basic"
    @echo ""
    @echo "3. Or run all examples interactively:"
    @echo "   just examples-all"
    @echo ""
    @echo "4. Use as a library in your project:"
    @echo "   cargo add wrtype --git https://github.com/conneroisu/wrtype"