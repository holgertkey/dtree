# Architecture

This document describes the internal architecture of dtree.

## Overview

dtree is built with a modular architecture that separates concerns into distinct modules. The codebase is organized around the Model-View-Controller pattern adapted for terminal UIs.

```
┌─────────────────────────────────────────────┐
│              main.rs                        │
│  (Entry point, terminal setup, CLI)        │
└──────────────────┬──────────────────────────┘
                   │
        ┌──────────▼──────────┐
        │       App           │
        │  (State manager)    │
        └──┬──┬──┬──┬──┬──┬──┘
           │  │  │  │  │  │
     ┌─────┘  │  │  │  │  └──────┐
     │        │  │  │  │         │
┌────▼───┐┌──▼──▼──▼──▼──┐  ┌───▼────┐
│  UI    ││  Modules     │  │ Event  │
│        ││  - Navigation│  │Handler │
│        ││  - FileView  │  │        │
│        ││  - Search    │  │        │
│        ││  - Bookmarks │  │        │
│        ││  - Config    │  │        │
└────────┘└──────────────┘  └────────┘
```

## Module Breakdown

### Core Modules

#### `main.rs` (Entry Point)

**Responsibilities**:
- Command-line argument parsing with clap
- Terminal lifecycle management (setup/cleanup)
- Panic hook installation for graceful crashes
- Entry point routing (TUI, direct navigation, file viewing)
- External program launching (editor, file manager, hex editor)
- Path and bookmark resolution

**Key Functions**:
- `main()` - Entry point
- `setup_terminal()` - Initialize terminal state
- `cleanup_terminal()` - Restore terminal state
- `run_app()` - Main event loop
- `open_in_editor()` - Launch external editor
- `open_in_file_manager()` - Launch external file manager
- `open_in_hex_editor()` - Launch hex editor for binary files
- `resolve_path_or_bookmark()` - Resolve user input to path

**Key Decisions**:
- Uses stderr for TUI, stdout for path output (enables bash wrapper)
- Panic hook ensures terminal is always restored
- 100ms event polling for async operations

#### `app.rs` (Application State)

**Responsibilities**:
- Central state management
- Orchestrates all submodules
- Delegates operations to specialized modules
- Minimal logic, mostly composition

**Size**: 74 lines (after refactoring from 1130 lines)

**Structure**:
```rust
pub struct App {
    nav: Navigation,
    file_viewer: FileViewer,
    search: Search,
    ui: UI,
    event_handler: EventHandler,
    config: Config,
    bookmarks: Bookmarks,
    show_files: bool,
    show_help: bool,
    fullscreen_viewer: bool,
    show_sizes: bool,
    dir_size_cache: DirSizeCache,
    need_terminal_clear: bool,
}
```

**Key Methods**:
- `new()` - Initialize application state
- `handle_key()` - Delegate key events
- `handle_mouse()` - Delegate mouse events
- `render()` - Delegate rendering
- `poll_search()` - Check for search updates
- `poll_sizes()` - Check for size calculation updates

### Data Structures

#### `tree_node.rs` (Tree Data Structure)

**Responsibilities**:
- Recursive tree structure for filesystem
- Lazy loading of directory contents
- Error tracking for inaccessible nodes
- Expand/collapse state management

**Structure**:
```rust
pub struct TreeNode {
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<TreeNodeRef>,
    pub expanded: bool,
    pub has_error: bool,
    pub error_message: Option<String>,
}

pub type TreeNodeRef = Rc<RefCell<TreeNode>>;
```

**Key Design Decisions**:
- Uses `Rc<RefCell<>>` for zero-copy operations
- Lazy loading: children loaded only when expanded
- Only shows directories by default (files loaded on demand)
- Error information stored in-band for graceful degradation

**Methods**:
- `new()` - Create node from path
- `load_children()` - Load directory contents
- `toggle_expand()` - Expand or collapse
- `is_expanded()` - Check expand state

### Functional Modules

#### `navigation.rs` (Tree Navigation)

**Size**: 194 lines

**Responsibilities**:
- Tree navigation logic
- Maintains root node and flat list of visible nodes
- Rebuilds flat list on tree changes
- Selection management
- Path-based tree operations

**Structure**:
```rust
pub struct Navigation {
    pub root: TreeNodeRef,
    pub flat_list: Vec<TreeNodeRef>,
    pub selected: usize,
    show_files: bool,
    show_hidden: bool,
    follow_symlinks: bool,
}
```

**Key Methods**:
- `new()` - Initialize from path
- `rebuild_flat_list()` - Flatten visible tree
- `move_up()` / `move_down()` - Navigate selection
- `toggle_node()` - Expand/collapse at path
- `expand_path_to_node()` - Expand path to specific node
- `reload_tree()` - Rebuild entire tree
- `go_to_parent()` - Navigate to parent directory

**Performance Notes**:
- Flat list rebuild is O(n) where n = visible nodes
- Uses `Rc<RefCell<>>` to avoid cloning nodes
- Rebuilds only on structural changes (expand/collapse/reload)

#### `file_viewer.rs` (File Content Display)

**Size**: 179 lines

**Responsibilities**:
- File content loading and formatting
- Syntax highlighting integration
- Scroll management
- Binary file detection
- File search within content
- HEAD/TAIL mode for large files

**Structure**:
```rust
pub struct FileViewer {
    pub content: Vec<String>,
    pub file_info: String,
    pub scroll: usize,
    pub show_line_numbers: bool,
    pub tail_mode: bool,
    pub search_mode: bool,
    pub search_query: String,
    pub search_results: Vec<usize>,
    pub current_match: usize,
}
```

**Key Methods**:
- `load_file_with_width()` - Load and format file
- `scroll_up()` / `scroll_down()` - Navigate content
- `enable_tail_mode()` / `disable_tail_mode()` - Toggle mode
- `enter_search_mode()` - Start file search
- `perform_search()` - Execute search
- `next_match()` / `prev_match()` - Navigate matches

**Binary File Detection**:
- Checks for NULL bytes in first 8KB
- Shows informational message instead of content
- Suggests hex editor for viewing

#### `search.rs` (Tree Search)

**Size**: ~390 lines

**Responsibilities**:
- Two-phase search (quick + deep)
- Fuzzy matching with scoring
- Result management
- Background thread coordination
- Progress tracking

**Structure**:
```rust
pub struct Search {
    pub mode: bool,
    pub query: String,
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub focus_on_results: bool,
    pub in_progress: bool,
    pub dirs_scanned: usize,
    result_receiver: Option<Receiver<SearchMessage>>,
    cancel_sender: Option<Sender<()>>,
    fuzzy_mode: bool,
}

pub struct SearchResult {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub score: Option<i64>,
    pub match_indices: Option<Vec<usize>>,
}
```

**Two-Phase Search Architecture**:

**Phase 1 (Quick)**:
- Searches already-loaded visible nodes
- Instant results
- No blocking

**Phase 2 (Deep)**:
- Background thread spawned
- Searches entire tree from root
- Sends results via channel
- Non-blocking UI
- Shows progress counter

**Fuzzy Matching**:
- Query starting with `/` activates fuzzy mode
- Uses `SkimMatcherV2` algorithm
- Results ranked by score
- Match indices tracked for highlighting

**Thread Communication**:
- `crossbeam-channel` for results (unbounded)
- `crossbeam-channel` for cancellation (bounded)
- Polled in main event loop

#### `ui.rs` (Rendering)

**Size**: 332 lines

**Responsibilities**:
- All rendering logic
- Layout management
- Widget composition
- Theme application
- Panel sizing and resizing

**Structure**:
```rust
pub struct UI {
    pub tree_area_start: u16,
    pub tree_area_end: u16,
    pub tree_area_top: u16,
    pub tree_area_height: u16,
    pub viewer_area_start: u16,
    pub viewer_area_top: u16,
    pub viewer_area_height: u16,
    pub terminal_width: u16,
    pub terminal_height: u16,
    pub split_position: u16,
    pub tree_scroll_offset: usize,
    pub bottom_panel_split_position: u16,
    pub bottom_panel_top: u16,
    pub bottom_panel_height: u16,
}
```

**Rendering Modes**:
- Tree-only view
- Split view (tree + file preview)
- Fullscreen file viewer
- Search results panel (bottom)
- Bookmarks panel (bottom)
- Help overlay

**Layout Strategy**:
- Uses ratatui's Layout system
- Calculates areas on each render
- Stores dimensions for mouse hit testing
- Supports dynamic resizing

#### `event_handler.rs` (Input Processing)

**Size**: 363 lines

**Responsibilities**:
- All keyboard event processing
- All mouse event processing
- Mode-specific input routing
- Mouse state tracking (dragging, double-click)

**Structure**:
```rust
pub struct EventHandler {
    pub dragging: bool,
    pub dragging_vertical: bool,
    pub last_click_time: Option<(Instant, usize)>,
    pub last_bookmark_click_time: Option<(Instant, usize)>,
    pub last_search_click_time: Option<(Instant, usize)>,
}
```

**Input Routing**:
```
handle_key()
  ├─> Search mode → handle_search_input()
  ├─> Bookmark selection → bookmark navigation
  ├─> Bookmark creation → bookmark input
  ├─> Fullscreen viewer → fullscreen keys
  │     └─> File search → handle_file_search_input()
  └─> Tree mode → tree navigation

handle_mouse()
  ├─> Click → handle_mouse_click()
  ├─> Drag → handle dragging
  ├─> Scroll → handle_scroll_up/down()
  └─> Release → stop dragging
```

**Double-Click Detection**:
- Tracks last click time and position
- Configurable timeout (default: 500ms)
- Separate tracking for tree/bookmarks/search

#### `config.rs` (Configuration Management)

**Size**: 328 lines

**Responsibilities**:
- TOML configuration file loading
- Auto-creation of default config
- Color parsing (names, hex, indexed)
- External program validation

**Structure**:
```rust
pub struct Config {
    pub appearance: AppearanceConfig,
    pub behavior: BehaviorConfig,
    pub keybindings: KeybindingsConfig,
}

pub struct AppearanceConfig {
    pub split_position: u16,
    pub show_icons: bool,
    pub show_line_numbers: bool,
    pub enable_syntax_highlighting: bool,
    pub syntax_theme: String,
    pub colors: ThemeConfig,
}
```

**Auto-Creation**:
- Creates `~/.config/dtree/` on first run
- Writes default config with extensive comments
- User can delete to reset

**Color Parsing**:
- Supports color names ("red", "cyan")
- Supports RGB hex ("#FF0000")
- Supports indexed colors (0-255)
- Validates and provides fallbacks

#### `bookmarks.rs` (Bookmark Management)

**Size**: ~200 lines

**Responsibilities**:
- Bookmark persistence (JSON)
- Interactive creation/selection modes
- Filter functionality
- Deletion with confirmation

**Structure**:
```rust
pub struct Bookmarks {
    bookmarks: Vec<Bookmark>,
    file_path: PathBuf,
    pub is_creating: bool,
    pub is_selecting: bool,
    pub is_filter_mode: bool,
    input_buffer: String,
    selected_index: usize,
    scroll_offset: usize,
    pending_deletion: Option<usize>,
}

pub struct Bookmark {
    pub key: String,
    pub path: PathBuf,
    pub name: Option<String>,
}
```

**Storage**:
- File: `~/.config/dtree/bookmarks.json`
- Auto-saves on changes
- Auto-loads on startup
- Graceful handling of missing file

#### `dir_size.rs` (Directory Size Calculation)

**Responsibilities**:
- Asynchronous size calculation
- Background thread per directory
- Safety limits (time, file count)
- Result caching

**Key Features**:
- 5-second timeout per directory
- 10,000 file limit per directory
- Partial result indicators
- Format: K/M/G/T

#### `file_icons.rs` (File Type Icons)

**Responsibilities**:
- Nerd Font icon mapping
- Emoji fallback icons
- Language-specific icons
- Directory-specific icons

**Categories**:
- Programming languages (Rust, Python, JS, etc.)
- Configuration files (Cargo.toml, package.json)
- Special directories (.git, node_modules)
- Media files (images, video, audio)
- Documents (PDF, Office)

### Supporting Modules

#### `theme.rs`

Color management and theme application.

#### `terminal.rs`

Terminal setup/cleanup helpers.

## Data Flow

### Startup Flow

```
main()
  → setup_terminal()
  → Config::load()
  → App::new()
      → Navigation::new()
      → FileViewer::new()
      → Search::new()
      → Bookmarks::new()
  → run_app()
      [event loop]
  → cleanup_terminal()
```

### Event Loop

```
loop {
  if need_terminal_clear { terminal.clear() }
  terminal.draw()

  if event::poll(100ms) {
    match event {
      Key → app.handle_key() → EventHandler
      Mouse → app.handle_mouse() → EventHandler
    }
  } else {
    app.poll_search()    // Check background search
    app.poll_sizes()     // Check size calculation
  }
}
```

### Search Flow

```
User presses '/'
  → Search::enter_mode()
  → User types query
  → User presses Enter
  → Search::perform_search()
      → Phase 1: Quick search (loaded nodes)
      → Phase 2: Spawn background thread
          → Walk entire tree
          → Send results via channel
          → Send progress updates
  → Main loop polls results
  → UI updates incrementally
```

### Navigation Flow

```
User presses Enter on directory
  → EventHandler::handle_key()
  → Navigation::go_to_directory()
      → Create new root node
      → Load children
      → Navigation::rebuild_flat_list()
          → Recursively collect visible nodes
          → Build Vec<TreeNodeRef>
  → Render
```

## Performance Considerations

### Zero-Copy Tree Operations

Uses `Rc<RefCell<>>` to avoid cloning nodes:

```rust
pub type TreeNodeRef = Rc<RefCell<TreeNode>>;
```

**Benefits**:
- O(1) tree operations instead of O(n²)
- Flat list stores references, not clones
- Expand/collapse doesn't copy subtrees

### Lazy Loading

- Directories loaded only when expanded
- Files loaded only when file viewer enabled
- Syntax highlighting loaded only on first use

### Async Operations

- Search runs in background thread
- Size calculation runs in background threads
- UI remains responsive during long operations

### Limits

- `max_file_lines = 10000` - Prevents loading huge files
- 5-second timeout for size calculation
- 10,000 file limit for size calculation

## Error Handling

### Error Strategy

1. **Graceful degradation**: Errors don't crash, they mark nodes
2. **User visibility**: Errors shown inline with ⚠ icon
3. **Detailed messages**: "Cannot read: Permission denied"
4. **Aggregation**: "5 entries inaccessible"

### Error Types

- **Filesystem errors**: Permission denied, not found, etc.
- **Binary file detection**: Shows message, suggests hex editor
- **Symlink errors**: Broken links show error
- **Metadata errors**: Can't get file size/permissions

## Testing

### Current State

- Unit tests for help toggle behavior
- No comprehensive test suite yet

### Future Plans

- Unit tests for navigation logic
- Integration tests for full workflows
- Property-based tests for tree invariants
- Mock filesystem for testing

## Dependencies

### Core

- `ratatui 0.28` - TUI framework
- `crossterm 0.28` - Terminal manipulation
- `anyhow 1.0` - Error handling

### CLI

- `clap 4.5` - Argument parsing

### Data

- `serde 1.0` - Serialization
- `serde_json 1.0` - JSON (bookmarks)
- `toml 0.8` - TOML (config)

### Features

- `arboard 3.4` - Clipboard
- `syntect 5.2` - Syntax highlighting
- `fuzzy-matcher 0.3` - Fuzzy search
- `crossbeam-channel 0.5` - Thread communication
- `once_cell 1.19` - Lazy statics
- `unicode-width 0.1` - Unicode display width
- `unicode-truncate 1.1` - Unicode-safe truncation
- `which 6.0` - Find executables
- `dirs 5.0` - User directories

## Future Architecture Plans

### Incremental Flat List Updates

Currently rebuilds entire flat list on expand/collapse. Optimization:

- Track only changed subtrees
- Update affected ranges in flat list
- Reduce O(n) to O(log n) for many operations

### Plugin System

Potential for plugins:

- Custom file viewers
- Additional search backends
- Cloud storage integration
- Git integration

### State Persistence

Save and restore state:

- Last visited directory
- Expand/collapse state
- Search history
- Navigation history

## Contributing to Architecture

When adding features:

1. **Identify the right module**: Don't add to `app.rs`
2. **Create new module if needed**: Keep modules focused
3. **Use existing patterns**: Follow Rc<RefCell<>> for trees
4. **Document public APIs**: Explain non-obvious behavior
5. **Consider performance**: Async for slow operations
6. **Handle errors gracefully**: Never crash, always inform

## Next Steps

- [Module Overview](./modules.md) - Detailed module documentation
- [Contributing](./contributing.md) - How to contribute
- [Building](./building.md) - Build from source
