# Configuration

dtree uses a TOML configuration file that is automatically created on first launch.

## Configuration File Location

```
~/.config/dtree/config.toml
```

## Configuration Structure

The configuration file has three main sections:

1. **`[appearance]`** - Visual settings (colors, icons, layout)
2. **`[behavior]`** - Functional settings (limits, external programs)
3. **`[keybindings]`** - Keyboard shortcuts (currently not fully customizable)

## Default Configuration

On first launch, dtree creates this configuration:

```toml
[appearance]
# UI split position (percentage from left, 20-80)
split_position = 50

# Enable file type icons (requires Nerd Fonts)
show_icons = false

# Show line numbers in fullscreen viewer by default
show_line_numbers = false

# Enable syntax highlighting in file preview
enable_syntax_highlighting = true

# Syntax highlighting theme
# Options: base16-ocean.dark, base16-ocean.light, InspiredGitHub,
#          Solarized (dark), Solarized (light), etc.
syntax_theme = "base16-ocean.dark"

[appearance.colors]
# Color for selected item (name, RGB hex, or 0-255)
selected_color = "cyan"

# Color for directories
directory_color = "blue"

# Color for files
file_color = "white"

# Color for borders and UI elements
border_color = "gray"

# Color for error messages
error_color = "red"

# Color for highlighted text (search results, etc.)
highlight_color = "yellow"

[behavior]
# Maximum lines to load from files (performance limit)
max_file_lines = 10000

# Show hidden files (dotfiles) by default
show_hidden = false

# Follow symbolic links when traversing
follow_symlinks = false

# Mouse double-click timeout in milliseconds
double_click_timeout_ms = 500

# External editor command (for 'e' key)
editor = "nano"

# External hex editor for binary files
hex_editor = "hexyl"

# External file manager command (for 'o' key)
file_manager = "mc"

[keybindings]
# Note: Custom keybindings not yet fully implemented
# These are documentation of default bindings

quit = ["q", "Esc"]
search = ["/"]
toggle_files = ["s"]
toggle_help = ["i"]
copy_path = ["c"]
```

## Appearance Settings

### Colors

dtree supports three color formats:

#### 1. Color Names

```toml
selected_color = "cyan"
directory_color = "blue"
file_color = "white"
```

Available names:
- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `gray`, `light_red`, `light_green`, `light_yellow`, `light_blue`, `light_magenta`, `light_cyan`

#### 2. RGB Hex Colors

```toml
selected_color = "#00FFFF"
directory_color = "#0000FF"
error_color = "#FF0000"
```

#### 3. Indexed Colors (0-255)

```toml
selected_color = "51"   # Cyan
directory_color = "39"  # Blue
border_color = "240"    # Dark gray
```

See [256 color chart](https://www.ditig.com/256-colors-cheat-sheet) for reference.

### Split Position

Controls the default position of the vertical divider:

```toml
split_position = 50  # Middle (default)
split_position = 30  # Narrower tree, wider preview
split_position = 70  # Wider tree, narrower preview
```

Valid range: 20-80

### File Icons

Enable Nerd Font icons for file types:

```toml
show_icons = true
```

Requirements:
- Nerd Font installed
- Terminal configured to use Nerd Font
- Font supports required glyphs

If icons don't display correctly, set to `false` for emoji fallback.

### Line Numbers

Show line numbers by default in fullscreen viewer:

```toml
show_line_numbers = true
```

You can always toggle with `l` key in fullscreen mode.

### Syntax Highlighting

Enable or disable syntax highlighting:

```toml
enable_syntax_highlighting = true
```

Choose a theme:

```toml
syntax_theme = "base16-ocean.dark"      # Dark theme (default)
syntax_theme = "base16-ocean.light"     # Light theme
syntax_theme = "InspiredGitHub"         # GitHub-style
syntax_theme = "Solarized (dark)"       # Solarized dark
syntax_theme = "Solarized (light)"      # Solarized light
```

Available themes depend on your syntect installation. The default themes work out of the box.

## Behavior Settings

### File Preview Limit

Control how many lines to load from files:

```toml
max_file_lines = 10000  # Default
max_file_lines = 5000   # Faster for large files
max_file_lines = 50000  # More content (slower)
```

For files exceeding this limit, dtree shows HEAD mode (first N lines) or TAIL mode (last N lines).

### Show Hidden Files

Include dotfiles in the tree by default:

```toml
show_hidden = true
```

This affects the initial state. You cannot currently toggle this at runtime.

### Follow Symlinks

Follow symbolic links when traversing directories:

```toml
follow_symlinks = true
```

**Warning**: Enabling this can cause infinite loops with circular symlinks.

### Mouse Timing

Adjust double-click detection:

```toml
double_click_timeout_ms = 500  # Default (0.5 seconds)
double_click_timeout_ms = 300  # Faster double-click
double_click_timeout_ms = 800  # Slower double-click
```

### External Programs

#### Editor

Set your preferred text editor:

```toml
editor = "nano"      # Default
editor = "vim"       # Vim
editor = "nvim"      # Neovim
editor = "emacs -nw" # Emacs (no window)
editor = "micro"     # Micro
```

Used when pressing `e` on a text file.

#### Hex Editor

Set your preferred hex editor for binary files:

```toml
hex_editor = "hexyl"    # Default (cargo install hexyl)
hex_editor = "xxd"      # Traditional xxd
hex_editor = "hd"       # Hex dump
hex_editor = "hexdump"  # Hexdump
```

Used when pressing `e` on a binary file.

#### File Manager

Set your preferred file manager:

```toml
file_manager = "mc"      # Midnight Commander (default)
file_manager = "ranger"  # Ranger
file_manager = "nnn"     # NNN
file_manager = "lf"      # LF
file_manager = "vifm"    # Vifm
```

Used when pressing `o` on a file or directory.

## Resetting Configuration

To reset to defaults, simply delete the config file:

```bash
rm ~/.config/dtree/config.toml
```

It will be recreated on next launch.

## Per-Project Configuration

dtree does not currently support per-project configuration files. All settings are global.

## Environment Variables

dtree respects these environment variables:

- `EDITOR` - Fallback if `editor` not set in config (not implemented yet)
- `TERM` - Terminal type detection
- `COLORTERM` - True color support detection

## Configuration Examples

### Minimal Setup

```toml
[appearance]
split_position = 50

[behavior]
max_file_lines = 10000
editor = "nano"
file_manager = "mc"
```

### Dark Theme with Nerd Fonts

```toml
[appearance]
split_position = 40
show_icons = true
enable_syntax_highlighting = true
syntax_theme = "base16-ocean.dark"

[appearance.colors]
selected_color = "#00FFFF"
directory_color = "#569CD6"
file_color = "#D4D4D4"
border_color = "#404040"
error_color = "#F44747"
highlight_color = "#FFFF00"

[behavior]
max_file_lines = 10000
editor = "nvim"
hex_editor = "hexyl"
file_manager = "ranger"
```

### Light Theme

```toml
[appearance]
enable_syntax_highlighting = true
syntax_theme = "InspiredGitHub"

[appearance.colors]
selected_color = "#0000FF"
directory_color = "#0000FF"
file_color = "#000000"
border_color = "#CCCCCC"
error_color = "#FF0000"
highlight_color = "#FFFF00"
```

### Performance-Optimized

```toml
[appearance]
split_position = 50
show_icons = false
enable_syntax_highlighting = false

[behavior]
max_file_lines = 5000
show_hidden = false
follow_symlinks = false
```

## Troubleshooting Configuration

### Config File Not Created

Run dtree once to create it:

```bash
dt
# Press Esc to exit
```

### Invalid TOML Syntax

dtree will show an error if the TOML is malformed. Check:

- Matching quotes
- Valid color formats
- Correct section names
- No duplicate keys

### Colors Not Working

- Check terminal supports colors: `echo $TERM`
- Try simpler color names first
- Use indexed colors for maximum compatibility

### Editor/File Manager Not Found

dtree validates that the command exists. If not found:

```bash
# Check if command exists
which nano
which mc

# Install if missing
sudo apt install nano mc
```

Or change to an installed program in the config.

## Next Steps

- [Key Bindings](./keybindings.md) - Complete keybinding reference
- [Features](./features.md) - Explore all features
- [Troubleshooting](./troubleshooting.md) - Common issues
