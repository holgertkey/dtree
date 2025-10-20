# Contributing to dtree

Thank you for your interest in contributing to dtree! This guide will help you get started.

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

## Ways to Contribute

### 1. Report Bugs

Found a bug? Please [open an issue](https://github.com/holgertkey/dtree/issues/new) with:

- **Clear title**: Describe the issue concisely
- **Steps to reproduce**: List exact steps to trigger the bug
- **Expected behavior**: What should happen
- **Actual behavior**: What actually happens
- **Environment**: OS, terminal, Rust version
- **Screenshots**: If applicable

### 2. Suggest Features

Have an idea? [Open an issue](https://github.com/holgertkey/dtree/issues/new) with:

- **Use case**: Why is this feature needed?
- **Proposed solution**: How should it work?
- **Alternatives**: Other approaches you've considered
- **Examples**: Similar features in other tools

### 3. Improve Documentation

Documentation improvements are always welcome:

- Fix typos or unclear wording
- Add examples
- Improve formatting
- Translate to other languages (future)

### 4. Write Code

Contribute code by:

- Fixing bugs
- Implementing features
- Improving performance
- Adding tests
- Refactoring code

## Development Setup

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **Terminal**: Any modern terminal emulator

### Clone and Build

```bash
# Fork the repository on GitHub first

# Clone your fork
git clone https://github.com/YOUR_USERNAME/dtree.git
cd dtree

# Add upstream remote
git remote add upstream https://github.com/holgertkey/dtree.git

# Build debug version
cargo build

# Run in debug mode
cargo run

# Run with arguments
cargo run -- -v README.md
```

### Development Workflow

```bash
# Create a feature branch
git checkout -b feature/my-feature

# Make changes and test
cargo run

# Run tests
cargo test

# Check code formatting
cargo fmt --check

# Run linter
cargo clippy

# Build release
cargo build --release

# Commit changes
git add .
git commit -m "Add my feature"

# Push to your fork
git push origin feature/my-feature

# Open a pull request on GitHub
```

## Project Structure

```
dtree/
├── src/
│   ├── main.rs           # Entry point, CLI, terminal setup
│   ├── app.rs            # Application state manager
│   ├── navigation.rs     # Tree navigation logic
│   ├── file_viewer.rs    # File content display
│   ├── search.rs         # Search functionality
│   ├── ui.rs             # Rendering logic
│   ├── event_handler.rs  # Input processing
│   ├── config.rs         # Configuration management
│   ├── bookmarks.rs      # Bookmark management
│   ├── dir_size.rs       # Directory size calculation
│   ├── file_icons.rs     # File type icons
│   ├── theme.rs          # Color theming
│   └── tree_node.rs      # Tree data structure
├── docs/                 # Documentation
├── tests/                # Integration tests
├── Cargo.toml            # Dependencies
├── CLAUDE.md             # Development guide for Claude
└── README.md             # Project README
```

## Architecture

dtree follows a modular architecture. See [Architecture](./architecture.md) for details.

**Key principles**:

1. **Separation of concerns**: Each module has a single responsibility
2. **Composition over inheritance**: `app.rs` orchestrates submodules
3. **Zero-copy when possible**: Use `Rc<RefCell<>>` for shared ownership
4. **Async for slow operations**: Background threads for search, size calculation
5. **Graceful error handling**: Never crash, always inform user

## Coding Guidelines

### Rust Style

Follow standard Rust conventions:

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)

### Code Organization

- **Keep modules focused**: Single responsibility per module
- **Keep functions small**: Aim for < 50 lines
- **Avoid `app.rs` bloat**: Create new modules instead
- **Use descriptive names**: `handle_search_input` not `hsi`

### Error Handling

- **Use `anyhow::Result`**: For ergonomic error propagation
- **Never panic**: Use `Result` or `Option`
- **Graceful degradation**: Show errors to user, don't crash
- **Detailed messages**: "Cannot read: Permission denied" not "Error"

### Performance

- **Measure before optimizing**: Use benchmarks
- **Avoid unnecessary clones**: Use references or `Rc<RefCell<>>`
- **Lazy load**: Don't load what you don't need
- **Async for slow ops**: Background threads for long-running tasks

### Comments

- **Explain why, not what**: Code shows what, comments explain why
- **Document public APIs**: Use doc comments (`///`)
- **English only**: All comments in English
- **No commented code**: Remove, don't comment

### Testing

- **Write tests for new features**: Aim for >70% coverage
- **Test edge cases**: Empty directories, permission errors, etc.
- **Use meaningful names**: `test_expand_collapses_all_children`
- **Mock I/O**: Use mock filesystem for tests

## Pull Request Process

### Before Submitting

1. **Run tests**: `cargo test`
2. **Run clippy**: `cargo clippy`
3. **Run fmt**: `cargo fmt`
4. **Test manually**: Run dtree and test your changes
5. **Update docs**: If adding features or changing behavior
6. **Write tests**: Add tests for new functionality

### PR Guidelines

**Title**:
- Clear and descriptive
- Use imperative mood: "Add feature" not "Added feature"
- Examples: "Fix cursor bug in search mode", "Add fuzzy search support"

**Description**:
- Explain what and why
- Link to related issues
- Add screenshots for UI changes
- List breaking changes

**Template**:
```markdown
## Summary
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- Bullet list of changes
- Be specific

## Testing
How was this tested?

## Screenshots
If applicable

## Breaking Changes
List any breaking changes

Closes #123
```

### Review Process

1. **Automated checks**: CI runs tests, clippy, fmt
2. **Manual review**: Maintainer reviews code
3. **Feedback**: Address review comments
4. **Approval**: Once approved, PR will be merged
5. **Merge**: Squash and merge (maintains clean history)

## Common Tasks

### Adding a New Feature

1. **Plan the feature**:
   - Define scope and behavior
   - Identify which module(s) to modify
   - Consider performance and UX

2. **Implement**:
   - Create feature branch
   - Write code in appropriate module
   - Add keybindings if needed
   - Update help text

3. **Test**:
   - Manual testing
   - Write unit tests
   - Test edge cases

4. **Document**:
   - Update README if major feature
   - Add to docs/features.md
   - Update help text (HELP.txt)
   - Update CLAUDE.md roadmap

5. **Submit PR**

### Fixing a Bug

1. **Reproduce**: Understand the bug fully
2. **Locate**: Find the source of the bug
3. **Fix**: Implement minimal fix
4. **Test**: Verify fix works
5. **Prevent**: Add test to prevent regression
6. **Submit PR**

### Improving Performance

1. **Measure**: Use profiling tools
2. **Identify**: Find bottleneck
3. **Optimize**: Implement optimization
4. **Benchmark**: Verify improvement
5. **Test**: Ensure correctness
6. **Submit PR** with benchmark results

### Adding Documentation

1. **Identify gap**: What's missing or unclear?
2. **Write**: Clear, concise, with examples
3. **Review**: Read from user perspective
4. **Submit PR**

## Code Review Guidelines

### As a Reviewer

- **Be kind**: Assume good intent
- **Be specific**: "Consider using X" not "This is wrong"
- **Explain why**: Help author learn
- **Approve generously**: Perfect is enemy of good

### As an Author

- **Respond promptly**: Address feedback quickly
- **Ask questions**: If unclear, ask
- **Don't take it personally**: Focus on code, not ego
- **Iterate**: Multiple rounds of review are normal

## Getting Help

- **Documentation**: Check [docs](.) first
- **Issues**: Search existing issues
- **Discussions**: Ask on GitHub Discussions
- **Contact**: Open an issue for help

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## Recognition

Contributors will be recognized in:

- README.md (major contributions)
- Git commit history
- Release notes

## Questions?

Don't hesitate to ask! Open an issue or discussion, and we'll help you get started.

## Thank You!

Every contribution, no matter how small, makes dtree better. Thank you for your time and effort!
