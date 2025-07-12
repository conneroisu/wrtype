# Contributing

We welcome contributions to wrtype! This guide will help you get started with contributing code, documentation, bug reports, and feature requests.

## Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/yourusername/wrtype.git
cd wrtype
```

### 2. Set Up Development Environment

```bash
# Using Nix (recommended)
nix develop

# Or install dependencies manually (see Building from Source)
```

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-description
```

## Types of Contributions

### Bug Reports

When reporting bugs, please include:

- **Operating System** and version
- **Wayland Compositor** (Sway, GNOME, KDE, etc.)
- **wrtype version** (`wrtype --version`)
- **Steps to reproduce** the issue
- **Expected behavior** vs actual behavior
- **Error messages** (if any)

Use the bug report template:

```markdown
**Environment:**
- OS: Ubuntu 22.04
- Compositor: Sway 1.8
- wrtype version: 0.1.0

**Steps to Reproduce:**
1. Run `wrtype "test"`
2. Observe behavior

**Expected:** Should type "test"
**Actual:** Nothing happens

**Error Output:**
```
Error: Failed to connect to Wayland display
```
```

### Feature Requests

For new features, please:

- Check existing issues first
- Describe the use case clearly
- Explain why it would be beneficial
- Consider implementation complexity
- Provide examples of usage

### Documentation

Documentation improvements are always welcome:

- Fix typos or unclear explanations
- Add examples for complex use cases
- Improve API documentation
- Translate to other languages
- Update outdated information

### Code Contributions

## Development Guidelines

### Code Style

We follow standard Rust conventions:

```rust
// Use descriptive names
fn execute_keyboard_command() { }

// Document public APIs
/// Executes a sequence of keyboard commands.
/// 
/// # Arguments
/// * `commands` - Vector of commands to execute
/// 
/// # Returns
/// * `Ok(())` on success, error on failure
pub fn execute_commands(&mut self, commands: Vec<Command>) -> Result<()> {
    // Implementation
}

// Use Result for error handling
fn risky_operation() -> Result<String, MyError> {
    // Implementation
}
```

### Testing

All code should include appropriate tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifier_parsing() {
        assert_eq!(Modifier::from_name("ctrl"), Some(Modifier::Ctrl));
        assert_eq!(Modifier::from_name("invalid"), None);
    }

    #[test]
    fn test_keymap_generation() {
        let mut builder = KeymapBuilder::new();
        let keycode = builder.get_keycode_for_char('a');
        assert_eq!(keycode, 1);
    }
}
```

### Error Handling

Use `anyhow` for error handling:

```rust
use anyhow::{Context, Result};

fn connect_wayland() -> Result<Connection> {
    Connection::connect_to_env()
        .context("Failed to connect to Wayland display")
}
```

### Performance

Consider performance implications:

- Use efficient data structures
- Minimize allocations in hot paths
- Cache expensive computations
- Profile before optimizing

## Contribution Process

### 1. Development

```bash
# Make your changes
$EDITOR src/main.rs

# Test your changes
cargo test
cargo run -- "test input"

# Check code quality
cargo fmt
cargo clippy
```

### 2. Documentation

```bash
# Update documentation if needed
$EDITOR docs/src/relevant-section.md

# Generate API docs
cargo doc

# Test documentation builds
cd docs && mdbook build
```

### 3. Commit Guidelines

Use conventional commit format:

```bash
git commit -m "feat: add support for custom key delays"
git commit -m "fix: handle Unicode characters correctly"
git commit -m "docs: update installation instructions"
git commit -m "test: add tests for modifier key handling"
```

**Commit Types:**
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Adding tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvements
- `style:` - Code style changes
- `ci:` - CI/CD changes

### 4. Pull Request

```bash
# Push your branch
git push origin feature/your-feature-name

# Create pull request on GitHub
```

**Pull Request Template:**

```markdown
## Description
Brief description of the changes.

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Refactoring

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Tested on multiple compositors (if applicable)

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or clearly documented)
```

## Specific Areas for Contribution

### High Priority

1. **Compositor Compatibility**
   - Test with different Wayland compositors
   - Fix compositor-specific issues
   - Document compatibility

2. **Error Handling**
   - Improve error messages
   - Add recovery mechanisms
   - Handle edge cases

3. **Performance**
   - Optimize keymap generation
   - Reduce protocol overhead
   - Memory usage improvements

4. **Documentation**
   - More examples
   - Better API documentation
   - Troubleshooting guides

### Medium Priority

1. **Features**
   - Configuration file support
   - Macro recording/playback
   - Advanced timing controls

2. **Testing**
   - Integration tests
   - Compositor compatibility tests
   - Performance benchmarks

3. **Tooling**
   - Better development scripts
   - Debugging utilities
   - Profiling tools

### Future Ideas

1. **GUI Interface**
   - Graphical configuration tool
   - Macro editor
   - Live testing interface

2. **Protocol Extensions**
   - Support for newer Wayland protocols
   - Custom protocol extensions
   - Enhanced capabilities

3. **Platform Support**
   - Additional Unix platforms
   - Alternative input methods
   - Cross-platform compatibility

## Code Review Process

### For Contributors

1. **Self-Review**
   - Review your own code first
   - Test thoroughly
   - Check documentation

2. **Address Feedback**
   - Respond to review comments
   - Make requested changes
   - Update tests as needed

3. **Stay Engaged**
   - Monitor your PR for feedback
   - Be responsive to questions
   - Help with testing

### For Reviewers

1. **Be Constructive**
   - Provide helpful feedback
   - Suggest improvements
   - Explain reasoning

2. **Focus on Important Issues**
   - Security concerns
   - Correctness
   - Performance impacts
   - API design

3. **Be Timely**
   - Review promptly
   - Don't block unnecessarily
   - Approve when ready

## Community Guidelines

### Communication

- **Be Respectful** - Treat all contributors with respect
- **Be Patient** - Remember that people have different experience levels
- **Be Helpful** - Share knowledge and assist others
- **Be Constructive** - Focus on solutions, not problems

### Collaboration

- **Share Knowledge** - Document solutions and learnings
- **Help Others** - Review code, answer questions
- **Stay Informed** - Keep up with project developments
- **Participate** - Join discussions, provide feedback

## Getting Help

### Questions?

- **GitHub Discussions** - For general questions
- **GitHub Issues** - For specific problems
- **Code Comments** - For implementation details
- **Documentation** - Check existing docs first

### Stuck?

- Review similar implementations
- Ask for help in discussions
- Break down the problem
- Start with smaller changes

## Recognition

Contributors are recognized through:

- Git commit history
- Contributors file
- Release notes
- Special thanks in documentation

We appreciate all contributions, large and small! Whether you're fixing a typo, adding a feature, or improving documentation, your help makes wrtype better for everyone.

Thank you for contributing! 🦀