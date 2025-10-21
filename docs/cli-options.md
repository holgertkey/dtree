# CLI Reference

Complete command-line interface reference for dtree.

## Table of Contents

- [Synopsis](#synopsis)
- [Commands](#commands)
- [Options](#options)
- [Examples](#examples)
- [Exit Codes](#exit-codes)
- [Environment Variables](#environment-variables)

---

## Synopsis

```bash
dtree [OPTIONS] [PATH|BOOKMARK]
dt [PATH|BOOKMARK|COMMAND]
```

**Note**: `dt` is the recommended bash wrapper (see [Bash Integration](./bash-integration.md)).

---

## Commands

### Interactive Navigation

```bash
# Launch interactive tree from current directory
dtree
dt

# Navigate to specific directory
dtree /path/to/directory
dt /path/to/directory

# Jump to bookmark
dtree myproject
dt myproject

# Return to previous directory (bash wrapper only)
dt -
```

**Behavior**:
- Opens interactive TUI if no valid bookmark/path provided
- With bash wrapper (`dt`), changes shell directory on exit
- Without wrapper, prints path to stdout (for integration with other tools)

### File Viewing

```bash
# View file in fullscreen mode
dtree -v FILE
dtree --view FILE
dt -v FILE

# Examples
dt -v README.md
dt -v /var/log/syslog
dtree --view src/main.rs
```

**Behavior**:
- Opens file in fullscreen viewer immediately
- Press `q` to exit to tree view (navigate to file's directory)
- Press `Esc` to exit dtree completely
- Supports syntax highlighting for code files
- Automatically detects and handles binary files

### Bookmark Management

```bash
# List all bookmarks
dtree -bm
dtree -bm list
dt -bm
dt -bm list

# Add bookmark for current directory
dtree -bm add NAME
dt -bm add NAME

# Add bookmark for specific path
dtree -bm add NAME PATH
dt -bm add NAME PATH

# Remove bookmark
dtree -bm remove NAME
dt -bm remove NAME

# Examples
dt -bm add work
dt -bm add myproject ~/projects/my-project
dt -bm remove work
```

**Bookmark Names**:
- Must be alphanumeric, hyphens, or underscores
- Cannot contain path separators (`/`, `\`)
- Cannot use reserved names: `-`, `.`, `..`
- Case-sensitive

**Storage**:
- Bookmarks saved to: `~/.config/dtree/bookmarks.json`
- Auto-created on first use
- Persists across sessions

### Help and Version

```bash
# Show help
dtree -h
dtree --help

# Show version
dtree --version
```

---

## Options

### `-h, --help`

Print help information and exit.

```bash
dtree -h
dtree --help
```

### `-v, --view FILE`

Open file in fullscreen viewer mode.

```bash
dtree -v README.md
dtree --view /var/log/syslog
```

**Features**:
- Syntax highlighting for code files
- Line numbers (toggle with `l`)
- Search within file (press `/`)
- HEAD/TAIL modes for large files
- Binary file support (opens hex editor with `e`)

### `--version`

Print version information and exit.

```bash
dtree --version
```

Output format: `dtree X.Y.Z`

### `-bm, --bm [SUBCOMMAND]`

Bookmark management mode.

**Subcommands**:
- `list` - List all bookmarks (default if no subcommand)
- `add NAME [PATH]` - Add bookmark
- `remove NAME` - Remove bookmark

```bash
dtree -bm list
dtree -bm add myproject
dtree -bm add myproject /path/to/project
dtree -bm remove myproject
```

### `[PATH|BOOKMARK]`

Optional positional argument for navigation.

**Resolution Priority**:
1. **Bookmark** - If bookmark with this name exists
2. **Directory Path** - If valid directory path
3. **File Path** - If valid file path (opens parent directory)
4. **Error** - If none of the above

```bash
# Bookmark (if exists)
dt myproject

# Absolute path
dt /home/user/projects

# Relative path
dt ../other-project

# File (navigates to parent directory)
dt README.md
```

---

## Examples

### Basic Navigation

```bash
# Open dtree in current directory
dt

# Navigate to /var/log
dt /var/log

# Navigate to home directory
dt ~

# Navigate to parent directory
dt ..
```

### Using Bookmarks

```bash
# Save current directory as bookmark
dt                          # Open dtree
# Press 'm', type 'work', press Enter
# Press 'q' to exit

# Jump to bookmark
dt work

# Or use CLI
dt -bm add work
dt work
```

### File Viewing

```bash
# View system log
dt -v /var/log/syslog

# View source code with syntax highlighting
dt -v src/main.rs

# View large log file (use tail mode)
dt -v /var/log/nginx/access.log
# Press 'End' for tail mode (last 10K lines)
```

### Workflow Examples

**Example 1: Quick Project Navigation**
```bash
# Save project locations
dt -bm add frontend ~/projects/app-frontend
dt -bm add backend ~/projects/app-backend
dt -bm add docs ~/projects/documentation

# Jump between projects
dt frontend
# ... work on frontend ...
dt backend
# ... work on backend ...
dt -
# Return to frontend
```

**Example 2: Log Inspection**
```bash
# View latest log entries
dt -v /var/log/syslog
# Press 'End' for tail mode
# Press '/' to search
# Type 'error', press Enter
# Press 'n' for next match
```

**Example 3: Code Exploration**
```bash
# Open project
dt ~/projects/myapp

# Inside dtree:
# Press 's' to enable file viewer
# Press '/' to search for files
# Type 'main', press Enter
# Press 'v' to view in fullscreen
# Press 'e' to edit in $EDITOR
```

---

## Exit Codes

dtree uses standard UNIX exit codes:

| Code | Meaning                                                        |
|------|----------------------------------------------------------------|
| `0`  | Success - Normal exit                                          |
| `1`  | Error - General error (invalid arguments, file not found, etc.)|
| `2`  | Error - Invalid usage (wrong arguments)                        |

**Examples**:
```bash
# Success
dt /home/user
echo $?  # 0

# Error: directory doesn't exist
dt /nonexistent/path
echo $?  # 1

# Error: invalid option
dtree --invalid-option
echo $?  # 2
```

---

## Environment Variables

### `DTREE_PREV_DIR`

Used by bash wrapper to track previous directory for `dt -` command.

**Set by**: Bash wrapper after each successful navigation
**Format**: Absolute directory path
**Usage**: Internal - don't modify manually

```bash
# After navigation
dt ~/projects
echo $DTREE_PREV_DIR  # /home/user (previous location)

# Return to previous
dt -
echo $PWD             # /home/user
echo $DTREE_PREV_DIR  # /home/user/projects
```

### `EDITOR`

Default text editor for `e` key (if not configured in `config.toml`).

```bash
export EDITOR=vim
dt -v README.md
# Press 'e' to open in vim
```

**Priority**:
1. `editor` in `~/.config/dtree/config.toml`
2. `$EDITOR` environment variable
3. Default: `nano`

---

## Configuration Files

### `~/.config/dtree/config.toml`

Main configuration file (auto-created on first run).

See [Configuration Reference](./configuration.md) for details.

### `~/.config/dtree/bookmarks.json`

Bookmark storage (auto-created when first bookmark is added).

**Format**:
```json
{
  "myproject": {
    "path": "/home/user/projects/my-project",
    "name": "my-project"
  }
}
```

**Backup recommendation**: Include in dotfiles backup

---

## Integration with Other Tools

### Shell Integration (Bash)

See [Bash Integration Guide](./bash-integration.md) for `dt` wrapper setup.

### Script Integration

Use `dtree` (not `dt`) in scripts:

```bash
#!/bin/bash
# Select directory and process files
selected=$(dtree /path/to/start | tail -n1)
if [ -n "$selected" ] && [ -d "$selected" ]; then
  cd "$selected"
  # Process files...
fi
```

### Combining with Other Commands

```bash
# Find and navigate to directory
dt $(find ~ -type d -name "projects" | head -1)

# View most recent log
dt -v $(ls -t /var/log/*.log | head -1)

# Add multiple bookmarks from list
while read -r name path; do
  dt -bm add "$name" "$path"
done < bookmarks.txt
```

---

## Comparison with Similar Tools

### vs `tree`

```bash
# tree - static output
tree /path

# dtree - interactive navigation
dt /path
```

**Advantages**:
- Interactive navigation
- File preview
- Bookmarks
- Search functionality

### vs `ranger`/`nnn`

```bash
# ranger - full file manager
ranger

# dtree - focused on directory navigation
dt
```

**Advantages over full file managers**:
- Lighter and faster
- Better for quick navigation
- Seamless shell integration
- Focused feature set

**When to use ranger/nnn instead**:
- Need file operations (copy, move, delete)
- Want tabs/multi-pane support
- Prefer full file manager features

---

## Troubleshooting

### Bookmark Not Found

```bash
$ dt myproject
Error: bookmark 'myproject' not found and path does not exist
```

**Solution**:
```bash
# List existing bookmarks
dt -bm list

# Create bookmark if needed
dt -bm add myproject /path/to/project
```

### Permission Denied

```bash
$ dt /root
Error: Permission denied
```

**Solution**: Use `sudo` or navigate to accessible directories only.

### Command Not Found

```bash
$ dt
bash: dt: command not found
```

**Solution**: Add bash wrapper to `~/.bashrc` (see [Installation](./installation.md)).

### File Not Found in Viewer

```bash
$ dt -v nonexistent.txt
Error: File not found: nonexistent.txt
```

**Solution**: Check file path and permissions.

---

## See Also

- [Getting Started](./getting-started.md) - Quick start guide
- [Usage Guide](./usage.md) - Interactive usage
- [Key Bindings](./keybindings.md) - Keyboard shortcuts
- [Configuration](./configuration.md) - Config file reference
- [Bash Integration](./bash-integration.md) - Shell wrapper setup

---

## Appendix: Command Summary

### One-line Command Reference

```bash
# Navigation
dt                              # Open interactive tree
dt PATH                         # Navigate to directory
dt BOOKMARK                     # Jump to bookmark
dt -                            # Previous directory

# File viewing
dt -v FILE                      # View file

# Bookmarks
dt -bm                          # List bookmarks
dt -bm add NAME [PATH]          # Add bookmark
dt -bm remove NAME              # Remove bookmark

# Help
dtree -h                        # Show help
dtree --version                 # Show version
```

### Interactive Mode Quick Reference

Inside dtree TUI:

```
j/k ↓/↑         Navigate
l/h →/←         Expand/collapse
Enter           Enter directory
u Backspace     Parent directory
s               Toggle file viewer
v               Fullscreen viewer
/               Search
m               Create bookmark
'               Select bookmark
e               Open in editor
o               File manager
c               Copy path
i               Help
z               Toggle sizes
q               Exit (cd)
Esc             Exit (no cd)
```
