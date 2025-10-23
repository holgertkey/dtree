# dtree - Directory Tree Navigator

**A fast, lightweight, and feature-rich TUI for interactive directory tree navigation.**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)

---

## Why dtree?

**dtree** combines the simplicity of `tree` with the power of modern file managers, creating the perfect tool for developers who live in the terminal. Unlike traditional file managers that try to do everything, dtree focuses on what matters most: **fast navigation** and **intelligent file viewing**.

**Navigate instantly.** Jump between projects with bookmarks, find files with fuzzy search, and preview code with syntax highlighting - all without leaving your keyboard. No more endless `cd` commands or opening heavy GUIs just to peek at a file.

**Stay in flow.** Built with Rust for blazing-fast performance and zero-copy operations, dtree handles massive directory trees without breaking a sweat. Asynchronous search runs in the background while you keep working. Your terminal, your speed.

**Your tool, your way.** Fully customizable themes, configurable keybindings, and seamless shell integration. Works beautifully with vim-style navigation, integrates with your favorite editor, and fits naturally into your existing workflow.

---

## Features

- 🌳 **Interactive Tree View** - Visual directory navigation with expand/collapse
- 📄 **File Preview** - Syntax-highlighted preview for 100+ languages
- 🔍 **Fuzzy Search** - Fast asynchronous search with intelligent matching
- 🔖 **Bookmarks** - Save and quickly jump to favorite directories
- 📏 **Directory Sizes** - Async calculation with visual indicators
- 🖥️ **Fullscreen Viewer** - Dedicated file viewer with search and tail mode
- ✂️ **Visual Selection** - Vim-style line selection with keyboard and mouse
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

# Create and use bookmarks
dt                    # Open dtree
# Press 'm' to create bookmark
# Press 'q' to exit
dt myproject          # Jump to bookmark instantly

# Basic navigation inside dtree:
# j/k - move up/down
# l/h - expand/collapse
# v - fullscreen viewer
# V - visual selection mode (in fullscreen)
# s - toggle file preview
# / - search
# i - help
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

## CLI Commands

### Basic Usage

```bash
# Launch interactive tree from current directory
dt

# Navigate to specific directory
dt /path/to/directory

# Jump to bookmark
dt myproject

# Return to previous directory (like cd -)
dt -
```

### File Viewing

```bash
# View file in fullscreen mode
dt -v README.md
dtree --view README.md

# After viewing, press 'q' to navigate to file's directory
```

### Bookmark Management

```bash
# List all bookmarks
dt -bm
dt -bm list

# Add bookmark for current directory
dt -bm add myproject

# Add bookmark for specific path
dt -bm add myproject /path/to/directory

# Remove bookmark
dt -bm remove myproject
```

### Help and Version

```bash
# Show help
dtree -h
dtree --help

# Show version
dtree --version
```

### Navigation Priority

When you run `dt <name>`, dtree resolves in this order:
1. **Bookmark** - If bookmark exists, jump to it
2. **Path** - If valid path, navigate to it
3. **Error** - Show error message

---

## Keyboard Shortcuts

### Tree Navigation

| Key                | Action                               |
|--------------------|--------------------------------------|
| `j` or `↓`         | Move down one item                   |
| `k` or `↑`         | Move up one item                     |
| `l` or `→`         | Expand directory (show children)     |
| `h` or `←`         | Collapse directory (hide children)   |
| `Enter`            | Change root to selected directory    |
| `u` or `Backspace` | Go to parent directory               |
| `q`                | Exit and cd to selected directory    |
| `Esc`              | Exit without changing directory      |

### File Viewing (Split View)

| Key         | Action                                       |
|-------------|----------------------------------------------|
| `s`         | Toggle file viewer mode (show/hide split)    |
| `v`         | Open file in fullscreen viewer               |
| `Ctrl+j`    | Scroll preview down by line                  |
| `Ctrl+k`    | Scroll preview up by line                    |
| `Page Down` | Scroll preview down by page                  |
| `Page Up`   | Scroll preview up by page                    |
| `Home`      | Jump to start of file                        |
| `End`       | Jump to end of file                          |

### Fullscreen Viewer

| Key         | Action                                     |
|-------------|--------------------------------------------|
| `j` or `↓`  | Scroll down by line                        |
| `k` or `↑`  | Scroll up by line                          |
| `Ctrl+j`    | Jump to next file in directory             |
| `Ctrl+k`    | Jump to previous file in directory         |
| `Page Down` | Scroll down by page                        |
| `Page Up`   | Scroll up by page                          |
| `Home`      | Switch to HEAD mode (first 10,000 lines)   |
| `End`       | Switch to TAIL mode (last 10,000 lines)    |
| `l`         | Toggle line numbers                        |
| `w`         | Toggle line wrapping (wrap/truncate)       |
| `/`         | Enter file search mode                     |
| `n`         | Next search match (when results exist)     |
| `N`         | Previous search match (Shift+n)            |
| `V`         | Enter visual selection mode                |
| `q`         | Return to tree view (stay in program)      |
| `Esc`       | Exit program (or clear search if active)   |

### Visual Selection Mode (Fullscreen Viewer)

| Key             | Action                                       |
|-----------------|----------------------------------------------|
| `V`             | Enter visual mode (Vim-style selection)      |
| `j` or `↓`      | Expand selection downward                    |
| `k` or `↑`      | Expand selection upward                      |
| `Page Down`     | Jump selection down by page                  |
| `Page Up`       | Jump selection up by page                    |
| `Home`          | Jump to start of file                        |
| `End`           | Jump to end of file                          |
| `Mouse Scroll`  | Move selection cursor (auto-scroll)          |
| `y`             | Copy selected lines and exit visual mode     |
| `Esc` or `V`    | Exit visual mode without copying             |

### Search (Tree Search)

| Key          | Action                                       |
|--------------|----------------------------------------------|
| `/`          | Enter search mode                            |
| Type         | Add characters to search query               |
| `Backspace`  | Remove last character                        |
| `Enter`      | Execute search and show results              |
| `Tab`        | Switch focus between tree and results panel  |
| `j` or `↓`   | Navigate down in results                     |
| `k` or `↑`   | Navigate up in results                       |
| `Esc`        | Close results and exit search mode           |

**Fuzzy Search**: Start query with `/` for fuzzy matching (e.g., `/fuz` finds "fuzzy.rs")

### Bookmarks

| Key        | Action                                   |
|------------|------------------------------------------|
| `m`        | Create bookmark (enter name)             |
| `'`        | Open bookmark selection menu             |
| `j` or `↓` | Navigate bookmarks (in selection)        |
| `k` or `↑` | Navigate bookmarks (in selection)        |
| `d`        | Delete bookmark (press twice to confirm) |
| `Tab`      | Toggle filter mode (type to filter)      |
| `Enter`    | Jump to selected bookmark                |
| `Esc`      | Close bookmark menu                      |

### File Operations

| Key | Action                                                   |
|-----|----------------------------------------------------------|
| `e` | Open file in external editor (or hex editor for binary)  |
| `o` | Open in file manager (dirs → self, files → parent)       |
| `c` | Copy current path to clipboard                           |

### Other

| Key | Action                        |
|-----|-------------------------------|
| `i` | Toggle help screen            |
| `z` | Toggle directory size display |

### Mouse Support

| Action               | Effect                              |
|----------------------|-------------------------------------|
| **Tree View**        |                                     |
| Click                | Select item under cursor            |
| Double-click         | Expand/collapse directory           |
| Scroll wheel         | Navigate tree up/down               |
| Drag divider         | Resize panels                       |
| **File Preview**     |                                     |
| Scroll wheel         | Scroll preview content              |
| **Fullscreen Viewer** |                                    |
| Scroll wheel         | Scroll document                     |
| Shift+Click+Drag     | Select text for copying             |
| **Visual Mode**      |                                     |
| Scroll wheel         | Move selection cursor (auto-scroll) |

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
- **Search within files**: Press `/` in fullscreen mode, navigate with `n`/`N`
- **Tail mode**: View last 10,000 lines of large files (perfect for logs)
- **Visual selection mode**: Vim-style line selection with `V` key
  - Select lines with `j`/`k` keyboard navigation
  - Use mouse scroll to expand selection
  - Copy to clipboard with `y`
  - Visual feedback with highlighted selection and cursor
- **Text selection**: Shift+Mouse drag for quick selection

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
wrap_lines = true        # Wrap long lines (true) or truncate (false)
editor = "nvim"
hex_editor = "hexyl"
file_manager = "ranger"
```

### 📦 Binary File Support

- **Auto-detection**: Checks for NULL bytes
- **Hex editor integration**: Press `e` to view in hex editor
- **Supported types**: Executables, images, archives, videos, PDFs, databases

---

## Screenshots & Demos

![dtree Screenshot](docs/assets/dtree_screenshot.png)

*dtree in action: interactive tree view with file preview and syntax highlighting*

**📺 [View All Demos & Animations →](docs/DEMOS.md)**

See animated demonstrations of:
- 🌳 Tree navigation with vim-style keybindings
- 📄 File viewer with syntax highlighting
- ✂️ Visual selection mode for copying text
- 🔍 Fuzzy search functionality
- 🔖 Bookmark management

---

## Quick Reference Card

Essential commands for daily use:

### Command Line
```bash
dt                    # Open tree navigator
dt /path              # Navigate to directory
dt myproject          # Jump to bookmark
dt -                  # Return to previous directory
dt -v file.txt        # View file in fullscreen
dt -bm list           # List bookmarks
```

### Inside dtree
```
Navigation:       j/k (down/up)   h/l (collapse/expand)   u (parent)
File Viewing:     s (toggle)      v (fullscreen)          e (editor)
Search:           / (search)      Enter (execute)         Tab (focus)
Bookmarks:        m (create)      ' (select)              d (delete)
Actions:          c (copy path)   o (file manager)        z (sizes)
Help:             i (help)        q (exit+cd)             Esc (exit)
```

### Fullscreen Viewer
```
Navigate:         j/k (scroll)    Ctrl+j/k (next/prev file)
Page:             PgUp/PgDn       Home/End (head/tail mode)
Search:           / (search)      n/N (next/prev match)
View:             l (line #)      w (wrap)            V (visual mode)
Visual Mode:      j/k (select)    y (copy)            Esc (exit)
Exit:             q (back to tree)
```

For complete keybinding reference, see [docs/keybindings.md](./docs/keybindings.md).

**📄 Printable Cheat Sheet**: [CHEATSHEET.md](./CHEATSHEET.md)

---

## Documentation

Complete documentation is available in the [docs](./docs) directory:

### User Guides
- **[Demo Gallery](./docs/DEMOS.md)** - Visual demonstrations and animated GIFs
- **[Getting Started](./docs/getting-started.md)** - Quick start guide
- **[Installation](./docs/installation.md)** - Installation instructions
- **[Usage](./docs/usage.md)** - Basic usage guide
- **[CLI Options](./docs/cli-options.md)** - Complete command-line reference
- **[Key Bindings](./docs/keybindings.md)** - Complete keybinding reference
- **[Configuration](./docs/configuration.md)** - Configuration reference
- **[Features](./docs/features.md)** - Feature documentation

### Developer Guides
- **[Architecture](./docs/architecture.md)** - Internal architecture
- **[Contributing](./docs/contributing.md)** - Contribution guide
- **[Building](./docs/building.md)** - Build from source

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

