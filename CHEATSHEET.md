# dtree Cheat Sheet

Quick reference guide for dtree - Directory Tree Navigator

---

## Installation

```bash
cargo build --release
cp target/release/dtree ~/bin/
```

Add to `~/.bashrc`:
```bash
dt() {
  local result=$(command dtree "$@")
  [ -n "$result" ] && [ -d "$result" ] && cd "$result"
}
```

---

## Command Line

```bash
dt                          # Open tree navigator
dt /path/to/directory       # Navigate to directory
dt myproject                # Jump to bookmark
dt -                        # Return to previous directory
dt -v file.txt              # View file in fullscreen
dt -bm list                 # List bookmarks
dt -bm add name [path]      # Add bookmark
dt -bm remove name          # Remove bookmark
dtree --help                # Show help
dtree --version             # Show version
```

---

## Tree Navigation

| Key | Action |
|-----|--------|
| `j` `↓` | Move down |
| `k` `↑` | Move up |
| `l` `→` | Expand directory |
| `h` `←` | Collapse directory |
| `Enter` | Enter directory (change root) |
| `u` `Backspace` | Go to parent directory |
| `q` | Exit and cd to selected directory |
| `Esc` | Exit without cd |

---

## File Viewing

### Split View Mode

| Key | Action |
|-----|--------|
| `s` | Toggle split view on/off |
| `Ctrl+j` | Scroll down by line |
| `Ctrl+k` | Scroll up by line |
| `PgDn` | Scroll down by page |
| `PgUp` | Scroll up by page |
| `Home` | Jump to start of file |
| `End` | Jump to end of file |

### Fullscreen Mode

| Key | Action |
|-----|--------|
| `v` | Enter fullscreen (from tree) |
| `j` `k` `↓` `↑` | Scroll by line |
| `Ctrl+j` `Ctrl+k` | Next/previous file |
| `PgDn` `PgUp` | Scroll by page |
| `Home` | HEAD mode (first 10K lines) |
| `End` | TAIL mode (last 10K lines) |
| `l` | Toggle line numbers |
| `/` | Search within file |
| `n` `N` | Next/previous match |
| `q` | Back to tree view |
| `Esc` | Exit program |

---

## Search

### Tree Search

| Key | Action |
|-----|--------|
| `/` | Enter search mode |
| `Type` | Add to query |
| `Backspace` | Delete character |
| `Enter` | Execute search |
| `Tab` | Switch tree ↔ results |
| `j` `k` | Navigate results |
| `Enter` | Jump to result |
| `Esc` | Close search |

**Fuzzy**: Start with `/` (e.g., `/fuz` finds `fuzzy.rs`)

### File Search (Fullscreen)

| Key | Action |
|-----|--------|
| `/` | Enter search |
| `Enter` | Execute → jump to first |
| `n` | Next match |
| `N` | Previous match |
| `Esc` | Clear search |

---

## Bookmarks

### Interactive Mode

| Key | Action |
|-----|--------|
| `m` | Create bookmark |
| `'` | Open bookmark menu |
| `j` `k` | Navigate list |
| `Tab` | Toggle filter mode |
| `d` | Delete (press twice) |
| `Enter` | Jump to bookmark |
| `Esc` | Close menu |

### CLI Mode

```bash
dt -bm                      # List all
dt -bm add work             # Add current dir
dt -bm add work /path       # Add specific path
dt -bm remove work          # Delete
dt work                     # Jump to bookmark
```

---

## File Operations

| Key | Action |
|-----|--------|
| `e` | Open in editor (text) or hex editor (binary) |
| `o` | Open in file manager |
| `c` | Copy path to clipboard |

---

## Other

| Key | Action |
|-----|--------|
| `i` | Toggle help screen |
| `z` | Toggle directory sizes |

---

## Mouse

| Action | Effect |
|--------|--------|
| **Click** | Select item |
| **Double-click** | Expand/collapse |
| **Scroll** | Navigate/scroll |
| **Drag divider** | Resize panels |
| **Shift+Drag** | Select text (fullscreen) |

---

## Configuration

File: `~/.config/dtree/config.toml`

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
show_hidden = false
editor = "nvim"
hex_editor = "hexyl"
file_manager = "ranger"
```

---

## Common Workflows

### Quick Navigation
```bash
dt                      # Open tree
# Navigate with j/k, expand with l
# Press Enter to enter directory
# Press q to exit and cd
```

### Search and Jump
```bash
dt                      # Open tree
# Press / to search
# Type 'src', press Enter
# Navigate results with j/k
# Press Enter to jump
```

### Bookmark Workflow
```bash
dt ~/projects/myapp     # Navigate to project
# Press m, type 'myapp', Enter
# Press q to exit
dt myapp                # Instant jump!
```

### File Inspection
```bash
dt -v /var/log/syslog   # Open log
# Press End for tail mode
# Press / to search
# Type 'error', Enter
# Press n for next match
```

### Edit Workflow
```bash
dt ~/projects           # Open tree
# Press s for split view
# Navigate to file with j/k
# Press e to open in editor
```

---

## Tips & Tricks

**Speed Navigation**
- Use bookmarks for frequent locations
- Use `dt -` to toggle between two directories
- Double-click to expand/collapse quickly

**Large Files**
- Use `End` for tail mode on logs
- Use `Home` to return to head mode
- Search with `/` to find specific content

**Binary Files**
- Automatically detected
- Press `e` to open in hex editor
- Configure `hex_editor` in config

**Customization**
- Edit `~/.config/dtree/config.toml`
- Choose syntax theme (20+ available)
- Customize all colors
- Set preferred editor/file manager

**Performance**
- Use `z` to see directory sizes
- Large directories load incrementally
- Search runs in background (non-blocking)

---

## Troubleshooting

**Command not found**
```bash
# Add to ~/.bashrc and reload
source ~/.bashrc
```

**Colors not working**
```bash
# Check terminal supports 256 colors
echo $TERM
# Should be: xterm-256color or similar
```

**Icons not showing**
```bash
# Install Nerd Fonts
# Or set show_icons = false in config
```

**Editor won't open**
```bash
# Set in config.toml:
editor = "nano"  # or vim, nvim, etc.
```

---

## Quick Start

```bash
# 1. Install
cargo build --release && cp target/release/dtree ~/bin/

# 2. Add wrapper to ~/.bashrc
dt() { local r=$(dtree "$@"); [ -d "$r" ] && cd "$r"; }

# 3. Reload shell
source ~/.bashrc

# 4. Use!
dt                      # Open navigator
dt ~/projects           # Jump to directory
dt -bm add myproject    # Save bookmark
dt myproject            # Jump to bookmark
```

---

**Documentation**: https://github.com/holgertkey/dtree/tree/main/docs
**License**: MIT
**Built with**: Rust + Ratatui
