# Contributing to Tooka

Thank you for your interest in contributing to Tooka! 🎉

We welcome contributions of all kinds: bug reports, feature requests, documentation improvements, and code contributions.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [How to Contribute](#how-to-contribute)
- [Development Setup](#development-setup)
- [Coding Guidelines](#coding-guidelines)
- [Submitting Changes](#submitting-changes)
- [Community](#community)

## Code of Conduct

This project adheres to a [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to demetzbenjamin23@gmail.com.

## Getting Started

### Prerequisites

- Rust 1.87 or later
- Cargo (comes with Rust)
- Git

### Fork and Clone

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/tooka.git
   cd tooka
   ```
3. Add the upstream repository:
   ```bash
   git remote add upstream https://github.com/tooka-org/tooka.git
   ```

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check the [existing issues](https://github.com/tooka-org/tooka/issues) to avoid duplicates.

When filing a bug report, use the bug report template and include:
- A clear description of the issue
- Steps to reproduce the behavior
- Expected vs. actual behavior
- Logs (see [wiki](https://github.com/Benji377/tooka/wiki/Configuration) for log location)
- Your environment (OS, terminal, version)

### Suggesting Features

Feature requests are welcome! Please use the feature request template and:
- Explain the problem you're trying to solve
- Describe your proposed solution
- Consider alternative approaches
- Provide any additional context

For larger features, consider opening a [discussion](https://github.com/tooka-org/tooka/discussions) first to gather feedback.

### Improving Documentation

Documentation improvements are always appreciated:
- Fix typos or unclear explanations
- Add examples or use cases
- Improve the wiki
- Write or improve code comments

## Development Setup

1. **Build the project:**
   ```bash
   cargo build
   ```

2. **Run the application:**
   ```bash
   cargo run -- [arguments]
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Run linter:**
   ```bash
   cargo clippy --all-features
   ```

5. **Format code:**
   ```bash
   cargo fmt
   ```

## Coding Guidelines

### General Principles

- Write clear, readable code
- Follow the existing code style
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### Rust Conventions

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Run `cargo fmt` before committing
- Ensure `cargo clippy` passes without warnings
- Write comprehensive tests for new features
- Document public APIs with doc comments

### Testing

- All new features should include tests
- Ensure all tests pass: `cargo test`
- Add tests to the appropriate test module
- Test edge cases and error conditions

### Commit Messages

Write clear, descriptive commit messages:
- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Keep the first line under 50 characters
- Provide additional context in the body if needed

Example:
```
Add support for custom date formats

- Implement date format parsing in utils
- Update rule matching to support custom formats
- Add tests for various date format patterns
```

## Submitting Changes

1. **Create a branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   - Write your code
   - Add or update tests
   - Update documentation

3. **Test your changes:**
   ```bash
   cargo test
   cargo clippy --all-features
   cargo fmt
   ```

4. **Commit your changes:**
   ```bash
   git add .
   git commit -m "Your descriptive commit message"
   ```

5. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Open a Pull Request:**
   - Go to the [Tooka repository](https://github.com/tooka-org/tooka)
   - Click "New Pull Request"
   - Select your fork and branch
   - Fill out the PR template
   - Link any related issues

### Pull Request Guidelines

- Fill out the PR template completely
- Link to relevant issues
- Ensure all CI checks pass
- Respond to review feedback promptly
- Keep PRs focused on a single feature or fix
- Update your branch with main if needed

## Community

- **Discussions:** [GitHub Discussions](https://github.com/tooka-org/tooka/discussions)
- **Issues:** [GitHub Issues](https://github.com/tooka-org/tooka/issues)
- **Quick Feedback:** [Feedback Form](https://tally.so/r/mBVyLe)

## Questions?

If you have questions about contributing, feel free to:
- Open a [discussion](https://github.com/tooka-org/tooka/discussions)
- Ask in an existing issue
- Submit the [feedback form](https://tally.so/r/mBVyLe)

---

Thank you for contributing to Tooka! Your efforts help make this project better for everyone. ❤️
