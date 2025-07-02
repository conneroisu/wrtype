# Examples

Real-world examples demonstrating wrtype's capabilities across different scenarios.

## Basic Examples

### Simple Text Entry

```bash
# Type a URL
wrtype "https://example.com"

# Type an email address
wrtype "user@domain.com"

# Type with special characters
wrtype "Price: $29.99 (20% off!)"
```

### Unicode and International Text

```bash
# Mathematical symbols
wrtype "∫₀^∞ e^(-x²) dx = √π/2"

# International characters
wrtype "Café naïve résumé"

# Multiple scripts
wrtype "Hello नमस्ते こんにちは 안녕하세요"

# Emoji
wrtype "Great work! 🎉 Keep it up! 💪"
```

## Text Editor Automation

### Code Writing

```bash
# Write a Rust function
wrtype 'fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}'

# Write HTML
wrtype '<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
</head>
<body>
    <h1>Hello, World!</h1>
</body>
</html>'
```

### Text Manipulation

```bash
# Select all and replace
wrtype -M ctrl a -m ctrl
wrtype -s 100 "New content replacing everything"

# Go to line start and add comment
wrtype -k Home "// " -k End

# Duplicate current line
wrtype -M ctrl c -m ctrl -k End -k Return -M ctrl v -m ctrl
```

### Find and Replace

```bash
# Open find dialog and search
wrtype -M ctrl f -m ctrl
wrtype -s 200 "old_function_name"
wrtype -k Escape

# Open replace dialog
wrtype -M ctrl h -m ctrl
wrtype "old_function_name" -k Tab "new_function_name"
wrtype -M alt a -m alt  # Replace All
```

## Terminal and Shell Automation

### Command Execution

```bash
# Basic commands
wrtype "ls -la" -k Return
wrtype "cd /home/user/projects" -k Return
wrtype "git status" -k Return

# Pipe commands
wrtype "ps aux | grep firefox" -k Return
wrtype "find . -name '*.rs' | wc -l" -k Return
```

### Interactive Commands

```bash
# SSH with password (be careful with this!)
wrtype "ssh user@server.com" -k Return
# Wait for password prompt...
wrtype -s 2000 "password123" -k Return

# Git operations
wrtype "git add ." -k Return
wrtype "git commit -m 'Update documentation'" -k Return
wrtype "git push origin main" -k Return
```

### Scripting and Automation

```bash
# Create and run a quick script
wrtype "echo '#!/bin/bash' > script.sh" -k Return
wrtype "echo 'echo Hello from script' >> script.sh" -k Return
wrtype "chmod +x script.sh" -k Return
wrtype "./script.sh" -k Return
```

## Web Browser Automation

### Navigation

```bash
# Open new tab
wrtype -M ctrl t -m ctrl

# Go to address bar
wrtype -M ctrl l -m ctrl
wrtype "github.com/conneroisu/wrtype" -k Return

# Search on page
wrtype -M ctrl f -m ctrl
wrtype "virtual keyboard" -k Return
```

### Form Filling

```bash
# Fill login form
wrtype "username@example.com"
wrtype -k Tab  # Move to password field
wrtype "secure_password_123"
wrtype -k Return  # Submit

# Fill contact form
wrtype "John Doe" -k Tab
wrtype "john@example.com" -k Tab  
wrtype "Subject: Important Message" -k Tab
wrtype "This is the message body with
multiple lines of content." -k Tab
wrtype -k Return  # Submit
```

## Desktop Environment Integration

### Application Launching

```bash
# Open application launcher (GNOME/KDE)
wrtype -M alt -k F2 -m alt
wrtype -s 300 "firefox" -k Return

# Or use Super key
wrtype -k Super_L
wrtype -s 200 "terminal" -k Return
```

### Window Management

```bash
# Switch windows
wrtype -M alt -k Tab -m alt

# Move between workspaces
wrtype -M ctrl -M alt -k Right -m alt -m ctrl

# Minimize window
wrtype -M alt -k F9 -m alt

# Close window
wrtype -M alt -k F4 -m alt
```

### System Operations

```bash
# Take screenshot
wrtype -k Print

# Open system settings
wrtype -M ctrl -M alt -k Delete -m alt -m ctrl

# Lock screen
wrtype -M ctrl -M alt -k l -m alt -m ctrl
```

## Development Workflows

### IDE/Editor Operations

```bash
# Open file in VS Code
wrtype -M ctrl o -m ctrl
wrtype -s 300 "src/main.rs" -k Return

# Format code
wrtype -M ctrl -M shift i -m shift -m ctrl

# Open terminal in IDE
wrtype -M ctrl -k grave -m ctrl  # Ctrl+`

# Run program
wrtype -k F5
```

### Git Workflows

```bash
# Interactive git add
wrtype "git add -p" -k Return
# For each hunk:
wrtype "y" -k Return  # Accept
# or
wrtype "n" -k Return  # Skip
# or  
wrtype "s" -k Return  # Split

# Interactive rebase
wrtype "git rebase -i HEAD~3" -k Return
# Edit commits in editor...
wrtype ":wq" -k Return  # Save and exit vim
```

## System Administration

### Package Management

```bash
# Update system (Debian/Ubuntu)
wrtype "sudo apt update && sudo apt upgrade" -k Return
# Enter password when prompted
wrtype -s 2000 "password" -k Return

# Install package
wrtype "sudo apt install htop" -k Return
wrtype "y" -k Return  # Confirm installation
```

### Log Analysis

```bash
# View system logs
wrtype "sudo journalctl -f" -k Return

# Search logs
wrtype "sudo journalctl | grep error" -k Return

# Check disk usage
wrtype "df -h" -k Return
wrtype "du -sh /*" -k Return
```

## Data Processing

### Working with Files

```bash
# Process CSV data
wrtype "cut -d, -f1,3 data.csv | head -10" -k Return

# Search and count
wrtype "grep -r 'TODO' src/ | wc -l" -k Return

# File manipulation
wrtype "find . -name '*.tmp' -delete" -k Return
```

### Text Processing with Pipes

```bash
# Complex pipeline
cat data.txt | wrtype --stdin -d 10
# This types the file content slowly

# Process and type result
echo "ls -la | grep .rs | awk '{print \$9}'" | bash | wrtype --stdin
```

## Testing and QA

### Automated Testing

```bash
# Fill test form
wrtype "test@example.com" -k Tab
wrtype "Test User" -k Tab  
wrtype "123 Test Street" -k Tab
wrtype "Test City" -k Tab
wrtype -k Return  # Submit

# Verify success message appears
# (you'd check this visually or with other tools)
```

### Load Testing

```bash
# Rapid form submission (be careful!)
for i in {1..10}; do
    wrtype "user$i@test.com" -k Tab
    wrtype "password123" -k Tab
    wrtype -k Return
    sleep 1
done
```

## Creative and Entertainment

### Gaming Automation

```bash
# Chat messages
wrtype -k Enter "GG everyone!" -k Return
wrtype -k Enter "/dance" -k Return

# Repeated actions (be mindful of game rules)
wrtype -k space -s 100 -k space -s 100 -k space
```

### Streaming and Content Creation

```bash
# Type chat responses
wrtype "Thanks for following! Welcome to the stream!"

# Add timestamps to content
wrtype "$(date): Starting new section" -k Return
```

## Accessibility and Assistive Technology

### Voice-to-Text Integration

```bash
# Simulate voice command result
# (This would typically be triggered by voice recognition software)
wrtype "Open calculator application"
wrtype -k Return
```

### Macro Replacement

```bash
# Expand abbreviations
# When user types "addr", expand to full address
wrtype -M ctrl a -m ctrl  # Select "addr"
wrtype "123 Main Street, Anytown, ST 12345"
```

## Advanced Patterns

### Conditional Text Entry

```bash
# Check if file exists, then type result
if [ -f "config.txt" ]; then
    wrtype "Config file found" -k Return
else
    wrtype "Config file missing" -k Return
fi
```

### Dynamic Content Generation

```bash
# Type current date and time
wrtype "Report generated on: $(date '+%Y-%m-%d %H:%M:%S')" -k Return

# Type system information
wrtype "System: $(uname -a)" -k Return
wrtype "User: $(whoami)" -k Return
wrtype "Directory: $(pwd)" -k Return
```

### Error Recovery

```bash
# Safe automation with error handling
if wrtype -M ctrl c -m ctrl 2>/dev/null; then
    echo "Successfully sent Ctrl+C"
else
    echo "Failed to send keyboard input"
    # Handle error...
fi
```

## Tips for Effective Usage

1. **Test in Safe Environment**: Always test automation scripts in non-production environments first.

2. **Add Appropriate Delays**: GUI applications need time to process input - use `-s` for timing.

3. **Handle Focus**: Ensure the target application has keyboard focus before sending input.

4. **Error Handling**: Wrap critical operations in error checking.

5. **Documentation**: Comment your automation scripts for future maintenance.

6. **Security**: Be careful with passwords and sensitive data in scripts.

These examples demonstrate wrtype's versatility across different domains. Adapt them to your specific needs and always consider the security and ethical implications of automation.