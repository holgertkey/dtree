#!/bin/bash
# Automated installation script for dtree on Linux
# This script builds the binary, installs it to ~/bin, and sets up the bash wrapper

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  dtree Installation Script for Linux  ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Build the project
echo -e "${YELLOW}[1/5]${NC} Building dtree in release mode..."
if ! cargo build --release; then
    echo -e "${RED}Error: Build failed${NC}" >&2
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# Step 2: Create bin directory and copy binary
echo -e "${YELLOW}[2/5]${NC} Installing binary to ~/bin/dtree..."
mkdir -p "$HOME/bin"
cp target/release/dtree "$HOME/bin/dtree"
chmod +x "$HOME/bin/dtree"
echo -e "${GREEN}✓ Binary installed to $HOME/bin/dtree${NC}"
echo ""

# Step 3: Check if ~/bin is in PATH
echo -e "${YELLOW}[3/5]${NC} Checking PATH configuration..."
if [[ ":$PATH:" != *":$HOME/bin:"* ]]; then
    echo -e "${YELLOW}Warning: $HOME/bin is not in your PATH${NC}"

    # Detect shell config file
    if [ -n "$BASH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    else
        SHELL_CONFIG="$HOME/.bashrc"
    fi

    echo -e "Adding PATH export to $SHELL_CONFIG..."
    echo "" >> "$SHELL_CONFIG"
    echo "# Added by dtree installer" >> "$SHELL_CONFIG"
    echo 'export PATH="$HOME/bin:$PATH"' >> "$SHELL_CONFIG"
    echo -e "${GREEN}✓ PATH updated in $SHELL_CONFIG${NC}"
    echo -e "${YELLOW}  Please run: source $SHELL_CONFIG${NC}"
else
    echo -e "${GREEN}✓ $HOME/bin is already in PATH${NC}"
fi
echo ""

# Step 4: Install bash wrapper function
echo -e "${YELLOW}[4/5]${NC} Installing bash wrapper function..."

# Detect shell config file
if [ -n "$BASH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
elif [ -n "$ZSH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
else
    SHELL_CONFIG="$HOME/.bashrc"
fi

# Check if wrapper already exists
if grep -q "^dt()" "$SHELL_CONFIG" 2>/dev/null; then
    echo -e "${YELLOW}Warning: dt() wrapper already exists in $SHELL_CONFIG${NC}"
    read -p "Do you want to replace it? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Skipping wrapper installation${NC}"
    else
        # Remove old wrapper (between function definition and closing brace)
        sed -i '/^dt()/,/^}/d' "$SHELL_CONFIG"
        echo -e "${GREEN}✓ Old wrapper removed${NC}"
    fi
fi

# Add wrapper function if it doesn't exist or was removed
if ! grep -q "^dt()" "$SHELL_CONFIG" 2>/dev/null; then
    cat >> "$SHELL_CONFIG" << 'EOF'

# dtree wrapper function - enables directory navigation
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
EOF
    echo -e "${GREEN}✓ Wrapper function added to $SHELL_CONFIG${NC}"
    echo -e "${YELLOW}  Please run: source $SHELL_CONFIG${NC}"
else
    echo -e "${BLUE}Wrapper function already exists${NC}"
fi
echo ""

# Step 5: Test installation
echo -e "${YELLOW}[5/5]${NC} Testing installation..."

if [ -x "$HOME/bin/dtree" ]; then
    VERSION=$("$HOME/bin/dtree" --version 2>&1)
    echo -e "${GREEN}✓ Binary is executable${NC}"
    echo -e "  Version: $VERSION"
else
    echo -e "${RED}Error: Binary is not executable${NC}" >&2
    exit 1
fi
echo ""

# Installation summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Installation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "Installation summary:"
echo -e "  ${GREEN}✓${NC} Binary: $HOME/bin/dtree"
echo -e "  ${GREEN}✓${NC} Wrapper: $SHELL_CONFIG"
echo -e "  ${GREEN}✓${NC} Config: ~/.config/dtree/config.toml (will be created on first run)"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo -e "  1. Reload your shell configuration:"
echo -e "     ${BLUE}source $SHELL_CONFIG${NC}"
echo -e "  2. Test the installation:"
echo -e "     ${BLUE}dt --version${NC}"
echo -e "     ${BLUE}dt${NC}  (opens interactive TUI)"
echo -e "  3. Create your first bookmark:"
echo -e "     ${BLUE}dt -bm add myproject /path/to/project${NC}"
echo ""
echo -e "For help, run: ${BLUE}dtree -h${NC}"
echo ""
