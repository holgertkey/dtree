# Basic Usage

This guide covers the fundamental operations in dtree.

## Launching dtree

### From Current Directory

```bash
dt
```

Opens the TUI showing the current directory tree.

### From Specific Directory

```bash
dt /path/to/directory
```

Opens the TUI with the specified directory as root.

### Direct Navigation

```bash
dt myproject
```

If `myproject` is a bookmark or directory, changes to it immediately without opening the TUI.

### File Viewing

```bash
dt -v filename.txt
```

Opens the file in fullscreen viewer mode.

## Understanding the Interface

### Tree View (Default)

```
┌─ Directory Tree ─────────────────────────┐
│ 📁 my-project/                           │
│ ├─ 📁 src/                               │
│ │  ├─ 📄 main.rs                         │
│ │  └─ 📄 lib.rs                          │
│ ├─ 📁 tests/                             │
│ └─ 📄 Cargo.toml                         │
└──────────────────────────────────────────┘
```

- `📁` indicates a directory
- `📄` indicates a file (when file viewer mode is enabled)
- `►` collapsed directory
- `▼` expanded directory
- `⚠` directory with read errors

### Split View (File Viewer Mode)

```
┌─ Tree ─────┬─ File Preview ───────────┐
│ my-project/│ fn main() {              │
│ ├─ src/    │     println!("Hello!");  │
│ │  main.rs │ }                        │
│ └─ lib.rs  │                          │
│            │ Lines: 3                 │
│            │ Size: 45 B               │
└────────────┴──────────────────────────┘
```

Press `s` to toggle between tree-only and split view.

### Fullscreen Viewer

```
┌─ main.rs ─────────────────────────────┐
│   1 │ fn main() {                     │
│   2 │     println!("Hello, world!");  │
│   3 │ }                               │
│     │                                 │
│ l: hide lines | q: back | Esc: exit   │
└───────────────────────────────────────┘
```

Press `v` on a file to enter fullscreen viewer.

## Basic Navigation

### Moving the Cursor

```
j or ↓        Move down one item
k or ↑        Move up one item
Ctrl+d        Jump down half page
Ctrl+u        Jump up half page
Home          Jump to first item
End           Jump to last item
```

### Expanding and Collapsing

```
l or →        Expand directory (show children)
h or ←        Collapse directory (hide children)
Enter         Expand deeply OR change root to selected directory
```

**Tip**: Double-click a directory to toggle expand/collapse.

### Changing Root

```
Enter         Change root to selected directory
u             Go to parent directory
Backspace     Go to parent directory
```

When you press `Enter` on a directory, it becomes the new root of the tree. Use `u` or `Backspace` to go back up.

### Example Navigation Flow

```bash
# Start from home
dt ~

# Navigate to Downloads
# Press 'j' until Downloads is selected
# Press 'l' to expand
# Press 'j' to navigate into it
# Press 'Enter' to make it the root

# Go back to parent
# Press 'u'
```

## File Operations

### Viewing Files

1. **Enable file viewer mode**: Press `s`
2. **Navigate to a file**: Use `j`/`k`
3. **Preview appears** in the right panel automatically

### Scrolling File Preview

```
Ctrl+j        Scroll down in preview by line
Ctrl+k        Scroll up in preview by line
Page Down     Scroll down in preview by page
Page Up       Scroll up in preview by page
Home          Jump to start of file
End           Jump to end of file
```

### Fullscreen File View

```
v             Open selected file in fullscreen viewer
```

In fullscreen mode:

```
j/k or ↓/↑    Scroll by line
Page Up/Down  Scroll by page
Home          Switch to HEAD mode (first 10K lines)
End           Switch to TAIL mode (last 10K lines)
l             Toggle line numbers
q             Return to tree view
Esc           Exit dtree completely
```

### Opening in External Editor

```
e             Open file in configured editor
```

The editor is set in `config.toml` (default: nano).

### Opening in File Manager

```
o             Open in configured file manager
```

- For files: opens parent directory
- For directories: opens the directory itself

## Searching

### Tree Search

Search for files and directories in the tree:

```
/             Enter search mode
```

Type your query and press `Enter`. Results appear in a panel at the bottom.

#### Normal Search

```
/             Enter search mode
main          Type query (finds "main.rs", "main.cpp", etc.)
Enter         Execute search
```

Matches any file/directory containing "main" (case-insensitive).

#### Fuzzy Search

```
/             Enter search mode
/mncpp        Type query with '/' prefix (finds "main.cpp", "menu_cpp.rs", etc.)
Enter         Execute search
```

Results are ranked by relevance score.

#### Navigating Results

```
Tab           Switch focus between tree and results
j/k or ↓/↑    Navigate through results
Enter         Jump to selected result in tree
Esc           Close search and results
```

### File Search (Fullscreen Only)

Search within the current file:

```
v             Open file in fullscreen
/             Enter file search mode
error         Type query
Enter         Execute search
```

Navigate through matches:

```
n             Next match
N             Previous match (Shift+n)
Esc           Clear search
```

The title bar shows match count: "Match 3/15"

## Bookmarks

### Creating Bookmarks

```
m             Enter bookmark creation mode
myproject     Type bookmark name
Enter         Save bookmark
Esc           Cancel
```

**Note**: Bookmarks save directories only. If cursor is on a file, the parent directory is saved.

### Using Bookmarks

#### Inside dtree

```
'             Open bookmark selection (apostrophe/tick)
```

Two modes available:

**Navigation mode** (default):
```
j/k           Navigate bookmarks
Enter         Jump to selected bookmark
d             Mark for deletion (press twice to confirm)
Tab           Switch to filter mode
```

**Filter mode**:
```
type text     Filter bookmarks by name/path
Tab           Switch back to navigation mode
Enter         Jump to selected bookmark
```

#### From Command Line

```bash
# Jump to bookmark
dt myproject

# List bookmarks
dt -bm
dt -bm list

# Add bookmark
dt -bm add work /path/to/work

# Remove bookmark
dt -bm remove work
```

## Directory Sizes

Toggle directory size display:

```
z             Toggle size display
```

When enabled:

- Sizes are calculated asynchronously in the background
- Shows "calc." while calculating
- Shows total size when done (e.g., "1.2M", "350K")
- Partial results prefixed with ">" (e.g., ">5.0G")

In file viewer mode, also shows individual file sizes.

**Performance limits**:
- 5-second timeout per directory
- 10,000 files maximum per directory

## Copying Paths

```
c             Copy current path to clipboard
```

Works with both files and directories. Requires clipboard support (xclip on Linux).

## Help System

```
i             Toggle help screen
```

Shows all available keybindings for the current mode.

Press `i` again to return to the tree.

## Exiting

```
q             Exit and cd to selected directory (with bash wrapper)
Esc           Exit without directory change
```

## Mouse Support

### Clicking

```
Click         Select item
Double-click  Expand/collapse directory
```

### Scrolling

```
Scroll wheel  Navigate tree (when mouse over tree area)
Scroll wheel  Scroll preview (when mouse over preview area)
```

### Resizing

```
Drag          Grab and drag the vertical divider to resize panels
```

### Text Selection (Fullscreen Only)

```
Shift+Click+Drag    Select text for copying
Ctrl+Shift+C        Copy selected text (terminal shortcut)
```

## Tips for Efficient Usage

### Tip 1: Stay in Keyboard Mode

You can accomplish everything without using the mouse:

- Navigate with `hjkl` (Vim-style)
- Search with `/`
- Bookmark with `m` and `'`
- Toggle views with `s`, `i`, `z`

### Tip 2: Use Bookmarks for Projects

Create bookmarks for frequently-accessed directories:

```bash
dt ~/projects/webapp
# Press 'm', name it 'webapp'

# Later:
dt webapp  # Instant navigation
```

### Tip 3: Search Everywhere

The search feature searches the entire tree, including collapsed directories:

```
/config       Finds all files/dirs with "config" in the name
//.git        Fuzzy search for ".git" directories
```

### Tip 4: Monitor Log Files

Use TAIL mode for log files:

```bash
dt -v /var/log/syslog
# Press 'End' to switch to TAIL mode
# Press '/' to search for errors
# Navigate with 'n' and 'N'
```

### Tip 5: Navigate Between Files

In fullscreen mode, use `Ctrl+j`/`Ctrl+k` to jump between files in the same directory without returning to the tree view.

## Common Workflows

### Workflow 1: Code Exploration

```bash
dt ~/my-project    # Open project
s                  # Enable file viewer
/                  # Search for file
config             # Type query
Enter              # Execute search
j                  # Navigate to result
Enter              # Jump to file in tree
v                  # View in fullscreen
/                  # Search within file
TODO               # Find TODOs
n                  # Next match
```

### Workflow 2: Directory Cleanup

```bash
dt ~/Downloads     # Open Downloads
z                  # Enable size display
                   # Wait for calculation
                   # Navigate to large dirs
l                  # Expand to drill down
                   # Identify files to delete
o                  # Open in file manager to delete
```

### Workflow 3: Quick Navigation

```bash
dt                 # Open from anywhere
'                  # Open bookmarks
j                  # Select bookmark
Enter              # Jump to location
q                  # Exit and cd there
```

## Next Steps

- [Key Bindings](./keybindings.md) - Complete keybinding reference
- [Features](./features.md) - Detailed feature documentation
- [Configuration](./configuration.md) - Customize dtree
