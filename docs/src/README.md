# wrtype

**wrtype** is a modern Rust implementation of [wtype](https://github.com/atx/wtype), providing `xdotool type` functionality for Wayland compositors. It enables programmatic text input and keyboard simulation through Wayland's virtual keyboard protocol.

## What is wrtype?

wrtype allows you to:
- **Type text** programmatically into any Wayland application
- **Simulate key presses** including modifier keys (Ctrl, Alt, Shift, etc.)
- **Control timing** with configurable delays between keystrokes
- **Process stdin** for piped text input
- **Handle Unicode** characters seamlessly

## Key Features

### 🚀 **Performance & Reliability**
- Written in Rust for memory safety and performance
- Robust error handling and graceful failure modes
- Efficient protocol communication with minimal overhead

### 🌍 **Unicode First**
- Full Unicode support for international text
- Dynamic keymap generation for any character
- Proper UTF-8 handling from stdin

### 🔧 **Developer Friendly**
- Drop-in replacement for wtype with identical CLI
- Comprehensive error messages
- Detailed logging and debugging support

### 📦 **Easy Installation**
- Available through Nix for reproducible builds
- Standalone binary with minimal dependencies
- Cross-platform Wayland support

## Quick Example

```bash
# Type some text
wrtype "Hello, World!"

# Type with delay between characters
wrtype -d 100 "Slow typing..."

# Simulate Ctrl+C
wrtype -M ctrl c -m ctrl

# Pipe text from command
echo "Dynamic content" | wrtype --stdin

# Press and hold a key
wrtype -P space -s 1000 -p space
```

## Why wrtype?

While the original wtype is excellent, wrtype offers several advantages:

- **Memory Safety**: Rust prevents common C pitfalls
- **Better Error Handling**: Clear error messages and recovery
- **Modern Codebase**: Clean, well-documented, maintainable code
- **Enhanced Unicode**: Robust support for complex text
- **Nix Integration**: First-class support for Nix workflows

## Use Cases

- **Test Automation**: Simulate user input in GUI tests
- **Accessibility Tools**: Assistive input for users with disabilities  
- **Gaming & Streaming**: Automated text input for games and broadcasts
- **Development Workflows**: Script complex keyboard sequences
- **System Administration**: Automate repetitive text entry tasks

## Getting Started

Ready to use wrtype? Check out the [Installation](installation.md) guide and [Quick Start](quick-start.md) tutorial to begin using wrtype in your workflows.

For developers interested in contributing or understanding the internals, see the [Architecture Overview](architecture.md) and [Building from Source](building.md) sections.