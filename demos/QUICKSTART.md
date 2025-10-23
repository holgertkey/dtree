# Quick Start - Generating Demo GIFs

## TL;DR

```bash
cd demos
./generate_all.sh
```

That's it! GIFs will be in `../docs/assets/`

---

## What Got Fixed

The previous issue was: `bash: dtree: command not found`

**Root Cause:** VHS runs commands in an isolated shell that doesn't respect PATH.

**Solution:** All `.tape` files now use absolute paths:
- ❌ Before: `Type "dtree"`
- ✅ After: `Type "/home/holger/bin/dtree"`

---

## Prerequisites

### 1. Install dtree
```bash
# From project root
cargo build --release
cp target/release/dtree ~/bin/

# Verify
which dtree  # Should show: /home/holger/bin/dtree
```

### 2. Install VHS
```bash
# Install VHS
go install github.com/charmbracelet/vhs@latest

# Install ffmpeg (required)
sudo apt install ffmpeg  # Ubuntu/Debian
brew install ffmpeg      # macOS

# Verify
which vhs  # Should be in ~/go/bin/vhs
```

---

## Generating GIFs

### Option 1: Generate All (Recommended)
```bash
cd demos
./generate_all.sh
```

Output:
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

### Option 2: Generate Individual Demo
```bash
cd demos
vhs tree_navigation.tape      # Basic navigation
vhs file_viewer.tape          # File viewer
vhs visual_selection.tape     # Visual mode
vhs search.tape               # Search
vhs bookmarks.tape            # Bookmarks
```

---

## Troubleshooting

### "dtree: command not found" appears in GIF

**Cause:** Tape file uses relative command name instead of absolute path.

**Fix:** Edit the `.tape` file and change:
```diff
- Type "dtree"
+ Type "/home/holger/bin/dtree"
```

### "ttyd: command not found"

```bash
# Install ttyd
sudo snap install ttyd

# Verify
which ttyd  # Should show: /snap/bin/ttyd
```

### GIFs are too large (>5MB)

```bash
# Optimize with gifsicle
gifsicle -O3 --lossy=80 input.gif -o output.gif

# Or reduce duration in tape file
# Keep demos under 15 seconds
```

### Browser/Chrome errors

Make sure PATH includes system directories in tape file:
```
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"
```

---

## Creating Your Own Demo

1. Copy the template:
   ```bash
   cp example_template.tape my_demo.tape
   ```

2. Edit `my_demo.tape`:
   ```
   Output docs/assets/my_demo.gif

   # Hide startup commands for cleaner demo
   Hide
   Type "cd /home/holger/github.com/holgertkey/dtree"
   Enter
   Type "/home/holger/bin/dtree"  # Use absolute path!
   Enter
   Sleep 1s
   Show

   # Your actions here...
   ```

3. Generate:
   ```bash
   vhs my_demo.tape
   ```

---

## Files Reference

**Tape files** (recording scripts):
- `tree_navigation.tape` - Basic tree navigation
- `file_viewer.tape` - File preview and fullscreen
- `visual_selection.tape` - Visual selection mode
- `search.tape` - Search functionality
- `bookmarks.tape` - Bookmark management
- `example_template.tape` - Template for new demos

**Scripts:**
- `generate_all.sh` - Batch generate all demos

**Documentation:**
- `README.md` - Full documentation
- `QUICKSTART.md` - This file
- `FIXES.md` - Detailed fix documentation
- `record_demos.md` - Advanced recording guide

---

## Next Steps

After generating GIFs:

1. **Verify:** Open GIFs and check they look good
2. **Optimize:** Use gifsicle if files are too large
3. **Commit:** Add to git
   ```bash
   git add docs/assets/*.gif
   git commit -m "Add demo GIFs"
   ```
4. **Push:** Share with the world!

---

## Need Help?

- Check `README.md` for detailed instructions
- Check `FIXES.md` for troubleshooting history
- Check `record_demos.md` for advanced tips
