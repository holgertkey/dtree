# Key Bindings

Complete reference of all keyboard shortcuts in dtree.

## Global Keys

These keys work in most modes:

| Key   | Action                                  |
|-------|-----------------------------------------|
| `i`   | Toggle help screen                      |
| `Esc` | Exit dtree (or cancel current operation)|

## Tree Navigation Mode

This is the default mode when you first launch dtree.

### Movement

| Key        | Action              |
|------------|---------------------|
| `j` or `↓` | Move down one item  |
| `k` or `↑` | Move up one item    |
| `Home`     | Jump to first item  |
| `End`      | Jump to last item   |

### Directory Operations

| Key                | Action                                    |
|--------------------|-------------------------------------------|
| `l` or `→`         | Expand directory (show children)          |
| `h` or `←`         | Collapse directory (hide children)        |
| `Enter`            | Change root to selected directory         |
| `u` or `Backspace` | Go to parent directory (change root)      |

### View Toggles

| Key | Action                                             |
|-----|----------------------------------------------------|
| `s` | Toggle file viewer mode (show/hide files in split) |
| `v` | Open file in fullscreen viewer (files only)        |
| `i` | Toggle help screen                                 |
| `z` | Toggle directory size display                      |

### File Operations

| Key | Action                                            |
|-----|---------------------------------------------------|
| `e` | Open file/directory in external editor            |
| `o` | Open in file manager (files → parent, dirs → self)|
| `c` | Copy current path to clipboard                    |

### Search and Bookmarks

| Key | Action                                       |
|-----|----------------------------------------------|
| `/` | Enter search mode (tree search)              |
| `m` | Create bookmark (mark current location)      |
| `'` | Open bookmark selection menu (apostrophe)    |

### Exit

| Key   | Action                                        |
|-------|-----------------------------------------------|
| `q`   | Exit and cd to selected directory (with bash) |
| `Esc` | Exit without directory change                 |

## File Viewer Mode

When file viewer is enabled (press `s`):

### Preview Scrolling

| Key         | Action                            |
|-------------|-----------------------------------|
| `Ctrl+j`    | Scroll preview panel down by line |
| `Ctrl+k`    | Scroll preview panel up by line   |
| `Page Down` | Scroll preview panel down by page |
| `Page Up`   | Scroll preview panel up by page   |
| `Home`      | Jump to start of file             |
| `End`       | Jump to end of file               |

All tree navigation keys still work normally.

## Fullscreen Viewer Mode

When viewing a file in fullscreen (press `v`):

### Navigation

| Key         | Action                                   |
|-------------|------------------------------------------|
| `j` or `↓`  | Scroll down by line                      |
| `k` or `↑`  | Scroll up by line                        |
| `Ctrl+j`    | Jump to next file in directory           |
| `Ctrl+k`    | Jump to previous file in directory       |
| `Page Down` | Scroll down by page                      |
| `Page Up`   | Scroll up by page                        |
| `Home`      | Switch to HEAD mode (first 10,000 lines) |
| `End`       | Switch to TAIL mode (last 10,000 lines)  |

### View Options

| Key | Action                               |
|-----|--------------------------------------|
| `l` | Toggle line numbers (show/hide)      |
| `w` | Toggle line wrapping (wrap/truncate) |

### File Search

| Key   | Action                                              |
|-------|-----------------------------------------------------|
| `/`   | Enter file search mode (search within current file) |
| `n`   | Next search match (when results exist)              |
| `N`   | Previous search match (Shift+n)                     |
| `Esc` | Clear search (or exit if no search active)          |

### Exit

| Key   | Action                                             |
|-------|----------------------------------------------------|
| `q`   | Return to tree view (stay in program)              |
| `Esc` | Exit program completely (or clear search if active)|

**Note**: In fullscreen mode, most tree keys are disabled. Only the keys listed above work.

## Search Mode

When you press `/` to enter search mode:

### Input

| Key             | Action                            |
|-----------------|-----------------------------------|
| Type characters | Add to search query               |
| `Backspace`     | Remove last character             |
| `Enter`         | Execute search and show results   |
| `Esc`           | Cancel search (exit search mode)  |

### Fuzzy Search

Start your query with `/` to enable fuzzy matching:

```
/fuz        Fuzzy search (finds "fuzzy.rs", "file_utils.rs", etc.)
/src/main   Fuzzy search with path components
```

Results are ranked by relevance score.

### Search Results Navigation

After executing a search:

| Key        | Action                                      |
|------------|---------------------------------------------|
| `Tab`      | Switch focus between tree and results panel |
| `j` or `↓` | Navigate down in results                    |
| `k` or `↑` | Navigate up in results                      |
| `Enter`    | Jump to selected result in tree             |
| `Esc`      | Close results panel and exit search mode    |

## Bookmark Creation Mode

When you press `m` to create a bookmark:

### Input

| Key                  | Action                                      |
|----------------------|---------------------------------------------|
| Type characters      | Add to bookmark name                        |
| `Backspace`          | Remove last character                       |
| `Enter`              | Save bookmark                               |
| `Esc`                | Cancel bookmark creation                    |
| `Ctrl+j` or `Ctrl+↓` | Scroll down through existing bookmarks list |
| `Ctrl+k` or `Ctrl+↑` | Scroll up through existing bookmarks list   |

### Naming Rules

- Alphanumeric characters, hyphens, and underscores allowed
- No path separators (`/`, `\`)
- No control characters
- Cannot use reserved names (`-`, `.`, `..`)

## Bookmark Selection Mode

When you press `'` to open bookmark selection:

### Navigation Mode (Default)

| Key        | Action                                             |
|------------|----------------------------------------------------|
| `j` or `↓` | Move selection down                                |
| `k` or `↑` | Move selection up                                  |
| `Ctrl+j`   | Scroll down in list                                |
| `Ctrl+k`   | Scroll up in list                                  |
| `Enter`    | Jump to selected bookmark                          |
| `d`        | Mark bookmark for deletion (press twice to confirm)|
| `Tab`      | Switch to filter mode                              |
| `Esc`      | Close bookmark selection                           |

### Filter Mode

| Key             | Action                                        |
|-----------------|-----------------------------------------------|
| Type characters | Filter bookmarks by name/path                 |
| `Backspace`     | Remove last character from filter             |
| `Tab`           | Switch back to navigation mode (keeps filter) |
| `j` or `↓`      | Navigate filtered results (in navigation mode)|
| `k` or `↑`      | Navigate filtered results (in navigation mode)|
| `Enter`         | Jump to selected bookmark                     |
| `Esc`           | Close bookmark selection                      |

### Deletion Workflow

Two-phase deletion (prevents accidental deletion):

1. Press `d` once → Bookmark marked with red `[DEL]` prefix
2. Press `d` again → Bookmark deleted
3. Navigate with `j`/`k` → Mark is cleared (cancels deletion)

## File Search Mode (Fullscreen Only)

When you press `/` in fullscreen viewer:

### Input

| Key             | Action                                 |
|-----------------|----------------------------------------|
| Type characters | Add to search query                    |
| `Backspace`     | Remove last character                  |
| `Enter`         | Execute search and jump to first match |
| `Esc`           | Cancel and exit file search mode       |

### After Search

| Key   | Action                                     |
|-------|--------------------------------------------|
| `n`   | Next match (cycles to first after last)    |
| `N`   | Previous match (cycles to last after first)|
| `Esc` | Clear search results                       |

Match counter appears in title: "Match 3/15"

## Visual Selection Mode (Fullscreen Only)

Vim-style line selection for copying large blocks of text.

When you press `V` in fullscreen viewer to enter visual selection mode:

### Selection Navigation

| Key         | Action                                            |
|-------------|---------------------------------------------------|
| `j` or `↓`  | Expand selection downward (move cursor down)      |
| `k` or `↑`  | Expand selection upward (move cursor up)          |
| `Page Down` | Jump selection down by page                       |
| `Page Up`   | Jump selection up by page                         |
| `Home`      | Jump to start of file                             |
| `End`       | Jump to end of file                               |
| Scroll wheel| Move cursor (with auto-scroll at edges)           |

### Actions

| Key         | Action                                            |
|-------------|---------------------------------------------------|
| `V`         | Enter visual selection mode                       |
| `y` or `Y`  | Copy selected lines to clipboard and exit         |
| `V` or `Esc`| Exit visual mode without copying                  |

### Visual Feedback

- **Selected lines**: Gray background
- **Cursor position**: Blue background
- **Status bar**: Shows selection size (e.g., "VISUAL: 25 lines")
- **Title bar**: Displays `[VISUAL MODE]` indicator

### Features

- **Bidirectional**: Select up or down from start point
- **Multi-page**: Select across hundreds of lines
- **Auto-scroll**: Cursor scrolls view when reaching edges
- **Keyboard-driven**: No mouse needed
- **Mouse support**: Scroll wheel moves cursor in visual mode

### Example Workflow

```
1. Press v to view a log file in fullscreen
2. Press V to enter visual selection mode
3. Press j/j/j to expand selection downward
4. Press Page Down to jump to next page
5. Press y to copy all selected lines
6. Exit and paste with Ctrl+Shift+V
```

### Use Cases

- Copying log sections for bug reports
- Extracting configuration blocks
- Saving command output
- Selecting multi-page code sections

**Note**: Visual mode is only available in fullscreen viewer (`v`), not in split-view mode.

## Mouse Bindings

### Tree View

| Action                  | Effect                                |
|-------------------------|---------------------------------------|
| Click                   | Select item under cursor              |
| Double-click            | Expand/collapse directory             |
| Scroll wheel            | Navigate tree up/down                 |
| Drag vertical divider   | Resize tree/preview panels            |
| Drag horizontal divider | Resize bottom panel (search/bookmarks)|

### File Preview

| Action       | Effect                 |
|--------------|------------------------|
| Scroll wheel | Scroll preview content |

### Fullscreen Viewer

| Action           | Effect                                          |
|------------------|-------------------------------------------------|
| Scroll wheel     | Scroll document (or move cursor in visual mode) |
| Shift+Click+Drag | Select text for copying (traditional)           |
| Ctrl+Shift+C     | Copy selected text (terminal shortcut)          |

**Note**:
- Regular clicks are ignored in fullscreen mode (view-only)
- In visual mode (`V`), scroll wheel moves the selection cursor with auto-scroll

## Context-Specific Behavior

### `Esc` Key Behavior

The `Esc` key has context-aware behavior:

| Context                          | Action                        |
|----------------------------------|-------------------------------|
| Tree mode                        | Exit dtree                    |
| Search mode                      | Cancel search                 |
| Search results                   | Close results panel           |
| File search mode (fullscreen)    | Cancel search input           |
| File search results (fullscreen) | Clear search results          |
| Visual selection mode            | Exit visual mode (no copy)    |
| No search results (fullscreen)   | Exit dtree                    |
| Bookmark creation                | Cancel creation               |
| Bookmark selection               | Close selection               |

### `Enter` Key Behavior

| Context                 | Action                                 |
|-------------------------|----------------------------------------|
| Tree mode, on directory | Change root to directory               |
| Search mode             | Execute search                         |
| Search results          | Jump to selected result                |
| File search mode        | Execute search and jump to first match |
| Bookmark creation       | Save bookmark                          |
| Bookmark selection      | Jump to selected bookmark              |

## Keybinding Conflicts

Some terminal emulators intercept certain key combinations. If a keybinding doesn't work:

### Common Conflicts

- **`Ctrl+j`/`Ctrl+k`**: Some terminals map these to Enter/other keys
- **Mouse support**: Enable mouse reporting in terminal settings

### Workarounds

If `Ctrl+j`/`Ctrl+k` don't work for scrolling:

- Use alternative keys: `Page Up`/`Page Down` for page-based scrolling
- Use `Home`/`End` to jump to start/end of file
- Check terminal emulator settings
- Try a different terminal (Alacritty, Kitty, WezTerm work well)

## Keybinding Customization

Keybindings can be customized in `~/.config/dtree/config.toml` in the `[keybindings]` section:

```toml
[keybindings]
quit = ["q", "Esc"]
search = ["/"]
toggle_files = ["s"]
toggle_help = ["i"]
copy_path = ["c"]
open_editor = ["e"]
open_file_manager = ["o"]
create_bookmark = ["m"]
select_bookmark = ["'"]
show_line_numbers = ["l"]
toggle_wrap = ["w"]

# Visual selection mode (fullscreen viewer only)
visual_mode = ["V"]          # Enter/exit visual selection mode (Shift+V)
visual_copy = ["y", "Y"]     # Copy selected lines to clipboard and exit
```

**Notes:**
- Each binding accepts a list of keys
- Multiple keys can trigger the same action
- Some keys are hardcoded for navigation (arrow keys, j/k/h/l)
- Visual mode keybindings only work in fullscreen viewer

**Example customizations:**
```toml
# Use 'v' (lowercase) to enter visual mode instead of 'V'
visual_mode = ["v"]

# Use Space to copy selection
visual_copy = ["Space", "y"]

# Multiple keys for the same action
quit = ["q", "Esc", "Q"]
```

## Quick Reference Card

### Essential Keys

```
Navigation:       j/k (down/up)  h/l (collapse/expand)
File viewing:     s (toggle)     v (fullscreen)
Search:           / (search)     m (bookmark)  ' (bookmarks)
Actions:          e (editor)     o (file mgr)  c (copy)
Modes:            i (help)       z (sizes)
Exit:             q (exit+cd)    Esc (exit)
```

### Fullscreen Keys

```
Navigate:         j/k (scroll)   Ctrl+j/k (next/prev file)
Modes:            Home (HEAD)    End (TAIL)  l (line #)  w (wrap)
Search:           / (search)     n/N (next/prev match)
Visual:           V (select)     j/k (expand)  y (copy)
Exit:             q (tree)       Esc (quit)
```

### Search Keys

```
Input:            type query     Backspace (delete)
Execute:          Enter          Tab (focus)
Results:          j/k (nav)      Enter (jump)
Exit:             Esc
```

### Bookmark Keys

```
Create:           m              type name     Enter
Select:           '              j/k (nav)     Enter
Filter:           Tab            type text     Tab
Delete:           d d            (press twice)
```

## Vim-Style Alternatives

If you're familiar with Vim, these mappings feel natural:

| Vim      | dtree      | Action                                      |
|----------|------------|---------------------------------------------|
| `j`      | `j`        | Down                                        |
| `k`      | `k`        | Up                                          |
| `h`      | `h`        | Left/Collapse                               |
| `l`      | `l`        | Right/Expand (or line numbers in fullscreen)|
| `/`      | `/`        | Search                                      |
| `n`      | `n`        | Next search result                          |
| `N`      | `N`        | Previous search result                      |
| `V`      | `V`        | Visual line mode (fullscreen only)          |
| `y`      | `y`        | Yank (copy) selection                       |
| `gg`     | `Home`     | Go to top                                   |
| `G`      | `End`      | Go to bottom                                |
| `:e`     | `e`        | Open in editor                              |

## Next Steps

- [Usage Guide](./usage.md) - Learn how to use these keys effectively
- [Configuration](./configuration.md) - Customize colors and settings
- [Features](./features.md) - Explore all features in detail
