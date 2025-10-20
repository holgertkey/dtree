# Installation

This guide covers different ways to install dtree on your system.

## Prerequisites

### Required

- **Rust toolchain** (for building from source)
  - Version 1.70 or later
  - Install from [rustup.rs](https://rustup.rs/)

### Optional

- **Nerd Fonts** (for file type icons)
  - Download from [nerdfonts.com](https://www.nerdfonts.com/)
  - Install a font like `FiraCode Nerd Font` or `JetBrains Mono Nerd Font`
  - Configure your terminal to use the font
  - Enable in config: `show_icons = true`

- **hexyl** (for binary file viewing)
  ```bash
  cargo install hexyl
  ```

## Installation Methods

### Method 1: Build from Source (Recommended)

This is currently the primary installation method:

```bash
# Clone the repository
git clone https://github.com/holgertkey/dtree.git
cd dtree

# Build the release binary
cargo build --release

# Install to user's bin directory
mkdir -p ~/bin
cp target/release/dtree ~/bin/

# Or install system-wide
sudo cp target/release/dtree /usr/local/bin/
```

Make sure `~/bin` is in your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/bin:$PATH"
```

### Method 2: cargo install (Future)

Once published to crates.io:

```bash
cargo install dtree
```

### Method 3: Package Manager (Future)

Package manager support is planned for:

- **Arch Linux**: AUR package
- **Homebrew**: `brew install dtree`
- **apt/deb**: Debian package
- **rpm**: Fedora package

## Bash Integration

For the best experience, add the `dt` wrapper function to your `~/.bashrc`:

```bash
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

  # If flags or bookmark commands are passed, just run dtree directly without cd
  case "$1" in
    -h|--help|--version)
      command dtree "$@"
      return
      ;;
    -bm)
      # Bookmark management commands - run directly
      command dtree "$@"
      return
      ;;
  esac

  # For navigation: delegate path/bookmark resolution to dtree
  # Capture stdout (path) separately from stderr (errors/warnings)
  local result=$(command dtree "$@")
  local exit_code=$?

  if [ $exit_code -ne 0 ]; then
    # dtree already printed error to stderr
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

After adding, reload your shell:

```bash
source ~/.bashrc
```

## Verification

Test your installation:

```bash
# Check version
dtree --version

# Launch from current directory
dt

# View a file
dt -v README.md

# Show help
dtree -h
```

## Post-Installation Configuration

On first launch, dtree creates a configuration file:

```
~/.config/dtree/config.toml
```

You can customize:

- Colors and themes
- Default editor and file manager
- File preview limits
- Keybindings
- Icon display

See [Configuration](./configuration.md) for details.

## Updating

### From Source

```bash
cd dtree
git pull
cargo build --release
cp target/release/dtree ~/bin/  # or /usr/local/bin/
```

### Using cargo (Future)

```bash
cargo install --force dtree
```

## Uninstallation

### Remove Binary

```bash
# User installation
rm ~/bin/dtree

# System installation
sudo rm /usr/local/bin/dtree
```

### Remove Configuration

```bash
rm -rf ~/.config/dtree
```

### Remove Bash Integration

Remove the `dt()` function from your `~/.bashrc`.

## Troubleshooting Installation

### Rust Not Found

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

### Build Errors

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

### Binary Not Found After Installation

Check your PATH:

```bash
echo $PATH

# Add ~/bin to PATH if not present
echo 'export PATH="$HOME/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Icons Not Showing

1. Install a Nerd Font
2. Configure your terminal to use it
3. Enable in config: `show_icons = true`
4. Restart your terminal

### Clipboard Not Working

Install xclip (Linux) or use native clipboard:

```bash
# Ubuntu/Debian
sudo apt install xclip

# Arch
sudo pacman -S xclip

# Fedora
sudo dnf install xclip
```

## Platform-Specific Notes

### Linux

Tested on:
- Ubuntu 20.04+
- Arch Linux
- Fedora 35+

### macOS

Should work but untested. Potential issues:
- Clipboard integration may need adjustments
- Terminal compatibility

### Windows

Not currently supported. WSL recommended:

```bash
# In WSL
cargo build --release
cp target/release/dtree ~/bin/
```

## Dependencies

dtree has minimal runtime dependencies:

- **Terminal with ANSI color support**
- **UTF-8 locale**
- **Mouse support** (optional, for mouse features)

Most modern terminals support these out of the box.

## Next Steps

- [Getting Started](./getting-started.md) - Quick start guide
- [Configuration](./configuration.md) - Customize dtree
- [Bash Integration](./bash-integration.md) - Advanced shell integration
