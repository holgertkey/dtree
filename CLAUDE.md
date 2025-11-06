# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## User notes
- **CRITICAL: All text in the code MUST be in English only** - this includes:
  - All comments
  - All error messages
  - All user-facing strings
  - All log messages
  - All documentation
  - No Russian, German, or any other language is allowed

- Avoid writing lines like this in the comments:
  "Generated with [Claude Code](https://claude.com/claude-code)
  Co-Authored-By: Claude <noreply@anthropic.com>"

- **IMPORTANT: When fixing bugs or errors in the code, ALWAYS write proper tests immediately to prevent regression.**

- Save all files created during program debugging in the '.debug' folder. Save report files in the '.debug/reports' folder. Save files for testing in the '.debug/testing' folder.

### Version Management
**Build Number Convention**: After each compilation with code changes, increment the build number in `Cargo.toml`:
- Format: `version = "MAJOR.MINOR.PATCH+BUILD"`
- Example: `1.0.0+000` → `1.0.0+001` → `1.0.0+002`
- The build number (`+NNN`) is a 3-digit zero-padded counter
- Reset build number to `+000` when MAJOR, MINOR, or PATCH version changes
- This helps track development iterations between releases

**When to increment**:
- ✅ After fixing bugs and recompiling
- ✅ After adding features and recompiling
- ✅ After refactoring and recompiling
- ❌ Do NOT increment for documentation-only changes
- ❌ Do NOT increment if code wasn't modified

## Project Overview

`dtree` is a lightweight Rust TUI (Terminal User Interface) application for interactive directory tree navigation. It provides a zi-like interface (from zoxide) for directory navigation without leaving the command-line context.

## Architecture

The project is organized into modular components:

 ## Module Structure

**`src/main.rs`**
- Entry point with `main()` function only
- Orchestrates terminal setup, app initialization, and cleanup
- Handles final output of selected path to stdout
- Functions: `open_in_editor()`, `open_in_file_manager()` - launch external programs
- **NOTE**: Contains platform-specific code for Unix/Windows (to be refactored to `platform.rs`)

**`src/tree_node.rs`**
- `TreeNode` struct: Recursive tree structure representing filesystem directories
- Lazy-loads children on demand (only when expanded)
- Only shows directories (files are filtered out)
- Handles expand/collapse state
- Methods: `new()`, `load_children()`, `toggle_expand()`

**`src/app.rs`**
- `App` struct: Main application state manager (74 lines after refactoring)
- Orchestrates all submodules: navigation, file_viewer, search, ui, event_handler, config
- Delegates operations to specialized modules
- Methods: `new()`, `handle_key()`, `handle_mouse()`, `render()`

**`src/navigation.rs`**
- `Navigation` struct: Tree navigation and node management (194 lines)
- Maintains root node (Rc<RefCell<TreeNode>>) and flattened list of visible nodes
- Rebuilds flat list on tree structure changes (expand/collapse)
- Methods: `new()`, `rebuild_flat_list()`, `move_up()`, `move_down()`, `toggle_node()`, `expand_path_to_node()`
- Uses zero-copy reference sharing via Rc<RefCell<>> for performance

**`src/file_viewer.rs`**
- `FileViewer` struct: File content loading and display (~870 lines)
- Loads and formats file content with configurable line limit
- Handles scrolling, Unicode truncation, file metadata display
- Search functionality: `/` to search within file, `n`/`N` to navigate matches
- Visual selection mode: Vim-style line selection with keyboard (V, j/k, y) and mouse support
- Methods: `new()`, `load_file_with_width()`, `scroll_up()`, `scroll_down()`, `format_file_info()`
- Visual mode methods: `enter_visual_mode()`, `visual_move_up/down()`, `copy_selection()`
- Graceful error handling for binary files, symlinks, permission errors

**`src/search.rs`**
- `Search` struct: Asynchronous search functionality and results management (~390 lines)
- Two-phase search architecture:
  - Phase 1: Quick search through already loaded (visible) nodes (instant)
  - Phase 2: Deep background search in separate thread (non-blocking)
- Features:
  - Two search modes: normal (substring) and fuzzy (query starts with `/`)
  - Normal mode: Case-insensitive substring matching
  - Fuzzy mode: Intelligent fuzzy matching with SkimMatcherV2 algorithm
    - Query starting with `/` activates fuzzy mode (e.g., `/fuz` finds `fuzzy.rs`)
    - Results ranked by relevance score (highest first)
    - Match indices tracked for character highlighting in UI
    - Displays scores in format `[95]` next to results
  - Search across entire tree (including collapsed nodes)
  - Live progress updates during background search (dirs scanned count)
  - Cancellable search (Esc key stops background thread)
  - Results appear incrementally as they're found
  - No UI blocking - app remains responsive during search
- Structures: `SearchMessage` enum (Result/Progress/Done), `SearchResult` struct (with score and match_indices)
- Methods: `new()`, `enter_mode()`, `exit_mode()`, `perform_search()`, `poll_results()`, `cancel_search()`, `add_char()`, `backspace()`, `move_up()`, `move_down()`, `update_fuzzy_mode()`, `get_search_query()`
- Supports switching focus between tree and results panel with Tab
- Thread-safe communication via crossbeam-channel (unbounded for results, bounded for cancellation)

**`src/ui.rs`**
- `UI` struct: All rendering logic and layout management (332 lines)
- Renders tree, file viewer, search bar, search results, help screen
- Manages split view, scroll offsets, terminal dimensions
- Methods: `render()`, `render_tree()`, `render_file_viewer()`, `render_search_bar()`, `render_search_results()`
- Applies theme colors from config to all UI elements
- Function: `get_help_content()` - returns help text as Vec<String>

**`src/event_handler.rs`**
- `EventHandler` struct: Keyboard and mouse input processing (363 lines)
- Handles all user interactions: navigation, search, file operations, mouse events
- Manages mouse state: dragging, double-click detection
- Methods: `handle_key()`, `handle_mouse()`, `handle_search_input()`
- Applies configurable keybindings and timeouts from config

**`src/config.rs`**
- `Config` struct: Configuration file management (~570 lines)
- Auto-creates `~/.config/dtree/config.toml` on first run
- Loads and parses TOML configuration with serde
- Structures: `AppearanceConfig`, `BehaviorConfig`, `KeybindingsConfig`, `ThemeConfig`
- Methods: `load()`, `from_file()`, `create_default_file()`, `parse_color()`, `editor_exists()`, `file_manager_exists()`
- Keybindings: All shortcuts are configurable (navigation, search, bookmarks, visual mode)
- Supports color names, RGB hex (#RRGGBB), and indexed colors (0-255)
- External editor and file manager configuration with existence validation

**`src/bookmarks.rs`**
- `Bookmarks` struct: Bookmark management and persistence
- Save favorite directories with alphanumeric keys (a-z, 0-9)
- Persistent storage in `~/.config/dtree/bookmarks.json`
- Methods: `new()`, `add()`, `get()`, `list()`, `save()`, `load()`
- Interactive modes: creation and selection
- Auto-loads on startup, auto-saves on changes

**`src/terminal.rs`**
- Terminal lifecycle management
- `setup_terminal()`: Enables raw mode and alternate screen
- `cleanup_terminal()`: Restores terminal to normal state
- `run_app()`: Main event loop for handling user input and rendering
  - Uses `event::poll(Duration::from_millis(100))` for non-blocking event checking
  - Allows background search thread to update results while maintaining UI responsiveness
  - Polls search results when no events available

**`src/platform.rs`** (planned for Windows port)
- Platform-specific implementations for Unix and Windows
- Functions: `open_external_program()`, `is_absolute_path()`, `normalize_path_separator()`
- Uses `#[cfg(unix)]` and `#[cfg(windows)]` for conditional compilation

### Terminal I/O Architecture

**Critical Design Detail**: Terminal UI uses `stderr` while output path uses `stdout`
- Ratatui terminal backend: `CrosstermBackend::new(std::io::stderr())` (see `terminal.rs`)
- Selected path output: `println!("{}", path.display())` to stdout (see `main.rs`)
- This separation allows the bash wrapper to capture the output path without interference from TUI
- Event loop uses 100ms timeout polling to enable async search updates without blocking UI

## Commands

### Build
```bash
# Linux/macOS
cargo build --release

# Windows (PowerShell)
cargo build --release
```

**Note**: The release profile is optimized for binary size:
- `opt-level = "z"` - Optimize for size
- `lto = true` - Link Time Optimization enabled
- `codegen-units = 1` - Single codegen unit for better optimization
- `strip = true` - Strip symbols from binary
- `panic = "abort"` - Abort on panic (smaller binary)

Result: ~2.5MB binary size (down from 4.6MB without optimizations)

### Run (for testing)
```bash
cargo run
```

### Install

#### Linux (primary development machine)

**IMPORTANT: User installation location is `/home/holger/bin/dtree`**

To update the installed binary after making changes:
```bash
# Build the project
cargo build --release

# Copy to user's bin directory
cp target/release/dtree /home/holger/bin/
```

Alternative installation (system-wide):
```bash
# Build and install to system path
cargo build --release
sudo cp target/release/dtree /usr/local/bin/
```

#### Windows

**Recommended installation location: `C:\Users\Holger\bin\dtree.exe`**

**Automated installation (recommended):**
```powershell
# One-step installation with all wrappers
.\install-windows-binary.ps1
```

This script will:
- Build the release binary
- Copy `dtree.exe` to `C:\Users\Holger\bin\`
- Copy `dt.bat` wrapper to `C:\Users\Holger\bin\` (for cmd.exe)
- Add directory to PATH
- Install PowerShell wrapper function (for PowerShell 5.x and 7.x+)
- Test the installation

**Result:** The `dt` command works in both PowerShell and cmd.exe!

**Manual installation:**
```powershell
# Build the project
cargo build --release

# Create bin directory and copy files
New-Item -Path "$env:USERPROFILE\bin" -ItemType Directory -Force
Copy-Item target\release\dtree.exe "$env:USERPROFILE\bin\"
Copy-Item dt.bat "$env:USERPROFILE\bin\"

# Add to PATH (restart terminal after this)
[Environment]::SetEnvironmentVariable("Path", $env:PATH + ";$env:USERPROFILE\bin", "User")

# Install PowerShell wrapper (for both 5.x and 7.x+)
.\install-windows-wrapper.ps1
```

**Alternative (cargo install):**
```powershell
cargo install --path .
# Places binary in: C:\Users\Holger\.cargo\bin\dtree.exe
```

#### Shell Integration

**Linux/macOS**: Add bash wrapper to `~/.bashrc`:

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

**Windows**: PowerShell integration (automatically installed by `install-windows-binary.ps1`)

**Automatic setup (recommended):**
```powershell
# Run this to install the wrapper
.\install-windows-wrapper.ps1
```

This script will:
- Automatically detect installed PowerShell versions (Windows PowerShell 5.x and PowerShell Core 7.x+)
- Install the `dt` wrapper function to all detected PowerShell profiles
- Update existing installations without breaking your profile
- Show detailed summary of what was installed

**Supported PowerShell versions:**
- Windows PowerShell 5.x (profile: `Documents\WindowsPowerShell\Microsoft.PowerShell_profile.ps1`)
- PowerShell Core 7.x+ (profile: `Documents\PowerShell\Microsoft.PowerShell_profile.ps1`)

If both versions are installed, the wrapper will be added to both profiles automatically.

**Manual setup** - Add to your PowerShell profile (`notepad $PROFILE`):

```powershell
function dt {
    param(
        [Parameter(ValueFromRemainingArguments=$true)]
        [string[]]$Arguments
    )

    $prevDir = $PWD.Path

    # Handle dt - (return to previous directory)
    if ($Arguments.Count -eq 1 -and $Arguments[0] -eq "-") {
        if ($env:DTREE_PREV_DIR -and (Test-Path $env:DTREE_PREV_DIR)) {
            Set-Location $env:DTREE_PREV_DIR
            $env:DTREE_PREV_DIR = $prevDir
        } else {
            Write-Error "dt: no previous directory"
        }
        return
    }

    # Handle flags that should run dtree directly
    if ($Arguments.Count -gt 0 -and $Arguments[0] -in @("-h", "--help", "--version", "-bm")) {
        & dtree.exe $Arguments
        return
    }

    # Navigation mode
    $result = & dtree.exe $Arguments 2>&1 | Out-String
    if ($LASTEXITCODE -eq 0 -and $result.Trim() -and (Test-Path $result.Trim())) {
        Set-Location $result.Trim()
        $env:DTREE_PREV_DIR = $prevDir
    }
}
```

**Testing PowerShell integration:**
```powershell
# Restart PowerShell or reload profile
. $PROFILE

# Test basic commands
dt --version        # Should work (calls dtree.exe directly)
dt -bm list         # Should work (calls dtree.exe directly)

# Test navigation
dt C:\Windows       # Should change directory to C:\Windows
dt -                # Should return to previous directory

# Test bookmarks
dt -bm add test     # Create bookmark
dt test             # Navigate to bookmark

# Test in both PowerShell versions (if both are installed)
# Windows PowerShell 5.x:
powershell -Command "dt --version"

# PowerShell Core 7.x+:
pwsh -Command "dt --version"
```

#### Cmd.exe Integration

**dt.bat wrapper** is automatically installed to `C:\Users\Holger\bin\dt.bat` by `install-windows-binary.ps1`

**How it works:**
- The `dt.bat` script wraps `dtree.exe` to provide `cd` integration in cmd.exe
- Same functionality as PowerShell wrapper: navigation, bookmarks, file viewing
- **No conflict with PowerShell**: PowerShell uses the `dt()` function, cmd.exe uses `dt.bat`

**Usage in cmd.exe:**
```cmd
REM Basic commands
dt --version        REM Show version
dt -bm list         REM List bookmarks

REM Navigation
dt C:\Windows       REM Navigate to directory
dt -                REM Return to previous directory
dt                  REM Open interactive TUI

REM File viewing
dt -v file.txt      REM View file in fullscreen

REM Bookmarks
dt -bm add test     REM Create bookmark
dt test             REM Navigate to bookmark
```

**Technical notes:**
- Uses `DTREE_PREV_DIR` environment variable to track previous directory
- Supports all dtree flags and arguments
- Automatically converts relative paths to absolute for file viewing
- Works alongside PowerShell wrapper without conflicts

### Uninstall

#### Windows

To uninstall dtree from Windows, use the provided uninstall script:

```powershell
# Basic uninstall (removes binary and wrappers only)
.\uninstall-windows.ps1

# Remove from PATH as well
.\uninstall-windows.ps1 -RemoveFromPath

# Remove configuration and bookmarks
.\uninstall-windows.ps1 -RemoveConfig

# Complete removal (everything)
.\uninstall-windows.ps1 -RemoveFromPath -RemoveConfig

# Force removal without confirmation prompts
.\uninstall-windows.ps1 -RemoveConfig -Force
```

**What gets removed:**
- `dtree.exe` from `C:\Users\Holger\bin\`
- `dt.bat` wrapper from `C:\Users\Holger\bin\`
- `dt` function from PowerShell profiles (both Windows PowerShell 5.x and PowerShell Core 7.x+)
- Optionally: PATH entry (with `-RemoveFromPath`)
- Optionally: Configuration directory at `%APPDATA%\dtree\` (with `-RemoveConfig`)

**Manual uninstall (if script fails):**
```powershell
# Remove binaries
Remove-Item "$env:USERPROFILE\bin\dtree.exe" -Force
Remove-Item "$env:USERPROFILE\bin\dt.bat" -Force

# Remove configuration (optional)
Remove-Item "$env:APPDATA\dtree" -Recurse -Force

# Remove PowerShell wrapper manually:
# Edit $PROFILE and remove the dt function block between markers:
# # >>> dtree wrapper start <<<
# # >>> dtree wrapper end <<<
```

#### Linux/macOS

```bash
# Remove binary
sudo rm /usr/local/bin/dtree
# or for user installation:
rm ~/bin/dtree

# Remove configuration (optional)
rm -rf ~/.config/dtree

# Remove bash wrapper from ~/.bashrc
# Edit ~/.bashrc and remove the dt() function
```

### Usage
```bash
# Navigation (with bookmark support)
dt                    # Open interactive tree from current directory
dt /path/to/dir       # Jump directly to path (no TUI)
dt myproject          # Jump directly to bookmark 'myproject' or directory (no TUI)
dt -                  # Return to previous directory (like cd -)

# File viewing
dt -v file.txt        # View file in fullscreen mode (supports navigation and cd after viewing)
dtree -v file.txt     # View file in fullscreen mode
dtree --view file.txt # View file in fullscreen mode

# Bookmark management
dt -bm                # List all bookmarks
dt -bm list           # List all bookmarks
dt -bm add work       # Save current directory as bookmark 'work'
dt -bm add work /path # Save specific path as bookmark 'work'
dt -bm remove work    # Remove bookmark 'work'

# General
dtree --version       # Print version
dtree -h              # Print help
```

**Behavior:**
- `dt` (no arguments) → Opens interactive TUI
- `dt <path>` or `dt <bookmark>` → Changes directory immediately (no TUI)
- `dt -v <file>` → Opens file viewer, then navigates to selected directory on exit (with `q`)
- Priority: Bookmark → Path → Error

## Key Bindings

**Navigation:**
- `j`/`↓` or `k`/`↑`: Navigate down/up
- `l`/`→`: Expand directory
- `h`/`←`: Collapse directory
- `u`/`Backspace`: Go to parent directory (changes root)
- `Enter`: Select directory and exit (returns path to bash wrapper)

**File Viewing:**
- `s`: Toggle file viewer mode (show/hide files in split view)
- `v`: Open file in fullscreen viewer (only works when a file is selected)
- `Ctrl+j`/`Ctrl+k`: Scroll file preview by line
- `Page Up`/`Page Down`: Scroll file preview by page
- `Home`/`End`: Jump to start/end of file

**Fullscreen Viewer (when viewing a file with `v`):**
- `l`: Toggle line numbers (show/hide)
- `w`: Toggle line wrapping (wrap/truncate long lines)
- `q`: Return to tree view (stay in program)
- `Esc`: Exit program completely (or clear search if search is active)
- `j`/`k` or `↓`/`↑`: Scroll down/up by line
- `Ctrl+j`/`Ctrl+k`: Jump to next/previous file in directory
- `Page Up`/`Page Down`: Scroll by page
- `Home`: Switch to HEAD mode (first 10,000 lines)
- `End`: Switch to TAIL mode (last 10,000 lines)
- `/`: Enter file search mode (search within file)
- `n`/`N`: Next/previous search match (when search results exist)
- `V`: Enter visual selection mode (Vim-style line selection)
- `Mouse scroll`: Scroll document (or move selection cursor in visual mode)

**Visual Selection Mode (activated with `V` in fullscreen viewer):**
- `j`/`k` or `↓`/`↑`: Expand selection down/up
- `Page Up`/`Page Down`: Jump selection by page
- `Home`/`End`: Jump to start/end of file
- `y`: Copy selected lines to clipboard and exit visual mode
- `Esc` or `V`: Exit visual mode without copying
- `Mouse scroll`: Move selection cursor (with auto-scroll)
- Visual feedback: Selected lines highlighted with gray background, cursor line with blue background
- Status bar shows selection size (e.g., "VISUAL: 25 lines")

**Bookmarks:**
- `m`: Create bookmark (saves directories only, if on file → saves parent dir)
- `'`: Open bookmarks menu (tick/apostrophe)
- `d` (in bookmark selection): Delete bookmark (two-phase: mark → confirm)

**Other:**
- `i`: Toggle help screen
- `c`: Copy current path to clipboard
- `e`: Open file in external editor (configurable in config.toml)
- `o`: Open in file manager (files → parent dir, dirs → self)
- `/`: Enter search mode (tree search)
- `z`: Toggle directory size display (shows calculated sizes)

## Important Implementation Details

1. **Parent Directory Navigation**: The `u`/`Backspace` key navigates to parent by:
   - Creating a new root from parent directory
   - Rebuilding the entire tree
   - Finding and selecting the previous directory in the new tree (see `app.rs::handle_key()`)

2. **Node Toggling**: Uses recursive traversal to find and toggle nodes by path comparison
   - `toggle_node_recursive()` in `app.rs` recursively searches tree
   - Must rebuild flat list after any tree modification

3. **Terminal Restoration**: Always restores terminal state before printing output path
   - See `terminal.rs::cleanup_terminal()` and `main.rs`
   - Ensures clean exit and proper bash wrapper integration

4. **Error Handling**: Uses `anyhow::Result` throughout for ergonomic error propagation

5. **Unicode Handling**: Lines are truncated using `unicode-truncate` to prevent artifacts
   - File viewer calculates max width based on panel size
   - Wide characters (emoji, CJK) are properly handled
   - Prevents UI corruption from long lines with special characters

6. **Fullscreen Viewer**: Can be activated in two ways:
   - Press `v` on a file in tree navigation mode
   - Use CLI: `dtree -v filename` or `dtree --view filename`
   - Exit with `q` or `Esc` to return to tree view (or exit if launched from CLI)

## Dependencies

- `ratatui 0.28`: TUI framework (widgets, layout, rendering)
- `crossterm 0.28`: Terminal manipulation (raw mode, events, alternate screen)
- `anyhow 1.0`: Error handling
- `clap 4.5`: CLI argument parsing with derive macros
- `arboard 3.4`: Clipboard operations (copy path functionality)
- `unicode-width 0.1`: Calculate display width of Unicode strings
- `unicode-truncate 1.1`: Safely truncate strings at Unicode boundaries
- `serde 1.0`: Serialization/deserialization framework (with derive feature)
- `serde_json 1.0`: JSON serialization for bookmarks storage
- `toml 0.8`: TOML configuration file parsing
- `dirs 5.0`: Platform-specific user directories (config_dir)
- `syntect 5.2`: Syntax highlighting with Sublime Text definitions (100+ languages)
- `once_cell 1.19`: Lazy-loaded static initialization for SyntaxSet/ThemeSet
- `which 6.0`: Locate executable binaries in PATH (editor existence check)
- `crossbeam-channel 0.5`: Thread-safe multi-producer multi-consumer channels for async search
- `fuzzy-matcher 0.3`: Fuzzy string matching algorithm (SkimMatcherV2) for search

## Platform Support

### Cross-platform Architecture

dtree is being ported to support Linux, macOS, and Windows through conditional compilation.

**Current status**:
- ✅ Linux: Fully supported (primary development platform)
- ✅ macOS: Should work (not extensively tested)
- 🚧 Windows: Work in progress (see `WINDOWS_PORTING_GUIDE.md`)

### Platform-specific Implementation

**Key differences between Unix and Windows:**

| Aspect | Unix (Linux/macOS) | Windows |
|--------|-------------------|---------|
| **Config location** | `~/.config/dtree/config.toml` | `%APPDATA%\dtree\config.toml` |
| **Bookmarks** | `~/.config/dtree/bookmarks.json` | `%APPDATA%\dtree\bookmarks.json` |
| **Path format** | `/absolute/path`, `./relative` | `C:\`, `D:\`, `\\server\share` |
| **Path separator** | `/` (forward slash) | `\` (backslash) |
| **Shell** | sh/bash | cmd.exe / PowerShell |
| **Default editor** | `$EDITOR` or `nano` | VS Code or Notepad |
| **Default file manager** | `xdg-open` / `open` | `explorer.exe` |
| **Line endings** | LF (`\n`) | CRLF (`\r\n`) |
| **Terminal** | Unix TTY | Windows Console API / Windows Terminal |

### Code Organization for Cross-platform

**Planned structure** (during Windows porting):

```rust
// src/platform.rs - Platform-specific implementations

#[cfg(unix)]
pub fn open_external_program(program: &str, path: &str) -> Result<()> {
    // Unix: use sh shell
    Command::new("sh").arg("-c").arg(format!("{} '{}'", program, path)).status()?;
    Ok(())
}

#[cfg(windows)]
pub fn open_external_program(program: &str, path: &str) -> Result<()> {
    // Windows: direct execution or cmd.exe
    Command::new(program).arg(path).status()?;
    Ok(())
}
```

**Important coding guidelines for cross-platform support:**

1. **Never hardcode paths**:
   - ❌ `path.starts_with('/')` - Unix only
   - ✅ `path.is_absolute()` - works everywhere
   - ✅ Use `std::path::Path` and `PathBuf`

2. **Never hardcode shell commands**:
   - ❌ `Command::new("sh")` - doesn't exist on Windows
   - ✅ Use `platform::open_external_program()`

3. **Use cross-platform crates**:
   - ✅ `dirs` - user directories (handles %APPDATA% vs ~/.config)
   - ✅ `crossterm` - terminal (works on Windows Console and Unix)
   - ✅ `arboard` - clipboard (uses Win32 API on Windows)

4. **Test on both platforms** (when possible):
   - Linux: Primary development and testing
   - Windows: Port testing (see `TESTING_WINDOWS.md`)

### Dependencies - Cross-platform Notes

All dependencies work cross-platform:
- `crossterm 0.28`: Works with Windows Console API and Unix terminals
- `ratatui 0.28`: Pure Rust, cross-platform
- `arboard 3.4`: Uses Win32 clipboard API on Windows, X11/Wayland on Linux
- `dirs 5.0`: Uses Windows API for %APPDATA%, XDG dirs on Linux
- All other dependencies: Pure Rust or cross-platform

### Windows Port Status

For detailed Windows porting instructions and progress, see:
- `WINDOWS_PORTING_GUIDE.md` - Complete step-by-step guide (6 phases)
- `TESTING_WINDOWS.md` - Test checklist for Windows
- `install-windows-wrapper.ps1` - PowerShell wrapper installer
- `install-windows-binary.ps1` - Binary installation script

**Windows porting phases:**
1. ✅ Preparation and analysis
2. 🚧 Cross-platform architecture (create `platform.rs`)
3. ⏳ Shell integration (PowerShell wrapper)
4. ⏳ Testing and debugging
5. ⏳ Documentation updates
6. ⏳ CI/CD for Windows builds

## Future Improvements

For detailed roadmap including completed features, planned improvements, and priority rankings, see [ROADMAP.md](ROADMAP.md).
