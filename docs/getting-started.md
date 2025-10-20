# Getting Started

This guide will help you get up and running with dtree in just a few minutes.

## Quick Start

After installation, you can start using dtree immediately:

```bash
# Launch dtree from current directory
dt

# Navigate to a specific directory
dt /path/to/directory

# View a file in fullscreen mode
dt -v README.md
```

## First Launch

When you launch dtree for the first time:

1. The TUI will open showing the current directory tree
2. A configuration file will be created at `~/.config/dtree/config.toml`
3. Use arrow keys or `j`/`k` to navigate
4. Press `i` to see the help screen
5. Press `q` to exit

## Basic Navigation

### Moving Around

```
j or ↓    Move down
k or ↑    Move up
l or →    Expand directory
h or ←    Collapse directory
Enter     Change root to selected directory
u         Go to parent directory
```

### Viewing Files

```
s         Toggle file viewer mode
v         Open file in fullscreen viewer
Ctrl+j/k  Scroll file preview
```

### Searching

```
/         Enter search mode
          Type your query
Enter     Execute search
Tab       Switch focus between tree and results
Esc       Exit search mode
```

## Your First Workflow

Let's walk through a typical workflow:

### 1. Navigate to a Project

```bash
# Start from home directory
dt ~

# Expand directories with 'l'
# Navigate with 'j' and 'k'
# Press Enter on your project directory
```

### 2. Find a File

```bash
# Press 's' to enable file viewer mode
# Press '/' to enter search mode
# Type the filename (e.g., 'main')
# Press Enter to search
# Use j/k to navigate results
# Press Enter to jump to file
```

### 3. View the File

```bash
# With file selected, press 'v' for fullscreen view
# Use j/k to scroll
# Press '/' to search within the file
# Press 'q' to return to tree view
```

### 4. Create a Bookmark

```bash
# Navigate to a frequently-used directory
# Press 'm' to create bookmark
# Type a name (e.g., 'myproject')
# Press Enter to save
```

### 5. Use the Bookmark

```bash
# Next time, from terminal:
dt myproject

# Or inside dtree:
# Press ' (apostrophe)
# Select your bookmark
# Press Enter to jump
```

## Essential Tips

### Tip 1: Use the Bash Wrapper

The `dt` wrapper function makes navigation seamless:

```bash
# Instead of:
cd /long/path/to/directory

# Use:
dt    # Navigate visually, press Enter on target, automatic cd
```

### Tip 2: Keyboard-First Workflow

You can accomplish everything without touching the mouse:

- Navigate with `hjkl` (Vim-style)
- Search with `/`
- Bookmark with `m`
- Toggle views with `s`, `i`, `z`

### Tip 3: Learn the Context

Different modes have different keybindings:

- **Tree mode**: Navigate directories
- **Search mode**: Type query, navigate results
- **Fullscreen mode**: View files, search within file
- **Bookmark mode**: Manage bookmarks

Press `i` in any mode to see available keys.

### Tip 4: Customize Your Config

Edit `~/.config/dtree/config.toml` to:

- Change colors
- Set default editor and file manager
- Enable file icons
- Adjust file preview limits
- Configure keybindings

### Tip 5: Use Fuzzy Search

For faster searching, use fuzzy mode:

```
/           Enter search mode
/fuz        Fuzzy search (finds "fuzzy.rs", "file_utils.rs", etc.)
/src/main   Fuzzy search with path
```

Results are ranked by relevance score.

## Common Use Cases

### Use Case 1: Project Navigation

```bash
# Create bookmarks for all your projects
dt ~/projects/web-app
# Press 'm', name it 'webapp'

dt ~/projects/backend
# Press 'm', name it 'backend'

# Later, jump instantly:
dt webapp
dt backend
```

### Use Case 2: Log File Monitoring

```bash
# View a log file
dt -v /var/log/app.log

# Press 'End' to switch to TAIL mode (see last 10K lines)
# Press '/' to search for errors
# Type 'error' and press Enter
# Navigate with 'n' and 'N'
```

### Use Case 3: Code Exploration

```bash
# Navigate to project
dt ~/my-project

# Enable file viewer with 's'
# Navigate tree with j/k/h/l
# Preview files automatically in right panel
# Search with '/' to find specific files
# Press 'v' on interesting file to view fullscreen
# Press 'e' to open in editor
```

### Use Case 4: Directory Size Analysis

```bash
# Navigate to directory
dt ~/Downloads

# Press 'z' to enable size display
# Wait for async calculation
# Identify large directories
# Navigate and repeat to drill down
```

## What's Next?

- [Installation](./installation.md) - Installation options
- [Configuration](./configuration.md) - Customize dtree
- [Features](./features.md) - Explore all features
- [Key Bindings](./keybindings.md) - Complete keybinding reference

## Getting Help

- Press `i` inside dtree to see the help screen
- Run `dtree -h` for command-line help
- Check [Troubleshooting](./troubleshooting.md) for common issues
