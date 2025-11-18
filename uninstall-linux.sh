#!/bin/bash
# Uninstall script for dtree on Linux
# Removes binary, wrapper function, and optionally configuration files

set -e  # Exit on error

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse command line arguments
REMOVE_CONFIG=false
REMOVE_FROM_PATH=false
FORCE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --remove-config)
            REMOVE_CONFIG=true
            shift
            ;;
        --remove-from-path)
            REMOVE_FROM_PATH=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --remove-config      Remove configuration and bookmarks from ~/.config/dtree"
            echo "  --remove-from-path   Remove ~/bin from PATH in shell config"
            echo "  --force             Skip confirmation prompts"
            echo "  -h, --help          Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                           # Remove binary and wrapper only"
            echo "  $0 --remove-config           # Also remove configuration"
            echo "  $0 --remove-config --force   # Remove everything without prompts"
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option: $1${NC}" >&2
            echo "Run '$0 --help' for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  dtree Uninstall Script for Linux     ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Confirm uninstallation unless --force is used
if [ "$FORCE" = false ]; then
    echo -e "${YELLOW}This will remove:${NC}"
    echo -e "  - Binary: ~/bin/dtree"
    echo -e "  - Wrapper function from shell config"
    if [ "$REMOVE_CONFIG" = true ]; then
        echo -e "  - Configuration: ~/.config/dtree"
    fi
    if [ "$REMOVE_FROM_PATH" = true ]; then
        echo -e "  - PATH entry for ~/bin"
    fi
    echo ""
    read -p "Continue with uninstallation? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${BLUE}Uninstallation cancelled${NC}"
        exit 0
    fi
fi

# Detect shell config file
if [ -n "$BASH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.bashrc"
elif [ -n "$ZSH_VERSION" ]; then
    SHELL_CONFIG="$HOME/.zshrc"
else
    SHELL_CONFIG="$HOME/.bashrc"
fi

# Step 1: Remove binary
echo -e "${YELLOW}[1/4]${NC} Removing binary..."
if [ -f "$HOME/bin/dtree" ]; then
    rm "$HOME/bin/dtree"
    echo -e "${GREEN}✓ Removed $HOME/bin/dtree${NC}"
else
    echo -e "${BLUE}Binary not found (already removed)${NC}"
fi
echo ""

# Step 2: Remove wrapper function from shell config
echo -e "${YELLOW}[2/4]${NC} Removing wrapper function..."
if [ -f "$SHELL_CONFIG" ] && grep -q "^dt()" "$SHELL_CONFIG" 2>/dev/null; then
    # Create backup
    cp "$SHELL_CONFIG" "$SHELL_CONFIG.dtree.backup"

    # Remove wrapper function (from "dt()" to closing brace and empty line after)
    sed -i '/^# dtree wrapper function/,/^}$/d' "$SHELL_CONFIG"

    # Also remove the empty line before the comment if it exists
    sed -i '/^$/N;/^\n# dtree wrapper function/d' "$SHELL_CONFIG"

    echo -e "${GREEN}✓ Removed wrapper function from $SHELL_CONFIG${NC}"
    echo -e "${BLUE}  Backup saved to $SHELL_CONFIG.dtree.backup${NC}"
else
    echo -e "${BLUE}Wrapper function not found (already removed)${NC}"
fi
echo ""

# Step 3: Remove PATH entry (optional)
echo -e "${YELLOW}[3/4]${NC} Checking PATH configuration..."
if [ "$REMOVE_FROM_PATH" = true ]; then
    if [ -f "$SHELL_CONFIG" ] && grep -q "export PATH=\"\$HOME/bin:\$PATH\"" "$SHELL_CONFIG" 2>/dev/null; then
        # Create backup if not already created
        if [ ! -f "$SHELL_CONFIG.dtree.backup" ]; then
            cp "$SHELL_CONFIG" "$SHELL_CONFIG.dtree.backup"
        fi

        # Remove PATH export and comment
        sed -i '/# Added by dtree installer/d' "$SHELL_CONFIG"
        sed -i '/export PATH="\$HOME\/bin:\$PATH"/d' "$SHELL_CONFIG"

        echo -e "${GREEN}✓ Removed PATH entry from $SHELL_CONFIG${NC}"
    else
        echo -e "${BLUE}PATH entry not found (already removed or manually modified)${NC}"
    fi
else
    echo -e "${BLUE}Skipping PATH removal (use --remove-from-path to remove)${NC}"
fi
echo ""

# Step 4: Remove configuration (optional)
echo -e "${YELLOW}[4/4]${NC} Checking configuration files..."
if [ "$REMOVE_CONFIG" = true ]; then
    if [ -d "$HOME/.config/dtree" ]; then
        rm -rf "$HOME/.config/dtree"
        echo -e "${GREEN}✓ Removed configuration directory: ~/.config/dtree${NC}"
    else
        echo -e "${BLUE}Configuration directory not found (already removed)${NC}"
    fi
else
    echo -e "${BLUE}Keeping configuration (use --remove-config to remove)${NC}"
fi
echo ""

# Uninstallation summary
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Uninstallation Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "What was removed:"
echo -e "  ${GREEN}✓${NC} Binary: ~/bin/dtree"
echo -e "  ${GREEN}✓${NC} Wrapper function: $SHELL_CONFIG"
if [ "$REMOVE_FROM_PATH" = true ]; then
    echo -e "  ${GREEN}✓${NC} PATH entry: removed"
fi
if [ "$REMOVE_CONFIG" = true ]; then
    echo -e "  ${GREEN}✓${NC} Configuration: ~/.config/dtree"
fi
echo ""

if [ -f "$SHELL_CONFIG.dtree.backup" ]; then
    echo -e "${YELLOW}Next steps:${NC}"
    echo -e "  1. Reload your shell configuration:"
    echo -e "     ${BLUE}source $SHELL_CONFIG${NC}"
    echo -e "  2. If you encounter issues, restore backup:"
    echo -e "     ${BLUE}cp $SHELL_CONFIG.dtree.backup $SHELL_CONFIG${NC}"
    echo ""
fi

echo -e "Thank you for using dtree!"
echo ""
