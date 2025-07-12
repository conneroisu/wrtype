#!/usr/bin/env bash
# Convenience script for running wrtype examples

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_color() {
    echo -e "${1}${2}${NC}"
}

# Function to run an example with description
run_example() {
    local name="$1"
    local description="$2"
    
    print_color "$BLUE" "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    print_color "$GREEN" "Running: $name"
    print_color "$YELLOW" "$description"
    print_color "$BLUE" "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    echo ""
    print_color "$YELLOW" "Press Enter to continue, or Ctrl+C to exit..."
    read -r
    
    if command -v nix >/dev/null 2>&1; then
        print_color "$BLUE" "Using Nix development environment..."
        nix develop -c cargo run --example "$name"
    else
        print_color "$BLUE" "Using system cargo..."
        cargo run --example "$name"
    fi
    
    echo ""
    print_color "$GREEN" "Example completed!"
    echo ""
}

# Function to show usage
show_usage() {
    cat << EOF
wrtype Examples Runner

Usage: $0 [OPTION]

Options:
    all                 Run all examples in sequence
    basic               Run basic text typing example
    shortcuts           Run keyboard shortcuts example  
    stdin               Run stdin processing example
    advanced            Run advanced sequences example
    build               Build all examples
    list                List available examples
    help                Show this help message

Examples:
    $0 all              # Run all examples
    $0 basic            # Run just the basic typing example
    $0 build            # Build all examples without running

Note: Examples require a running Wayland session with virtual-keyboard support.
EOF
}

# Function to list examples
list_examples() {
    print_color "$GREEN" "Available Examples:"
    echo ""
    print_color "$BLUE" "  basic_typing       - Simple text typing with Unicode support"
    print_color "$BLUE" "  shortcuts          - Keyboard shortcuts and key combinations"
    print_color "$BLUE" "  stdin_processing   - Processing piped input and stdin text"
    print_color "$BLUE" "  advanced_sequences - Complex automation workflows"
    echo ""
}

# Function to build all examples
build_examples() {
    print_color "$GREEN" "Building all examples..."
    
    if command -v nix >/dev/null 2>&1; then
        nix develop -c cargo build --examples
    else
        cargo build --examples
    fi
    
    print_color "$GREEN" "All examples built successfully!"
}

# Function to check Wayland environment
check_wayland() {
    if [[ -z "$WAYLAND_DISPLAY" ]]; then
        print_color "$RED" "Warning: WAYLAND_DISPLAY not set. Make sure you're running in a Wayland session."
        echo ""
    fi
}

# Main script logic
case "${1:-help}" in
    "all")
        check_wayland
        print_color "$GREEN" "Running all wrtype examples..."
        echo ""
        
        run_example "basic_typing" "Simple text typing with Unicode characters and delays"
        run_example "shortcuts" "Common keyboard shortcuts and key combinations"
        run_example "stdin_processing" "Processing text from stdin and pipes"
        run_example "advanced_sequences" "Complex automation workflows and key sequences"
        
        print_color "$GREEN" "All examples completed!"
        ;;
    
    "basic")
        check_wayland
        run_example "basic_typing" "Simple text typing with Unicode characters and delays"
        ;;
    
    "shortcuts")
        check_wayland
        run_example "shortcuts" "Common keyboard shortcuts and key combinations"
        ;;
    
    "stdin")
        check_wayland
        print_color "$YELLOW" "Note: You can also pipe input to this example:"
        print_color "$BLUE" "  echo 'Hello!' | $0 stdin"
        echo ""
        run_example "stdin_processing" "Processing text from stdin and pipes"
        ;;
    
    "advanced")
        check_wayland
        run_example "advanced_sequences" "Complex automation workflows and key sequences"
        ;;
    
    "build")
        build_examples
        ;;
    
    "list")
        list_examples
        ;;
    
    "help"|"-h"|"--help")
        show_usage
        ;;
    
    *)
        print_color "$RED" "Unknown option: $1"
        echo ""
        show_usage
        exit 1
        ;;
esac