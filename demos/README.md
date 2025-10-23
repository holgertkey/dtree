# dtree Demo GIFs

This directory contains VHS tape files for generating animated GIF demonstrations.

## Quick Start

### 0. Prerequisites

**Ensure dtree is installed and in PATH:**

```bash
# Build dtree if not already built
cd /home/holger/github.com/holgertkey/dtree
cargo build --release

# Install to ~/bin
cp target/release/dtree ~/bin/

# Verify installation
which dtree  # Should show: /home/holger/bin/dtree
dtree --version  # Should show version
```

### 1. Install VHS

```bash
# Install VHS (Terminal GIF Generator)
go install github.com/charmbracelet/vhs@latest

# Install ffmpeg (required by VHS)
sudo apt install ffmpeg  # Ubuntu/Debian
brew install ffmpeg      # macOS
```

**Alternative installation methods:**
- Arch Linux: `sudo pacman -S vhs`
- macOS: `brew install vhs`

More info: https://github.com/charmbracelet/vhs

### 2. Generate All Demos

```bash
cd demos
./generate_all.sh
```

This will create all GIF files in `docs/assets/`:
- `tree_navigation.gif` - Basic navigation
- `file_viewer.gif` - File preview and fullscreen mode
- `visual_selection.gif` - Visual selection mode
- `search.gif` - Search with fuzzy matching
- `bookmarks.gif` - Bookmark management

### 3. Generate Individual Demo

```bash
vhs tree_navigation.tape
vhs file_viewer.tape
vhs visual_selection.tape
vhs search.tape
vhs bookmarks.tape
```

## Files

- `*.tape` - VHS tape files (scripts for recordings)
- `generate_all.sh` - Batch generator script
- `record_demos.md` - Detailed recording guide

## Customization

Edit `.tape` files to change:
- **Timing**: `Sleep 500ms` controls pauses
- **Size**: `Set Width 1200` and `Set Height 600`
- **Theme**: `Set Theme "Catppuccin Mocha"`
- **Speed**: `Set PlaybackSpeed 1.0`
- **Font**: `Set FontSize 16`
- **Hide commands**: Use `Hide` and `Show` to skip boring setup
  ```
  Hide
  Type "cd /path && dtree"
  Enter
  Sleep 1s
  Show
  # Demo starts with dtree already visible
  ```

Available themes: https://github.com/charmbracelet/vhs#themes

## Tips

1. **Keep demos short**: 10-15 seconds is ideal
2. **Hide setup**: Use `Hide`/`Show` to skip cd and launch commands
3. **Clear actions**: Add pauses (`Sleep`) before key actions
4. **Test locally**: View GIFs before committing
5. **Optimize size**: Use `gifsicle` to reduce file size:
   ```bash
   gifsicle -O3 --lossy=80 input.gif -o output.gif
   ```

## Manual Recording

If you prefer manual recording with asciinema:

```bash
# Install
pip install asciinema
cargo install --git https://github.com/asciinema/agg

# Record
asciinema rec demo.cast
# ... use dtree ...
# Press Ctrl+D when done

# Convert to GIF
agg demo.cast demo.gif
```

## Troubleshooting

**"dtree: command not found" in GIFs:**
- **Solution:** Use ABSOLUTE PATH in tape files!
- All tape files now use: `Type "/home/holger/bin/dtree"` instead of `Type "dtree"`
- VHS runs in isolated shell and doesn't always respect PATH
- Make sure dtree is installed: `which dtree`
- Install to ~/bin: `cp target/release/dtree ~/bin/`

**Key Learning:** VHS's `Env PATH` directive is not reliable. Always use absolute paths for executables.

**VHS not found:**
- Make sure `$GOPATH/bin` is in your `$PATH`
- Default: `~/go/bin`
- Add to ~/.bashrc: `export PATH="$HOME/go/bin:$PATH"`

**ffmpeg errors:**
- Install ffmpeg: `sudo apt install ffmpeg`
- Or on macOS: `brew install ffmpeg`

**Fonts look bad:**
- Install a Nerd Font (e.g., JetBrains Mono Nerd Font)
- Set in terminal preferences
- Download from: https://www.nerdfonts.com/

**GIFs too large:**
- Reduce demo duration (keep under 15 seconds)
- Optimize with gifsicle: `gifsicle -O3 --lossy=80 input.gif -o output.gif`
- Reduce Width/Height in tape file

## More Info

See `record_demos.md` for detailed instructions and best practices.
