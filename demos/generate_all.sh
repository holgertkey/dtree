#!/bin/bash

# Generate all demo GIFs for dtree
# Requires VHS: https://github.com/charmbracelet/vhs

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Generating dtree demo GIFs ===${NC}\n"

# Check if VHS is installed
if ! command -v vhs &> /dev/null; then
    echo "Error: VHS is not installed"
    echo "Install with: go install github.com/charmbracelet/vhs@latest"
    echo "Or see: https://github.com/charmbracelet/vhs"
    exit 1
fi

# Check if dtree is available
if ! command -v dtree &> /dev/null; then
    echo -e "${YELLOW}Warning: dtree command not found in PATH${NC}"
    echo "Make sure dtree is installed (e.g., in ~/bin/dtree)"
    echo "Adding ~/bin to PATH for this session..."
    export PATH="$HOME/bin:$PATH"

    if ! command -v dtree &> /dev/null; then
        echo "Error: Still cannot find dtree. Please install it first:"
        echo "  cd .."
        echo "  cargo build --release"
        echo "  cp target/release/dtree ~/bin/"
        exit 1
    fi
fi

echo -e "${GREEN}✓ Found dtree at: $(which dtree)${NC}\n"

# Create output directory
mkdir -p ../docs/assets

# List of demos to generate
DEMOS=(
    "tree_navigation"
    "file_viewer"
    "visual_selection"
    "search"
    "bookmarks"
)

# Export PATH for VHS subshells
export PATH="$HOME/bin:$PATH"

# Generate each demo
for demo in "${DEMOS[@]}"; do
    echo -e "${GREEN}Generating ${demo}.gif...${NC}"
    # Run VHS with explicit PATH
    PATH="$HOME/bin:$PATH" vhs "${demo}.tape"
    echo ""
done

echo -e "${GREEN}✓ All demos generated successfully!${NC}"
echo -e "GIFs are in: ${BLUE}docs/assets/${NC}"
echo ""
echo "Next steps:"
echo "1. Review the generated GIFs"
echo "2. Optimize file sizes if needed (use gifsicle or similar)"
echo "3. Update README.md with the new GIF links"
