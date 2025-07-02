# Quick Start

This guide will get you up and running with wrtype in minutes. We'll cover the most common use cases and basic functionality.

## Basic Text Typing

The simplest use of wrtype is typing text:

```bash
# Type a simple message
wrtype "Hello, World!"
```

```cmdrun
echo 'wrtype "Hello from the docs!"'
```

### Special Characters

wrtype handles Unicode characters seamlessly:

```bash
# Unicode characters
wrtype "∇⋅∇ψ = ρ"

# Emoji
wrtype "🚀 Rust is awesome! 🦀"

# Mixed scripts
wrtype "English 中文 العربية русский"
```

### Escaping Special Characters

Use quotes or double-dashes to handle special characters:

```bash
# Text starting with dash
wrtype -- "-this-starts-with-dash"

# Text with quotes
wrtype 'He said "Hello!"'
```

## Key Combinations

### Modifier Keys

Press and release modifier keys:

```bash
# Ctrl+C (copy)
wrtype -M ctrl c -m ctrl

# Ctrl+Shift+T (new terminal tab)
wrtype -M ctrl -M shift t -m shift -m ctrl

# Alt+Tab (window switcher)
wrtype -M alt Tab -m alt
```

### Named Keys

Use XKB key names for special keys:

```bash
# Arrow keys
wrtype -k Left -k Left -k Right

# Function keys  
wrtype -k F5

# Other special keys
wrtype -k Return    # Enter
wrtype -k space     # Space
wrtype -k Escape    # Esc
wrtype -k Tab       # Tab
wrtype -k BackSpace # Backspace
```

## Timing Control

### Typing Speed

Control the delay between keystrokes:

```bash
# Fast typing (default)
wrtype "Quick text"

# Slow typing (100ms between characters)
wrtype -d 100 "Slow typing..."

# Very slow typing (500ms between characters)  
wrtype -d 500 "Very slow typing..."
```

### Precise Timing

Use sleep commands for precise timing:

```bash
# Hold key for specific duration
wrtype -P space -s 1000 -p space  # Hold space for 1 second

# Complex sequence with timing
wrtype -M ctrl a -s 100 -m ctrl -s 200 "Replace all text"
```

## Input from Stdin

Process text from other commands:

```bash
# Pipe command output
echo "Dynamic content" | wrtype --stdin

# Process file contents
cat myfile.txt | wrtype --stdin -d 50

# Chain commands
curl -s https://api.example.com/text | wrtype --stdin
```

### Mixed Input

Combine stdin with other commands:

```bash
# Type prefix, then stdin, then suffix
wrtype "Prefix: " - " (end)" < input.txt

# With timing
echo "Piped text" | wrtype -d 100 "Start: " --stdin " :End"
```

## Real-World Examples

### Code Snippets

Type code with proper indentation:

```bash
wrtype 'fn main() {
    println!("Hello, World!");
}'
```

### Form Filling

Automate form filling:

```bash
# Fill email field
wrtype "user@example.com"
wrtype -k Tab  # Move to next field

# Fill password  
wrtype "secure-password"
wrtype -k Return  # Submit
```

### Text Editor Automation

Automate text editor tasks:

```bash
# Select all and replace
wrtype -M ctrl a -m ctrl -s 100 "New content"

# Navigate and edit
wrtype -M ctrl -k Home -m ctrl  # Go to start
wrtype "// Added comment" -k Return
```

### System Navigation

Navigate desktop environments:

```bash
# Open application launcher
wrtype -M alt -k F2 -m alt
wrtype -s 200 "firefox" -k Return

# Switch workspace
wrtype -M ctrl -M alt -k Right -m alt -m ctrl
```

## Common Patterns

### Search and Replace

```bash
# Open find dialog and search
wrtype -M ctrl f -m ctrl
wrtype -s 100 "old text" -k Return
wrtype -k Escape  # Close find

# Open replace dialog
wrtype -M ctrl h -m ctrl  
wrtype "old text" -k Tab "new text"
wrtype -k Return  # Replace all
```

### File Operations

```bash
# Save file
wrtype -M ctrl s -m ctrl

# Open file dialog
wrtype -M ctrl o -m ctrl
wrtype -s 500 "filename.txt" -k Return
```

### Terminal Commands

```bash
# Type and execute command
wrtype "ls -la" -k Return

# Chain commands
wrtype "cd /tmp && ls" -k Return
```

## Pro Tips

### 1. **Test First**
Always test your wrtype commands in a safe environment before using them in production.

### 2. **Use Quotes**
Wrap text in quotes to avoid shell interpretation of special characters.

### 3. **Add Delays**
Use small delays (`-d 50`) when automating GUI applications to ensure they can keep up.

### 4. **Check Focus**
Ensure the target application has keyboard focus before running wrtype.

### 5. **Escape Hatch**
Have a way to regain control (like switching to another workspace) if something goes wrong.

## Next Steps

- Explore the complete [Command Reference](commands.md)
- See more [Examples](examples.md) for advanced use cases
- Learn about the [Architecture](architecture.md) if you want to contribute
- Check [Troubleshooting](troubleshooting.md) if you encounter issues