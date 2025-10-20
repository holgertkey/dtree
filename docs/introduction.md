# Introduction

**dtree** is a lightweight, fast, and feature-rich Terminal User Interface (TUI) application for interactive directory tree navigation. It combines the visual appeal of a tree view with powerful features like file preview, fuzzy search, bookmarks, and more.

## What is dtree?

dtree provides a visual, interactive way to navigate your filesystem directly from the terminal. Unlike traditional command-line tools like `cd` and `ls`, dtree gives you a bird's-eye view of your directory structure while maintaining the speed and efficiency of terminal-based workflows.

## Key Features

- **Interactive Tree View**: Visual directory tree with expand/collapse functionality
- **File Preview**: Syntax-highlighted preview panel with support for 100+ languages
- **Fuzzy Search**: Fast, asynchronous search with intelligent fuzzy matching
- **Bookmarks**: Save and quickly jump to favorite directories
- **Directory Sizes**: Asynchronous calculation of directory sizes with visual indicators
- **Fullscreen Viewer**: Dedicated file viewer with search, line numbers, and tail mode
- **Binary File Support**: Automatic detection with hex editor integration
- **Mouse Support**: Click, double-click, drag, and scroll
- **Customizable**: TOML-based configuration with theme support
- **Bash Integration**: Seamlessly integrates with your shell workflow

## Why dtree?

### Speed
- Written in Rust for maximum performance
- Zero-copy tree operations using `Rc<RefCell<>>`
- Asynchronous background operations (search, size calculation)
- Lazy loading of directory contents

### Usability
- Vim-inspired keybindings
- Mouse support for quick navigation
- Context-aware UI (different modes for different tasks)
- Comprehensive help system (press `i`)

### Integration
- Works seamlessly with your existing shell
- Integrates with external editors and file managers
- Respects terminal capabilities (colors, mouse, clipboard)

## Philosophy

dtree follows these design principles:

1. **Fast by default**: Operations should be instant or clearly show progress
2. **Keyboard-first**: All features accessible via keyboard, mouse optional
3. **Terminal-native**: No GUI dependencies, works over SSH
4. **Non-destructive**: View and navigate, never modify files
5. **Configurable**: Sensible defaults, everything customizable

## Next Steps

- [Getting Started](./getting-started.md) - Quick start guide
- [Installation](./installation.md) - Installation instructions
- [Basic Usage](./usage.md) - Learn the basics
- [Configuration](./configuration.md) - Customize dtree
