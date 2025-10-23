# Recording Demos for dtree

This guide explains how to create animated GIF demonstrations for the README.

## Prerequisites

### Install VHS (Recommended)

VHS is a tool for generating terminal GIFs from scripts.

```bash
# Install VHS
go install github.com/charmbracelet/vhs@latest

# Or use package manager
# Arch Linux
sudo pacman -S vhs

# macOS
brew install vhs

# Ensure ffmpeg is installed (required by VHS)
# Ubuntu/Debian
sudo apt install ffmpeg

# macOS
brew install ffmpeg
```

### Alternative: asciinema + agg

```bash
# Install asciinema
pip install asciinema

# Install agg (asciinema to gif converter)
cargo install --git https://github.com/asciinema/agg
```

## Recording Demos with VHS

VHS uses `.tape` files to define recordings. The tape files are in the `demos/` directory.

### Generate all demos

```bash
cd demos
./generate_all.sh
```

### Generate individual demo

```bash
vhs demos/tree_navigation.tape
vhs demos/file_viewer.tape
vhs demos/visual_selection.tape
vhs demos/search.tape
vhs demos/bookmarks.tape
```

## Recording Manually with asciinema

If you prefer manual recording:

```bash
# 1. Record session
asciinema rec demo.cast

# 2. Use dtree and perform actions
# Press Ctrl+D when done

# 3. Convert to GIF
agg demo.cast demo.gif

# Optional: customize GIF
agg --speed 1.5 --cols 80 --rows 24 demo.cast demo.gif
```

## Demo Scenarios

### 1. Tree Navigation (tree_navigation.gif)
- Start dtree
- Navigate with j/k
- Expand/collapse with l/h
- Enter directory with Enter
- Go to parent with u
- Exit with q

### 2. File Viewer (file_viewer.gif)
- Open dtree
- Press 's' to enable split view
- Navigate to files
- Show syntax highlighting
- Scroll with Ctrl+j/k
- Press 'v' for fullscreen
- Toggle line numbers with 'l'
- Exit with 'q'

### 3. Visual Selection (visual_selection.gif)
- Open file in fullscreen (dt -v file)
- Press 'V' to enter visual mode
- Select lines with j/k
- Show visual feedback
- Copy with 'y'
- Exit

### 4. Search (search.gif)
- Open dtree
- Press '/' to search
- Type query
- Show results panel
- Navigate results
- Press Enter to jump
- Show fuzzy search with '/fuz'

### 5. Bookmarks (bookmarks.gif)
- Navigate to directory
- Press 'm' to create bookmark
- Enter bookmark name
- Press q to exit
- Run 'dt bookmarkname' to jump
- Show bookmark list with dt -bm

## GIF Settings

Recommended settings for GIFs:
- **Size**: 80 columns × 24 rows (fits most screens)
- **Speed**: 1.0x-1.5x (not too fast)
- **Loop**: Infinite
- **Theme**: Dark theme with good contrast
- **Font**: Monospace, size 14-16

## File Locations

Place generated GIFs in:
```
docs/assets/
├── tree_navigation.gif
├── file_viewer.gif
├── visual_selection.gif
├── search.gif
└── bookmarks.gif
```

## Tips

1. **Keep it short**: 10-15 seconds per demo
2. **Show real content**: Use actual project files
3. **Clear actions**: Pause briefly before each action
4. **Clean terminal**: Clear screen before recording
5. **No typos**: Script it with VHS for perfection

## Updating README

After generating GIFs, update README.md image links:

```markdown
## Screenshots

### Tree Navigation
![Tree Navigation](docs/assets/tree_navigation.gif)

### File Viewer with Syntax Highlighting
![File Viewer](docs/assets/file_viewer.gif)

### Visual Selection Mode
![Visual Selection](docs/assets/visual_selection.gif)

### Fuzzy Search
![Search](docs/assets/search.gif)

### Bookmarks
![Bookmarks](docs/assets/bookmarks.gif)
```
