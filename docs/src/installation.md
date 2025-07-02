# Installation

wrtype can be installed through several methods depending on your system and preferences.

## Nix/NixOS (Recommended)

The easiest way to install wrtype is through Nix, which provides reproducible builds and automatic dependency management.

### Using Nix Flakes

```bash
# Install directly from GitHub
nix profile install github:conneroisu/wrtype

# Or add to your flake.nix
{
  inputs.wrtype.url = "github:conneroisu/wrtype";
  # ... in your system packages
  environment.systemPackages = [ inputs.wrtype.packages.${system}.default ];
}
```

### Using nix-env

```bash
# Install from the repository
nix-env -if https://github.com/conneroisu/wrtype/archive/main.tar.gz
```

### Development Shell

```bash
# Enter development environment
git clone https://github.com/conneroisu/wrtype
cd wrtype
nix develop
```

## From Source (Cargo)

If you have Rust and the required system dependencies installed:

### Prerequisites

**System Dependencies:**
- `libxkbcommon-dev` - XKB keyboard handling
- `wayland-dev` - Wayland client libraries  
- `wayland-protocols` - Wayland protocol definitions
- `pkg-config` - Build system integration

**Ubuntu/Debian:**
```bash
sudo apt install libxkbcommon-dev libwayland-dev wayland-protocols pkg-config
```

**Fedora/CentOS:**
```bash
sudo dnf install libxkbcommon-devel wayland-devel wayland-protocols-devel pkgconfig
```

**Arch Linux:**
```bash
sudo pacman -S libxkbcommon wayland wayland-protocols pkgconf
```

### Build and Install

```bash
# Clone the repository
git clone https://github.com/conneroisu/wrtype
cd wrtype

# Build release version
cargo build --release

# Install to ~/.cargo/bin
cargo install --path .

# Or copy binary manually
sudo cp target/release/wrtype /usr/local/bin/
```

## Pre-built Binaries

Pre-built binaries will be available on the GitHub releases page:

```bash
# Download latest release (example)
curl -L https://github.com/conneroisu/wrtype/releases/latest/download/wrtype-x86_64-linux -o wrtype
chmod +x wrtype
sudo mv wrtype /usr/local/bin/
```

## Verification

Test your installation:

```bash
# Check version
wrtype --version

# Test basic functionality
wrtype "Installation successful!"
```

## System Requirements

- **Operating System**: Linux with Wayland support
- **Wayland Compositor**: Any compositor supporting `zwp_virtual_keyboard_manager_v1`
- **Architecture**: x86_64, aarch64 (others may work but untested)

### Supported Compositors

wrtype works with any Wayland compositor that implements the virtual keyboard protocol:

- ✅ **Sway** - Full support
- ✅ **Hyprland** - Full support  
- ✅ **GNOME (Mutter)** - Full support
- ✅ **KDE (KWin)** - Full support
- ✅ **Weston** - Full support
- ✅ **River** - Full support

### Known Limitations

- **X11**: Does not work on X11 (use `xdotool` instead)
- **XWayland**: May have limited functionality with XWayland applications
- **Permissions**: Some compositors may require specific permissions

## Troubleshooting

### Permission Denied

```bash
# Ensure wrtype has execute permissions
chmod +x wrtype
```

### Library Not Found

```bash
# Check required libraries are installed
ldd wrtype

# Install missing dependencies
# (See system-specific commands above)
```

### Protocol Not Supported

```bash
# Verify your compositor supports virtual keyboard
wayland-info | grep virtual_keyboard
```

If you encounter issues, see the [Troubleshooting](troubleshooting.md) section for detailed solutions.