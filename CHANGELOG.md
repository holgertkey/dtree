# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-01-24

### Added
- **Interactive Tree Navigation**: Visual directory tree with expand/collapse functionality
- **File Preview**: Syntax-highlighted preview for 100+ programming languages
- **Fuzzy Search**: Asynchronous two-phase search with intelligent fuzzy matching
  - Normal mode: Substring matching
  - Fuzzy mode: SkimMatcherV2 algorithm with scoring and character highlighting
- **Bookmarks System**: Save and quickly jump to favorite directories
  - Persistent storage in `~/.config/dtree/bookmarks.json`
  - Interactive creation and selection modes
  - Deletion with confirmation
- **Directory Size Calculation**: Asynchronous size calculation with visual indicators
  - Background threads per directory
  - Safety limits (5s timeout, 10K files)
  - Formatted output (K/M/G/T)
- **Fullscreen File Viewer**: Dedicated viewer with advanced features
  - Syntax highlighting with configurable themes
  - Line numbers toggle (`l`)
  - Line wrapping toggle (`w`)
  - HEAD/TAIL mode switching (Home/End keys)
  - File search within content (`/`, `n`/`N` navigation)
  - Jump between files in directory (Ctrl+j/Ctrl+k)
- **Visual Selection Mode**: Vim-style line selection (V key)
  - Keyboard selection (j/k, Page Up/Down, Home/End)
  - Mouse selection support with auto-scroll
  - Copy to clipboard (`y`)
  - Visual feedback with highlighting
- **Binary File Support**: Automatic detection and handling
  - Hex editor integration
  - Informational display for binary files
- **Customizable Configuration**: TOML-based configuration system
  - Theme presets: default, gruvbox, nord, tokyonight, dracula, obsidian
  - Custom color schemes (names, hex, indexed)
  - Configurable keybindings
  - External editor/hex editor/file manager integration
  - Auto-creation of config on first run
- **File Icons**: Support for Nerd Fonts and emoji fallbacks
  - 100+ programming languages
  - Configuration files (Cargo.toml, package.json, etc.)
  - Special directories (.git, node_modules, etc.)
  - Media files
- **Mouse Support**: Full mouse interaction
  - Click to select/expand/collapse
  - Double-click to navigate
  - Drag to resize split view
  - Scroll in file viewer and tree
- **Bash Integration**: Seamless shell integration with `dt` wrapper
  - Direct navigation: `dt /path`
  - Bookmark jumping: `dt myproject`
  - Previous directory: `dt -`
  - File viewing: `dt -v file.txt`
- **Comprehensive Help System**: Interactive help screen (`i` key)

### Performance
- **Zero-Copy Tree Operations**: Uses `Rc<RefCell<>>` for efficient tree manipulation
- **Lazy Loading**: Directories and files loaded only when needed
- **Async Operations**: Non-blocking search and size calculation
- **Optimized Binary**: 2.5 MB with aggressive size optimization
  - LTO enabled
  - Symbol stripping
  - Single codegen unit

### Robustness
- **Comprehensive Terminal Cleanup**: Multi-stage cleanup prevents artifacts
  - Explicit disabling of all 6 mouse tracking modes
  - Double event draining (before and after screen transition)
  - Proper timing delays for terminal processing
  - Handles terminal resize in split view mode without artifacts
- **Graceful Error Handling**: No `std::process::exit()` calls
  - All errors propagate through `anyhow::Result`
  - Single exit point in main()
  - Detailed, user-friendly error messages
  - Config parse errors with fix instructions
- **Panic Recovery**: Panic hook ensures terminal restoration
  - Installed during terminal setup
  - Guarantees cleanup even on crash
- **Event::Resize Handling**: Prevents event accumulation and leakage

### Documentation
- Complete README with installation, usage, and features
- Architecture documentation in `docs/`
- Troubleshooting guide with common issues
- Configuration reference
- Keybindings cheat sheet
- CLI options documentation

### Technical Details
- **Language**: Rust (edition 2021)
- **TUI Framework**: ratatui 0.28
- **Terminal Backend**: crossterm 0.28
- **Lines of Code**: ~6,500
- **Test Coverage**: 33 tests (unit + integration)
- **Dependencies**: Minimal, all up-to-date

### Known Limitations
- No file operations (copy, move, delete) - use file manager integration
- No Windows native support (use WSL)
- No plugin system yet

## [Unreleased]

### Planned for v1.1.0
- CI/CD pipeline with GitHub Actions
- Increased test coverage (target: 80%+)
- Performance benchmarks
- Windows native support
- More customizable keybindings

---

For detailed roadmap and future plans, see [ROADMAP.md](ROADMAP.md).

[1.0.0]: https://github.com/holgertkey/dtree/releases/tag/v1.0.0
