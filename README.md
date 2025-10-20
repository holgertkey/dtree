# dtree - Directory Tree Navigator

**A fast, lightweight, and feature-rich TUI for interactive directory tree navigation.**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

---

## Features

- 🌳 **Interactive Tree View** - Visual directory navigation with expand/collapse
- 📄 **File Preview** - Syntax-highlighted preview for 100+ languages
- 🔍 **Fuzzy Search** - Fast asynchronous search with intelligent matching
- 🔖 **Bookmarks** - Save and quickly jump to favorite directories
- 📏 **Directory Sizes** - Async calculation with visual indicators
- 🖥️ **Fullscreen Viewer** - Dedicated file viewer with search and tail mode
- 🔧 **Binary File Support** - Automatic detection with hex editor integration
- 🎨 **Customizable** - TOML configuration with full theme support
- 🖱️ **Mouse Support** - Click, drag, scroll, and resize
- ⚡ **Fast** - Zero-copy tree operations, async background tasks

---

## Quick Start

```bash
# Build from source
git clone https://github.com/holgertkey/dtree.git
cd dtree
cargo build --release
cp target/release/dtree ~/bin/

# Launch from current directory
dt

# Navigate to a directory
dt /path/to/directory

# View a file in fullscreen
dt -v README.md
```

---

## Installation

### From Source

```bash
git clone https://github.com/holgertkey/dtree.git
cd dtree
cargo build --release

# Install to user bin
mkdir -p ~/bin
cp target/release/dtree ~/bin/

# Or install system-wide
sudo cp target/release/dtree /usr/local/bin/
```

### Bash Integration (Recommended)

Add this to your `~/.bashrc` for seamless shell integration:

```bash
# Directory tree navigator
dt() {
  # Store current directory before navigation
  local prev_dir="$PWD"

  # Handle special case: dt - (return to previous directory)
  if [ "$1" = "-" ]; then
    if [ -n "$DTREE_PREV_DIR" ] && [ -d "$DTREE_PREV_DIR" ]; then
      cd "$DTREE_PREV_DIR" || return
      export DTREE_PREV_DIR="$prev_dir"
    else
      echo "dt: no previous directory" >&2
      return 1
    fi
    return
  fi

  # If flags or bookmark commands are passed, run dtree directly
  case "$1" in
    -h|--help|--version)
      command dtree "$@"
      return
      ;;
    -bm)
      # Bookmark management - run directly
      command dtree "$@"
      return
      ;;
  esac

  # For navigation: dtree resolves paths/bookmarks
  local result=$(command dtree "$@")
  local exit_code=$?

  if [ $exit_code -ne 0 ]; then
    return $exit_code
  fi

  # Only cd if result is a valid directory
  if [ -n "$result" ] && [ -d "$result" ]; then
    cd "$result" || return
    # Save previous directory for dt -
    export DTREE_PREV_DIR="$prev_dir"
  fi
}
```

Then reload your shell:

```bash
source ~/.bashrc
```

---

## Usage

### Navigation

```
j/k or ↓/↑    Navigate down/up
l/→ or h/←    Expand/collapse directory
Enter         Change root to selected directory
u/Backspace   Go to parent directory
q             Exit and cd to selected directory
```

### File Viewing

```
s             Toggle file viewer mode
v             Open file in fullscreen viewer
Ctrl+j/k      Scroll file preview
e             Open in external editor
o             Open in file manager
```

### Search

```
/             Enter search mode
              Type query (or /query for fuzzy search)
Enter         Execute search
Tab           Switch focus between tree and results
n/N           Next/previous match (in fullscreen)
```

### Bookmarks

```
m             Create bookmark
'             Open bookmark selection
dt myproject  Jump to bookmark (from command line)
dt -bm list   List all bookmarks
```

### Other Features

```
i             Toggle help screen
c             Copy current path to clipboard
z             Toggle directory size display
```

---

## Key Features

### 🔍 Intelligent Search

- **Two-phase search**: Quick local search + deep background search
- **Fuzzy matching**: Start query with `/` for fuzzy mode
- **Async**: Non-blocking UI during search
- **Ranked results**: Best matches first with relevance scores

### 🔖 Powerful Bookmarks

- **Multi-character names**: Use descriptive names like `myproject-backend`
- **CLI management**: Add, remove, list from command line
- **Direct navigation**: `dt myproject` jumps instantly
- **Filter mode**: Type to filter bookmarks by name or path
- **Safe deletion**: Two-phase deletion prevents accidents

### 📄 Advanced File Viewer

- **Syntax highlighting**: 100+ languages supported
- **Line numbers**: Toggle with `l` key
- **Search within files**: Press `/` in fullscreen mode
- **Tail mode**: View last 10,000 lines of large files (perfect for logs)
- **Text selection**: Shift+Mouse to select and copy

### 🎨 Customization

Configuration file: `~/.config/dtree/config.toml`

```toml
[appearance]
split_position = 50
show_icons = true
enable_syntax_highlighting = true
syntax_theme = "base16-ocean.dark"

[appearance.colors]
selected_color = "cyan"
directory_color = "blue"
file_color = "white"

[behavior]
max_file_lines = 10000
editor = "nvim"
hex_editor = "hexyl"
file_manager = "ranger"
```

### 📦 Binary File Support

- **Auto-detection**: Checks for NULL bytes
- **Hex editor integration**: Press `e` to view in hex editor
- **Supported types**: Executables, images, archives, videos, PDFs, databases

---

## Screenshots

### Tree View
```
┌─ /home/user/projects ──────────────────────────┐
│ 📁 dtree/                                       │
│ ├─ 📁 src/                                      │
│ │  ├─ 📄 main.rs                                │
│ │  ├─ 📄 app.rs                                 │
│ │  ├─ 📄 navigation.rs                          │
│ │  ├─ 📄 file_viewer.rs                         │
│ │  └─ 📄 search.rs                              │
│ ├─ 📄 Cargo.toml                                │
│ ├─ 📄 README.md                                 │
│ └─ 📁 docs/                                     │
│    └─ 📄 architecture.md                        │
└─────────────────────────────────────────────────┘
```

### Split View with File Preview
```
┌─ Tree ────────┬─ File Preview ──────────────────┐
│ dtree/        │ # Architecture                   │
│ ├─ src/       │                                  │
│ │  main.rs    │ This document describes the      │
│ │  app.rs     │ internal architecture of dtree.  │
│ ├─ docs/      │                                  │
│ │  arch.md ◄──┼─ ## Overview                     │
│ └─ README.md  │                                  │
│               │ dtree is built with a modular... │
│               │                                  │
│ Rust | 128 lines | 4.2K                        │
└───────────────┴──────────────────────────────────┘
```

### Fullscreen Viewer
```
┌─ docs/architecture.md ──────────────────────────┐
│   1 │ # Architecture                            │
│   2 │                                           │
│   3 │ This document describes the internal      │
│   4 │ architecture of dtree.                    │
│   5 │                                           │
│   6 │ ## Overview                               │
│   7 │                                           │
│   8 │ dtree is built with a modular             │
│   9 │ architecture that separates concerns...   │
│     │                                           │
│ l: line# | /: search | q: back | Esc: exit     │
└──────────────────────────────────────────────────┘
```

---

## Documentation

Complete documentation is available in the [docs](./docs) directory:

- **[Getting Started](./docs/getting-started.md)** - Quick start guide
- **[Installation](./docs/installation.md)** - Installation instructions
- **[Usage](./docs/usage.md)** - Basic usage guide
- **[Configuration](./docs/configuration.md)** - Configuration reference
- **[Key Bindings](./docs/keybindings.md)** - Complete keybinding reference
- **[Features](./docs/features.md)** - Feature documentation
- **[Architecture](./docs/architecture.md)** - Internal architecture
- **[Contributing](./docs/contributing.md)** - Contribution guide

---

## Requirements

### Build Requirements

- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs/)

### Optional Dependencies

- **Nerd Fonts** - For file type icons (enable with `show_icons = true`)
- **hexyl** - For binary file viewing (`cargo install hexyl`)
- **xclip** - For clipboard support on Linux (usually pre-installed)

---

## Performance

dtree is designed for speed:

- **Zero-copy tree operations** - Uses `Rc<RefCell<>>` to avoid cloning
- **Lazy loading** - Directories loaded only when expanded
- **Async operations** - Search and size calculations run in background
- **Efficient rendering** - Only visible nodes are processed

Typical performance:
- **Tree navigation**: Instant (< 1ms)
- **Search**: ~100,000 files/second
- **File preview**: Loads up to 10,000 lines instantly

---

## Architecture

dtree has a modular architecture with separated concerns:

```
main.rs          Entry point, CLI, terminal setup
app.rs           State manager (orchestrator)
navigation.rs    Tree navigation logic
file_viewer.rs   File content display
search.rs        Search functionality
ui.rs            Rendering logic
event_handler.rs Input processing
config.rs        Configuration management
bookmarks.rs     Bookmark management
```

See [Architecture](./docs/architecture.md) for details.

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./docs/contributing.md) for guidelines.

### Development Setup

```bash
# Clone repository
git clone https://github.com/holgertkey/dtree.git
cd dtree

# Run in debug mode
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Built with [Ratatui](https://github.com/ratatui-org/ratatui) TUI framework
- Syntax highlighting powered by [syntect](https://github.com/trishume/syntect)
- Fuzzy search using [fuzzy-matcher](https://github.com/lotabout/fuzzy-matcher)
- Inspired by tools like `tree`, `ranger`, and `nnn`

---

## Support

- **Issues**: Report bugs and request features on [GitHub Issues](https://github.com/holgertkey/dtree/issues)
- **Discussions**: Ask questions on [GitHub Discussions](https://github.com/holgertkey/dtree/discussions)
- **Documentation**: Full docs in [docs](./docs) directory

---

**Made with ❤️ in Rust**
