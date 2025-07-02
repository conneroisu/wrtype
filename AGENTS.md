# AGENTS.md - Development Guidelines for wrtype

## Build/Test Commands
- `cargo build` - Build the project
- `cargo check` - Check for compilation errors without building
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run specific test
- `cargo run -- <args>` - Run with arguments (e.g., `cargo run -- "hello world"`)
- `cargo clippy` - Run linter
- `cargo fmt` - Format code

## Code Style Guidelines
- **Language**: Rust 2021 edition
- **Imports**: Group std, external crates, then local modules with blank lines between
- **Documentation**: Comprehensive doc comments for all public items using `///`
- **Error Handling**: Use `anyhow::Result<T>` for functions that can fail, `thiserror` for custom errors
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_SNAKE_CASE for constants
- **Structure**: Organize code in logical modules (main.rs, executor.rs, keymap.rs, wayland.rs)
- **Comments**: Detailed module-level comments explaining purpose and architecture
- **Types**: Explicit type annotations where helpful, prefer `u32` for keycodes, `Duration` for timing
- **Memory**: Use `Vec` for dynamic collections, `HashMap` for lookups, avoid unnecessary clones
- **Async**: This is a synchronous CLI tool - no async/await patterns needed