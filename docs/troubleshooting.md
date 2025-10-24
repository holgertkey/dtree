# Troubleshooting

Common issues and their solutions.

## Installation Issues

### Rust Not Found

**Problem**: `cargo: command not found`

**Solution**:
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
cargo --version
```

### Build Errors

**Problem**: Compilation fails with errors

**Solution**:
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

**Problem**: Missing system dependencies

**Solution** (Linux):
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install build-essential

# Arch
sudo pacman -S base-devel

# Fedora
sudo dnf groupinstall "Development Tools"
```

### Binary Not Found After Installation

**Problem**: `dt: command not found`

**Solution**:
```bash
# Check if ~/bin is in PATH
echo $PATH | grep $HOME/bin

# If not, add to ~/.bashrc
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify binary exists
ls -l ~/bin/dtree
```

## Runtime Issues

### Terminal Issues

**Problem**: Strange characters or broken UI

**Solutions**:

1. Check terminal supports UTF-8:
   ```bash
   echo $LANG  # Should show UTF-8
   # If not:
   export LANG=en_US.UTF-8
   ```

2. Check terminal supports colors:
   ```bash
   echo $TERM  # Should be xterm-256color or similar
   ```

3. Try a different terminal:
   - Alacritty (recommended)
   - Kitty
   - WezTerm
   - iTerm2 (macOS)

**Problem**: Mouse doesn't work

**Solution**:
1. Check terminal supports mouse:
   ```bash
   # Most modern terminals do, but some don't
   ```

2. Enable mouse in terminal settings

3. Try keyboard-only mode (all features work without mouse)

**Problem**: Colors look wrong

**Solutions**:

1. Check terminal color support:
   ```bash
   echo $COLORTERM  # Should show "truecolor" for best results
   ```

2. Try different color scheme in config:
   ```toml
   [appearance.colors]
   selected_color = "51"    # Use indexed colors instead
   directory_color = "39"
   ```

3. Switch to light theme:
   ```toml
   syntax_theme = "InspiredGitHub"
   ```

### File Icons Not Showing

**Problem**: Icons show as boxes or question marks

**Solutions**:

1. Install a Nerd Font:
   ```bash
   # Download from https://www.nerdfonts.com/
   # Install "FiraCode Nerd Font" or "JetBrains Mono Nerd Font"
   ```

2. Configure terminal to use the font

3. Restart terminal

4. If still not working, disable icons:
   ```toml
   [appearance]
   show_icons = false
   ```

### Syntax Highlighting Not Working

**Problem**: No colors in file preview

**Solutions**:

1. Check it's enabled:
   ```toml
   [appearance]
   enable_syntax_highlighting = true
   ```

2. Try different theme:
   ```toml
   syntax_theme = "base16-ocean.dark"
   ```

3. Check file extension is recognized

4. Some files may not have language support

### Clipboard Not Working

**Problem**: `c` key doesn't copy path

**Solutions**:

1. Install clipboard tool (Linux):
   ```bash
   # Ubuntu/Debian
   sudo apt install xclip

   # Arch
   sudo pacman -S xclip

   # Fedora
   sudo dnf install xclip
   ```

2. Check xclip works:
   ```bash
   echo "test" | xclip -selection clipboard
   xclip -selection clipboard -o
   ```

3. Alternative: Manually copy from terminal (select text)

## Performance Issues

### Slow Startup

**Problem**: Takes long to start

**Solutions**:

1. Large directory:
   - dtree only loads visible nodes
   - But initial scan can be slow for huge directories
   - Try starting from smaller subdirectory

2. Slow filesystem:
   - Network mounts can be slow
   - Use local directories when possible

### Slow Navigation

**Problem**: Lag when expanding directories

**Solutions**:

1. Disable size calculation:
   - Don't press `z` for huge directories
   - Size calculation can be slow

2. Disable file viewer:
   - Press `s` to toggle off
   - Reduces I/O load

3. Reduce file preview limit:
   ```toml
   [behavior]
   max_file_lines = 5000  # Lower limit
   ```

### High Memory Usage

**Problem**: dtree uses a lot of RAM

**Solutions**:

1. Close file viewer mode (`s`)

2. Don't expand too many directories at once

3. Reduce file preview limit in config

4. This is normal for very large trees (100K+ files)

## Feature Issues

### Search Not Working

**Problem**: Search finds no results

**Solutions**:

1. Check search scope:
   - Normal mode: searches directories only
   - File viewer mode (`s`): searches files + directories
   - Enable file viewer if searching for files

2. Check query syntax:
   - `/query` = fuzzy search
   - `query` = exact substring

3. Case sensitivity:
   - All searches are case-insensitive
   - "TEST" finds "test", "Test", "TEST"

**Problem**: Search is slow

**Solutions**:

1. This is normal for large trees (100K+ files)

2. Phase 1 (quick) is instant, Phase 2 (deep) runs in background

3. Cancel with `Esc` if taking too long

4. Use more specific query to reduce results

### Bookmarks Not Saving

**Problem**: Bookmarks disappear after restart

**Solutions**:

1. Check config directory:
   ```bash
   ls ~/.config/dtree/bookmarks.json
   ```

2. Check file permissions:
   ```bash
   chmod 644 ~/.config/dtree/bookmarks.json
   ```

3. Check disk space:
   ```bash
   df -h ~
   ```

4. Try creating manually:
   ```bash
   mkdir -p ~/.config/dtree
   echo '[]' > ~/.config/dtree/bookmarks.json
   ```

### External Editor Won't Open

**Problem**: `e` key doesn't open editor

**Solutions**:

1. Check editor exists:
   ```bash
   which nano  # Or your configured editor
   ```

2. Install editor if missing:
   ```bash
   sudo apt install nano  # Ubuntu/Debian
   ```

3. Configure different editor:
   ```toml
   [behavior]
   editor = "vim"  # Or "nvim", "emacs -nw", etc.
   ```

4. Check PATH:
   ```bash
   echo $PATH
   ```

### Terminal Artifacts

**Problem**: Strange characters or escape sequences appear after exiting dtree (e.g., `35;64;18M`)

**Solution**:
- This issue was completely resolved in v0.1.0+ with improved terminal cleanup
- Update to the latest version if you're experiencing this
- The fix includes:
  - Explicit disabling of all mouse tracking modes
  - Aggressive event draining during cleanup
  - Proper timing for terminal state transitions
- If still occurs, report as bug with details about your terminal emulator

**Technical Details** (for developers):
The artifacts were SGR mouse events leaking after terminal resize in split view mode. Fixed by:
1. Disabling all 6 mouse tracking modes explicitly (X10, cell motion, all motion, SGR, urxvt, plus crossterm's DisableMouseCapture)
2. Double-draining events: once after disabling mouse, once after leaving alternate screen
3. Proper delays (20ms + 10ms) to allow terminal processing
4. Minimal reset sequences without aggressive screen clearing

**Problem**: Terminal artifacts when exiting fullscreen viewer

**Solution**:
- Same fix as above applies to fullscreen mode
- Update to latest version
- If still occurs, report as bug

**Problem**: Line numbers not aligned

**Solution**:
- This is a font issue
- Use a monospace font
- Nerd Fonts are recommended

**Problem**: Can't exit fullscreen

**Solution**:
- `q` returns to tree view
- `Esc` exits program completely
- Check keyboard layout

## Configuration Issues

### Config File Not Created

**Problem**: No config file at `~/.config/dtree/config.toml`

**Solution**:
```bash
# Run dtree once to create it
dt
# Press Esc to exit

# Verify it was created
cat ~/.config/dtree/config.toml
```

### Config Changes Not Applied

**Problem**: Changes to config.toml don't work

**Solutions**:

1. Restart dtree after config changes

2. Check TOML syntax:
   ```bash
   # Invalid TOML will be rejected
   # Check for syntax errors
   ```

3. Check config file location:
   ```bash
   ~/.config/dtree/config.toml  # Correct
   ~/dtree/config.toml          # Wrong
   ```

4. Reset to defaults:
   ```bash
   rm ~/.config/dtree/config.toml
   dt  # Will recreate with defaults
   ```

### Colors Not Changing

**Problem**: Color settings don't work

**Solutions**:

1. Check format:
   ```toml
   selected_color = "cyan"      # Color name
   selected_color = "#00FFFF"   # Hex
   selected_color = "51"        # Indexed
   ```

2. Check terminal color support

3. Try indexed colors (0-255) for maximum compatibility

## Platform-Specific Issues

### Linux

**Problem**: Permission denied errors

**Solution**:
```bash
# Don't run as root
# Fix permissions:
chmod +x ~/bin/dtree
```

### macOS

**Problem**: Clipboard not working

**Solution**:
- macOS clipboard should work out of the box
- If not, try iTerm2 or Alacritty

**Problem**: Editor not opening

**Solution**:
- Check /dev/tty permissions
- Try different terminal

### Windows (WSL)

**Problem**: Installation on Windows

**Solution**:
- Use WSL (Windows Subsystem for Linux)
- Follow Linux installation guide
- Native Windows not currently supported

**Problem**: Clipboard not working in WSL

**Solution**:
- Clipboard between WSL and Windows is limited
- Use terminal text selection instead

## Error Messages

### "Cannot read: Permission denied"

**Problem**: Directory shows ⚠ with permission error

**Solution**:
- This is expected for protected directories
- Run as user, not root (security)
- Some directories (like /root) are inaccessible

### "Directory not found"

**Problem**: Bookmark or path doesn't exist

**Solution**:
1. Check path:
   ```bash
   ls /path/to/directory
   ```

2. Update bookmark:
   ```bash
   dt -bm remove oldname
   dt -bm add newname /new/path
   ```

### "Editor not found"

**Problem**: Configured editor doesn't exist

**Solution**:
```bash
# Install editor
sudo apt install nano

# Or change config
vim ~/.config/dtree/config.toml
# Set: editor = "vim"  # Or other installed editor
```

## Getting More Help

### Debug Information

When reporting issues, include:

```bash
# dtree version
dtree --version

# Rust version
cargo --version
rustc --version

# OS info
uname -a

# Terminal
echo $TERM
echo $COLORTERM

# Shell
echo $SHELL
```

### Reporting Bugs

1. Check [existing issues](https://github.com/holgertkey/dtree/issues)

2. Open new issue with:
   - Clear description
   - Steps to reproduce
   - Expected vs actual behavior
   - Debug information (above)
   - Screenshots if applicable

3. Be patient - maintainers respond when they can

### Community Help

- GitHub Discussions
- GitHub Issues
- Check documentation

## Known Issues

### Current Limitations

- No file operations (copy, move, delete) - use file manager integration
- No Windows native support (use WSL)
- Keybindings not fully customizable yet
- No plugin system yet

### Future Fixes

See [Roadmap](../CLAUDE.md) for planned improvements.

## Quick Fixes Summary

| Problem            | Quick Fix                           |
|--------------------|-------------------------------------|
| Icons broken       | Disable icons: `show_icons = false` |
| Colors wrong       | Use indexed colors in config        |
| Mouse not working  | Use keyboard-only mode              |
| Clipboard broken   | Install xclip (Linux)               |
| Editor won't open  | Check editor in PATH                |
| Config not loaded  | Restart dtree                       |
| Terminal artifacts | Update to latest version            |
| Slow performance   | Disable file viewer mode            |

## Still Having Issues?

If this guide doesn't solve your problem:

1. Search [GitHub Issues](https://github.com/holgertkey/dtree/issues)
2. Open a new issue with details
3. Be patient and provide requested information

We're here to help!
