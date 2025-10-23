# GIF Generation Fixes

## Problem: "dt: command not found"

**Issue:** VHS tapes were using the bash wrapper function `dt()` which is not available in VHS's isolated shell environment.

**Solution:** All tape files have been updated to:
1. Use `dtree` command directly instead of `dt`
2. Set `Env PATH` to include dtree location
3. Navigate to project directory before running dtree

## Applied Fixes

### 1. Updated all `.tape` files

**Before:**
```
Type "dt"
Enter
```

**After (with Hide/Show for cleaner demos):**
```
# Ensure dtree, ttyd, and system commands are in PATH
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"

# Hide startup commands - don't show cd and dtree launch
Hide
Type "cd /home/holger/github.com/holgertkey/dtree"
Enter
Type "/home/holger/bin/dtree"  # ABSOLUTE PATH (critical!)
Enter
Sleep 1s
Show

# Demo starts here - dtree is already running
```

**Key Changes:**
1. Use absolute path `/home/holger/bin/dtree` instead of just `dtree` (VHS doesn't respect PATH)
2. Wrap startup commands in `Hide`/`Show` to skip boring setup in GIFs
3. Recording starts when dtree UI is already visible

### 2. Updated `generate_all.sh`

Added PATH checks and exports:
```bash
# Check if dtree is available
if ! command -v dtree &> /dev/null; then
    echo "Warning: dtree command not found in PATH"
    export PATH="$HOME/bin:$PATH"
fi

# Export PATH for VHS subshells
PATH="$HOME/bin:$PATH" vhs "${demo}.tape"
```

### 3. Updated documentation

- `demos/README.md` - Added Prerequisites section
- `demos/README.md` - Enhanced Troubleshooting section
- `demos/record_demos.md` - Updated instructions

## Testing

To test if fixes work:

```bash
# Ensure dtree is installed
which dtree  # Should show: /home/holger/bin/dtree

# Test single demo
cd demos
vhs tree_navigation.tape

# Check output
ls -lh ../docs/assets/tree_navigation.gif
```

## Common Issues

### Issue: ttyd not found
**Solution:** Ensure ttyd is in PATH:
```bash
# Install ttyd if needed
sudo snap install ttyd

# Verify
which ttyd  # Should show: /snap/bin/ttyd
```

### Issue: Chrome/browser errors
**Solution:** PATH must include system directories:
```
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"
```

### Issue: dtree not found during recording
**Solution:**
1. Install dtree: `cp target/release/dtree ~/bin/`
2. Verify: `which dtree`
3. Add to PATH in tape: `Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"`

## Enhancement: Hide Startup Commands (Latest)

**Date:** 2025-10-23

**Issue:** GIFs showed unnecessary startup commands (`cd` and dtree launch), making demos longer and less professional.

**Solution:** Use VHS `Hide`/`Show` directives to skip startup commands:
```
Hide
Type "cd /home/holger/github.com/holgertkey/dtree"
Enter
Type "/home/holger/bin/dtree"
Enter
Sleep 1s
Show
# Recording starts here with dtree already visible
```

**Benefits:**
- ⚡ Faster, more focused demos
- 🎨 Cleaner, more professional appearance
- ✨ Viewers see dtree interface immediately
- 📦 Smaller GIF file sizes

## File Changes

All tape files updated:
- ✅ `tree_navigation.tape` (+ Hide/Show)
- ✅ `file_viewer.tape` (+ Hide/Show)
- ✅ `visual_selection.tape` (+ Hide/Show)
- ✅ `search.tape` (+ Hide/Show)
- ✅ `bookmarks.tape` (+ Hide/Show + alias for CLI demo)

Scripts updated:
- ✅ `generate_all.sh`

Documentation updated:
- ✅ `README.md`
- ✅ `record_demos.md`

## Verification

After fixes, generation should work:
```bash
cd demos
./generate_all.sh
```

Expected output:
```
=== Generating dtree demo GIFs ===
✓ Found dtree at: /home/holger/bin/dtree

Generating tree_navigation.gif...
Generating file_viewer.gif...
Generating visual_selection.gif...
Generating search.gif...
Generating bookmarks.gif...

✓ All demos generated successfully!
GIFs are in: docs/assets/
```
