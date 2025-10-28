# Windows Testing Checklist

Complete testing checklist for dtree on Windows platform.

---

## Pre-Testing Setup

- [ ] Rust installed (rustup-init.exe)
- [ ] Git for Windows installed
- [ ] Windows Terminal installed (recommended)
- [ ] Repository cloned
- [ ] Build successful: `cargo build --release`

---

## Basic Functionality

### Application Launch
- [x] Launch dtree from any directory: `cargo run`
- [x] Launch with path argument: `cargo run -- C:\Windows`
- [x] Launch compiled binary: `.\target\release\dtree.exe`
- [x] Help displays correctly: `dtree --help`
- [x] Version displays: `dtree --version`

### Tree Navigation
- [ ] Navigate down with `j` or `↓`
- [ ] Navigate up with `k` or `↑`
- [ ] Expand directory with `l` or `→`
- [ ] Collapse directory with `h` or `←`
- [ ] Go to parent with `u` or `Backspace`
- [ ] Select and exit with `Enter`
- [ ] Exit without selection with `Esc`

### Windows Path Support
- [ ] Absolute path with drive letter: `C:\Users`
- [ ] Different drive: `D:\` (if available)
- [ ] Path with spaces: `C:\Program Files`
- [ ] Path with spaces in quotes: `"C:\Program Files"`
- [ ] UNC network path: `\\server\share` (if available)
- [ ] Forward slashes: `C:/Windows` (should work)
- [ ] Mixed slashes: `C:\Users/Documents` (should normalize)

### Unicode Support
- [ ] Cyrillic filenames: Create file `тест.txt` and navigate to it
- [ ] Japanese filenames: Create file `テスト.txt`
- [ ] Emoji in filenames: Create file `📁test.txt`
- [ ] Chinese filenames: Create file `测试.txt`
- [ ] Display correctly in tree view
- [ ] Display correctly in file viewer

---

## File Operations

### File Viewing (Split View)
- [ ] Toggle split view with `s`
- [ ] View text file in split view
- [ ] View code file with syntax highlighting
- [ ] Scroll file with `Ctrl+j` / `Ctrl+k`
- [ ] Scroll by page with `Page Up` / `Page Down`
- [ ] Jump to start with `Home`
- [ ] Jump to end with `End`

### Fullscreen File Viewer
- [ ] Open file with `v` key (from tree)
- [ ] Open file from CLI: `dtree -v C:\Windows\System32\drivers\etc\hosts`
- [ ] Navigate with `j`/`k` or arrow keys
- [ ] Scroll by page with `Page Up`/`Page Down`
- [ ] Toggle line numbers with `l`
- [ ] Toggle line wrapping with `w`
- [ ] Return to tree with `q`
- [ ] Exit completely with `Esc`

### Visual Selection Mode
- [ ] Enter visual mode with `V` in fullscreen viewer
- [ ] Move selection down with `j` or `↓`
- [ ] Move selection up with `k` or `↑`
- [ ] Jump by page with `Page Up`/`Page Down`
- [ ] Jump to start with `Home`
- [ ] Jump to end with `End`
- [ ] Copy selection with `y` (should copy to clipboard)
- [ ] Exit visual mode with `Esc` or `V`
- [ ] Mouse scroll moves selection cursor
- [ ] Visual feedback: gray background for selected lines
- [ ] Status bar shows "VISUAL: N lines"

### File Search (Within File)
- [ ] Enter search mode with `/` in fullscreen viewer
- [ ] Type search query
- [ ] Navigate to next match with `n`
- [ ] Navigate to previous match with `N`
- [ ] Search highlights matches
- [ ] Clear search with `Esc`
- [ ] Search counter shows: "Match X of Y"

### External Programs
- [ ] Open file in notepad: Press `e` on a text file
- [ ] Open directory in Explorer: Press `o` on a directory
- [ ] Open file in VS Code: Configure editor in config, press `e`
- [ ] File opens correctly with spaces in path
- [ ] File opens correctly with Unicode in path

### Clipboard
- [ ] Copy current path with `c`
- [ ] Paste path in another app (verify clipboard)
- [ ] Copy works for directories
- [ ] Copy works for files
- [ ] Copy works with spaces in path
- [ ] Copy works with Unicode in path

---

## Search Functionality

### Tree Search
- [ ] Enter search mode with `/`
- [ ] Type search query (substring)
- [ ] Results appear incrementally
- [ ] Navigate results with `j`/`k` or arrow keys
- [ ] Select result with `Enter`
- [ ] Cancel search with `Esc`
- [ ] Search shows progress (dirs scanned)

### Fuzzy Search
- [ ] Enter fuzzy mode: Type `/` at start of query (e.g., `/fuz`)
- [ ] Results ranked by relevance score
- [ ] Scores displayed: `[95]` format
- [ ] Match indices highlighted in results
- [ ] More relevant results appear first
- [ ] Works with abbreviations (e.g., `/dt` finds `dtree`)

### Search Performance
- [ ] Search doesn't block UI
- [ ] Can navigate while search is running
- [ ] Background search thread works
- [ ] Cancel search immediately with `Esc`
- [ ] Large directories search quickly

---

## Bookmarks

### Creating Bookmarks
- [ ] Create bookmark with `m` on directory
- [ ] Enter bookmark key (a-z, 0-9)
- [ ] Enter bookmark name
- [ ] Bookmark saved successfully
- [ ] Create bookmark on file (should save parent dir)

### Using Bookmarks
- [ ] Open bookmarks menu with `'` (tick/apostrophe)
- [ ] List shows all bookmarks
- [ ] Navigate bookmarks with `j`/`k`
- [ ] Select bookmark with `Enter`
- [ ] Navigate to bookmark directory
- [ ] Exit bookmark menu with `Esc`

### Deleting Bookmarks
- [ ] Open bookmarks menu with `'`
- [ ] Press `d` on bookmark (first time: mark for deletion)
- [ ] Bookmark shows "DELETE?" indicator
- [ ] Press `d` again to confirm deletion
- [ ] Bookmark removed from list
- [ ] Press any other key to cancel deletion

### CLI Bookmark Management
- [x] List bookmarks: `dtree -bm` or `dtree -bm list`
- [ ] Add bookmark: `dtree -bm add work`
- [ ] Add bookmark with path: `dtree -bm add work C:\Projects`
- [ ] Remove bookmark: `dtree -bm remove work`
- [ ] Invalid command shows error and usage

### Bookmark Navigation
- [ ] Navigate to bookmark: `dt work` (using wrapper)
- [ ] Direct: `dtree work` outputs bookmark path
- [ ] Bookmark priority over directory name
- [ ] Error message if bookmark doesn't exist
- [ ] Error message if bookmark path invalid

---

## Configuration

### Config File
- [x] Config created on first run
- [x] Config location: `%APPDATA%\dtree\config.toml`
  - Check: `echo %APPDATA%\dtree\config.toml`
- [x] Config loads correctly
- [x] Config has Windows defaults (notepad, explorer)

### Bookmarks File
- [x] Bookmarks saved to: `%APPDATA%\dtree\bookmarks.json`
- [x] Bookmarks persist across sessions
- [x] Bookmarks load on startup
- [x] Bookmarks auto-save on changes

### Config Options
- [ ] Change theme in config (e.g., colors)
- [ ] Change keybindings in config
- [ ] Change editor: `editor = "code.exe"` (VS Code)
- [ ] Change file manager: `file_manager = "explorer.exe"`
- [ ] Custom colors work (RGB hex: `#FF0000`)
- [ ] Custom colors work (indexed: `42`)
- [ ] Invalid config shows error message

### Color Themes
- [ ] Default theme loads correctly
- [ ] Tree colors work
- [ ] File viewer colors work
- [ ] Search results colors work
- [ ] Help screen colors work
- [ ] Status bar colors work

---

## Shell Integration

### PowerShell Wrapper
- [x] Install wrapper: `.\install-windows.ps1`
- [x] Wrapper added to `$PROFILE`
- [x] Restart PowerShell or run `. $PROFILE`
- [x] `dt` command available

### Navigation with Wrapper
- [x] `dt` (no args) → Opens TUI ✅ Fixed on 2025-10-28
- [ ] Select directory and press `Enter` → `cd` to directory
- [ ] Exit with `Esc` → Stay in current directory
- [ ] `dt C:\Windows` → `cd` directly (no TUI)
- [ ] `dt work` → `cd` to bookmark (no TUI)
- [ ] `dt ..` → Error (should use `cd ..`)
- [ ] `dt .` → Opens TUI in current dir

### Previous Directory
- [ ] Navigate somewhere: `dt C:\Windows`
- [ ] Go back: `dt -` → Return to previous dir
- [ ] `dt -` again → Toggle between two dirs
- [ ] `dt -` without previous → Error message

### Flags with Wrapper
- [x] `dt --help` → Shows help (no cd)
- [x] `dt --version` → Shows version (no cd)
- [x] `dt -bm` → Lists bookmarks (no cd)
- [ ] `dt -bm add test` → Adds bookmark (no cd)
- [ ] `dt -v file.txt` → Views file, then can cd

---

## Edge Cases

### Long Paths (>260 characters)
- [ ] Create very long path (nested directories)
- [ ] Navigate to long path
- [ ] Display correctly in tree
- [ ] No truncation errors
- [ ] Copy long path to clipboard
- [ ] Bookmark long path

Note: Windows has MAX_PATH = 260 limit by default. May need to enable long paths:
```powershell
# Enable long paths (requires admin)
New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Control\FileSystem" -Name "LongPathsEnabled" -Value 1 -PropertyType DWORD -Force
```

### Network Shares (UNC Paths)
- [ ] Navigate to `\\server\share` (if available)
- [ ] Expand directories on network share
- [ ] View files on network share
- [ ] Bookmark network location
- [ ] Copy UNC path to clipboard
- [ ] Navigation speed acceptable

### Permission Errors
- [ ] Try to access `C:\System Volume Information`
- [ ] Error message displayed (not crash)
- [ ] Can continue navigation
- [ ] Try to access admin-only directory
- [ ] Graceful error handling

### Symlinks and Junctions
- [ ] Create symlink: `mklink /D link target`
- [ ] Create junction: `mklink /J junction target`
- [ ] Navigate through symlink
- [ ] Display symlink indicator (if implemented)
- [ ] No infinite loops
- [ ] No crashes

### Hidden Files
- [ ] System hidden files (e.g., `C:\pagefile.sys`)
- [ ] User hidden files (attrib +h)
- [ ] Hidden directories
- [ ] Displayed correctly (or filtered)

### Large Directories
- [ ] Navigate to directory with 1000+ files
- [ ] Performance acceptable
- [ ] No lag when scrolling
- [ ] Search works quickly
- [ ] Expand/collapse responsive

### Binary Files
- [ ] View binary file (.exe, .dll, .bin)
- [ ] Shows "Binary file" indicator
- [ ] No crashes or garbled output
- [ ] Can exit viewer cleanly
- [ ] Hex editor option available (if configured)

### Very Large Text Files
- [ ] View file >10MB
- [ ] View file >100MB (if available)
- [ ] HEAD mode works (first 10,000 lines)
- [ ] TAIL mode works (last 10,000 lines)
- [ ] No memory issues
- [ ] Scroll performance acceptable

### Empty Directories
- [ ] Navigate to empty directory
- [ ] Display "(empty)" indicator
- [ ] No errors
- [ ] Can navigate away

### Special Characters in Paths
- [ ] Path with `&`: `C:\dir&name`
- [ ] Path with `%`: `C:\dir%name`
- [ ] Path with `$`: `C:\dir$name`
- [ ] Path with `@`: `C:\dir@name`
- [ ] Path with `!`: `C:\dir!name`
- [ ] All display and navigate correctly

---

## Terminal Compatibility

### Windows Terminal (Recommended)
- [ ] Launch from Windows Terminal
- [ ] Colors display correctly
- [ ] Unicode renders properly
- [ ] Mouse support works
- [ ] No visual artifacts
- [ ] Performance good

### PowerShell 7
- [ ] Launch from PowerShell 7
- [ ] All features work
- [ ] Wrapper functions correctly
- [ ] Colors work

### PowerShell 5.1 (Built-in)
- [ ] Launch from PowerShell 5.1
- [ ] All features work
- [ ] Wrapper functions correctly
- [ ] Colors work (may be limited)

### Cmd.exe
- [ ] Launch from cmd.exe
- [ ] Basic functionality works
- [ ] Navigation works
- [ ] File viewing works
- [ ] Colors work (may be limited)
- [ ] Unicode may have issues (expected)

### Git Bash
- [ ] Launch from Git Bash
- [ ] Works like Linux (uses Unix paths)
- [ ] Bash wrapper works
- [ ] Full functionality

---

## Performance

### Startup Time
- [ ] Cold start: < 1 second
- [ ] Warm start: < 0.5 seconds
- [ ] Large directory: < 2 seconds

### Navigation Performance
- [ ] No lag when moving up/down
- [ ] Instant expand/collapse
- [ ] Smooth scrolling
- [ ] Responsive keyboard input

### Search Performance
- [ ] Quick search completes instantly
- [ ] Deep search doesn't block UI
- [ ] Progress updates smooth
- [ ] Can navigate during search

### File Viewer Performance
- [ ] File loads quickly (< 1 second for <1MB)
- [ ] Scrolling is smooth
- [ ] Syntax highlighting fast
- [ ] No lag with large files (in HEAD/TAIL mode)

### Memory Usage
- [ ] Reasonable memory usage (< 50MB typical)
- [ ] No memory leaks (use Task Manager)
- [ ] Stable over time

---

## Regression Tests

### Automated Tests
- [ ] Run: `cargo test`
- [ ] All tests pass
- [ ] No Windows-specific test failures
- [ ] Test coverage adequate

### Bug Fixes
After fixing any bug:
- [ ] Bug is fixed
- [ ] Test added for bug
- [ ] Test passes
- [ ] No new bugs introduced

---

## Installation Testing

### Binary Installation
- [ ] Build release: `cargo build --release`
- [ ] Copy binary: `.\install-windows-binary.ps1`
- [ ] Binary in PATH
- [ ] Can run from any directory: `dtree`

### Wrapper Installation
- [ ] Run: `.\install-windows.ps1`
- [ ] Function added to `$PROFILE`
- [ ] Works after PowerShell restart
- [ ] `dt` command available globally

---

## Documentation

- [ ] README.md has Windows instructions
- [ ] CLAUDE.md updated with Windows info
- [ ] WINDOWS_PORTING_GUIDE.md accurate
- [ ] All scripts documented
- [ ] Troubleshooting section complete

---

## CI/CD (If Implemented)

- [ ] GitHub Actions workflow for Windows
- [ ] Build succeeds on Windows runner
- [ ] Tests pass on Windows runner
- [ ] Artifacts uploaded correctly
- [ ] Release binaries work

---

## Final Checklist

- [ ] All critical tests pass
- [ ] No crashes or panics
- [ ] Performance acceptable
- [ ] User experience smooth
- [ ] Documentation complete
- [ ] Ready for release

---

## Test Environment

Record your test environment for reference:

- **Windows Version**: _________________ (e.g., Windows 11 23H2)
- **PowerShell Version**: ______________ (e.g., 7.4.0)
- **Terminal**: ________________________ (e.g., Windows Terminal 1.18)
- **Rust Version**: ____________________ (e.g., 1.75.0)
- **Binary Size**: _____________________ (e.g., 2.5 MB)
- **Test Date**: _______________________ (e.g., 2025-10-24)

---

## Known Issues

Document any issues found during testing:

1. _________________________________________________________________
2. _________________________________________________________________
3. _________________________________________________________________

---

## Notes

Additional observations or comments:

________________________________________________________________________
________________________________________________________________________
________________________________________________________________________

---

**Testing completed by**: ___________________
**Date**: ___________________
**Status**: ⬜ Passed | ⬜ Failed | ⬜ Needs Work
