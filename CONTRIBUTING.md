# Contributing to ADS-B TUI

Thank you for your interest in contributing to ADS-B TUI! This document provides guidelines and information for contributors.

## 🚀 Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/yourusername/adsb-tui.git
   cd adsb-tui
   ```
3. **Set up the development environment**:
   ```bash
   # Install Rust (if not already installed)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Install development tools
   cargo install cargo-watch cargo-edit cargo-audit

   # Run the project
   cargo run
   ```

## 🛠️ Development Workflow

### 1. Choose an Issue
- Check the [Issues](https://github.com/yourusername/adsb-tui/issues) page
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch
```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 3. Make Changes
- Write clear, concise commit messages
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed

### 4. Test Your Changes
```bash
# Run all checks
just check

# Run tests specifically
cargo test

# Test the application manually
cargo run
```

### 5. Submit a Pull Request
- Push your branch to your fork
- Create a Pull Request from your branch to the main repository
- Fill out the PR template with details about your changes
- Link to any related issues

## 📝 Code Guidelines

### Rust Style
- Follow the official [Rust Style Guide](https://doc.rust-lang.org/style-guide/)
- Use `cargo fmt` to format your code
- Fix all `cargo clippy` warnings
- Write idiomatic Rust code

### Commit Messages
- Use the present tense ("Add feature" not "Added feature")
- Start with a capital letter
- Keep the first line under 50 characters
- Add more detail in the body if needed

Examples:
```
Add support for custom column widths

The new column_widths config option allows users to specify
custom widths for each column in the aircraft table.
```

### Testing
- Add unit tests for new functions
- Add integration tests for new features
- Ensure all existing tests pass
- Test on multiple platforms if possible

## 🏗️ Project Structure

```
adsb-tui/
├── src/
│   ├── main.rs          # Application entry point
│   ├── app.rs           # Main application logic
│   ├── ui.rs            # Terminal user interface
│   ├── config.rs        # Configuration parsing
│   ├── model.rs         # Data structures
│   ├── net.rs           # Network operations
│   ├── routes.rs        # Flight route handling
│   ├── export.rs        # Data export
│   ├── storage.rs       # File operations
│   └── watchlist.rs     # Watchlist management
├── .github/
│   └── workflows/       # CI/CD pipelines
├── tests/               # Integration tests
├── docs/                # Documentation
├── Cargo.toml           # Package configuration
├── README.md            # Main documentation
└── justfile             # Development tasks
```

## 🧪 Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        // Test code here
        assert_eq!(result, expected);
    }
}
```

### Integration Tests
Create files in the `tests/` directory:
```rust
// tests/integration_test.rs
use adsb_tui::*;

#[test]
fn test_full_workflow() {
    // Integration test code
}
```

### Manual Testing
- Test with real ADS-B data sources
- Verify UI responsiveness
- Check keyboard shortcuts
- Test configuration options

## 📚 Documentation

### Code Documentation
- Add doc comments to public functions and structs
- Use `cargo doc` to generate documentation
- Include examples in doc comments when helpful

```rust
/// Calculate the distance between two points
///
/// # Examples
///
/// ```
/// let distance = calculate_distance(point1, point2);
/// assert!(distance > 0.0);
/// ```
pub fn calculate_distance(p1: Point, p2: Point) -> f64 {
    // implementation
}
```

### User Documentation
- Update README.md for new features
- Add configuration examples
- Update keyboard shortcuts documentation

## 🔧 Development Tools

### Just Commands
```bash
just build          # Build in debug mode
just build-release  # Build in release mode
just test           # Run tests
just fmt            # Format code
just clippy         # Run linter
just check          # Run all checks
just run            # Run the application
```

### Useful Cargo Commands
```bash
cargo check         # Quick compilation check
cargo build         # Build project
cargo test          # Run tests
cargo doc           # Generate documentation
cargo bench         # Run benchmarks
cargo update        # Update dependencies
```

## 🚨 Issue Reporting

When reporting bugs, please include:
- **Version**: Output of `adsb-tui --version`
- **Platform**: OS and architecture
- **Configuration**: Relevant config file sections
- **Steps to reproduce**: Detailed steps
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Logs**: Any error messages or logs

## 💡 Feature Requests

For new features:
- Check if the feature already exists
- Describe the use case clearly
- Explain why it's needed
- Consider implementation complexity

## 📞 Getting Help

- **Issues**: For bugs and feature requests
- **Discussions**: For questions and general discussion
- **Discord**: For real-time chat (if available)

## 📋 Pull Request Checklist

Before submitting a PR:
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Code is formatted with `cargo fmt`
- [ ] Clippy warnings are fixed
- [ ] Documentation is updated
- [ ] Commit messages are clear
- [ ] PR description explains the changes

## 🎉 Recognition

Contributors will be:
- Listed in the README.md contributors section
- Mentioned in release notes
- Credited for their work

Thank you for contributing to ADS-B TUI! 🎉
