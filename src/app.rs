use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    style::{Modifier, Style, Color},
    layout::{Layout, Constraint, Direction},
    text::{Line, Span},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use anyhow::Result;
use arboard::Clipboard;

use crate::tree_node::{TreeNode, TreeNodeRef};
use std::rc::Rc;
use std::cell::RefCell;

pub struct App {
    root: TreeNodeRef,
    flat_list: Vec<TreeNodeRef>, // References to visible nodes
    selected: usize,
    show_files: bool,
    file_content: Vec<String>, // Содержимое просматриваемого файла
    file_scroll: usize,        // Позиция скролла в файле
    split_position: u16,       // Позиция разделителя (ширина левой панели в %)
    dragging: bool,            // Флаг перетаскивания разделителя
    terminal_width: u16,       // Текущая ширина терминала
    tree_area_start: u16,      // Начало области дерева
    tree_area_end: u16,        // Конец области дерева
    current_file_path: PathBuf, // Путь к текущему просматриваемому файлу
    current_file_size: u64,    // Размер файла в байтах
    current_file_permissions: u32, // Права доступа к файлу
    last_click_time: Option<(Instant, usize)>, // Время и индекс последнего клика для двойного клика
    tree_area_top: u16,        // Верхняя граница области дерева (y)
    tree_area_height: u16,     // Высота области дерева
    viewer_area_start: u16,    // Начало области просмотра файла (x)
    viewer_area_top: u16,      // Верхняя граница области просмотра (y)
    viewer_area_height: u16,   // Высота области просмотра
    show_help: bool,           // Показывать ли help вместо содержимого файла
    search_mode: bool,         // Активен ли режим поиска
    search_query: String,      // Текущий поисковый запрос
    search_results: Vec<PathBuf>, // Найденные пути (полные пути к файлам/директориям)
    search_results_selected: usize, // Выбранный элемент в списке результатов
    show_search_results: bool, // Показывать ли панель с результатами поиска
    focus_on_search: bool,     // Фокус на панели поиска (true) или на дереве (false)
}

impl App {
    pub fn new(start_path: PathBuf) -> Result<Self> {
        let mut root = TreeNode::new(start_path, 0)?;
        root.load_children(false)?;
        root.is_expanded = true;
        let root = Rc::new(RefCell::new(root));

        let mut app = App {
            root,
            flat_list: Vec::new(),
            selected: 0,
            show_files: false,
            file_content: Vec::new(),
            file_scroll: 0,
            split_position: 50, // 50% ширины по умолчанию
            dragging: false,
            terminal_width: 0,
            tree_area_start: 0,
            tree_area_end: 0,
            current_file_path: PathBuf::new(),
            current_file_size: 0,
            current_file_permissions: 0,
            last_click_time: None,
            tree_area_top: 0,
            tree_area_height: 0,
            viewer_area_start: 0,
            viewer_area_top: 0,
            viewer_area_height: 0,
            show_help: false,
            search_mode: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_results_selected: 0,
            show_search_results: false,
            focus_on_search: false,
        };

        app.rebuild_flat_list();
        Ok(app)
    }

    fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        Self::collect_visible_nodes(&self.root, &mut self.flat_list);
    }

    fn collect_visible_nodes(node: &TreeNodeRef, result: &mut Vec<TreeNodeRef>) {
        result.push(Rc::clone(node));

        let node_borrowed = node.borrow();
        if node_borrowed.is_expanded {
            for child in &node_borrowed.children {
                Self::collect_visible_nodes(child, result);
            }
        }
    }

    fn get_selected_node(&self) -> Option<TreeNodeRef> {
        self.flat_list.get(self.selected).map(|n| Rc::clone(n))
    }

    pub fn get_help_content() -> Vec<String> {
        vec![
            "DTREE - Interactive Directory Tree Navigator".to_string(),
            "".to_string(),
            "DESCRIPTION".to_string(),
            "  dtree is a lightweight TUI application for interactive directory tree".to_string(),
            "  navigation. It provides a visual tree view with file preview capabilities.".to_string(),
            "".to_string(),
            "KEYBOARD NAVIGATION".to_string(),
            "  ↑ / k          Navigate up in the tree".to_string(),
            "  ↓ / j          Navigate down in the tree".to_string(),
            "  → / l          Expand directory (show subdirectories)".to_string(),
            "  ← / h          Collapse directory (hide subdirectories)".to_string(),
            "  u              Go to parent directory (change root)".to_string(),
            "  Backspace      Go to parent directory (change root)".to_string(),
            "  Enter          Select directory and exit (cd to selected)".to_string(),
            "  q / Esc        Quit without selection".to_string(),
            "  s              Toggle file viewer mode (show/hide files)".to_string(),
            "  c              Copy current path to clipboard (files and directories)".to_string(),
            "  i              Show/hide this help screen".to_string(),
            "".to_string(),
            "SEARCH".to_string(),
            "  /              Enter search mode".to_string(),
            "  Type query     Type your search query (case-insensitive)".to_string(),
            "  Enter          Execute search and show results panel".to_string(),
            "  Esc            Cancel search (in search mode) or close results panel".to_string(),
            "  q              Close search results panel (when panel is open)".to_string(),
            "".to_string(),
            "  In Search Results Panel:".to_string(),
            "  Tab            Switch focus between tree and search results".to_string(),
            "  ↑↓ / jk        Navigate through search results".to_string(),
            "  Enter          Select result and jump to it in the tree".to_string(),
            "".to_string(),
            "  Search features:".to_string(),
            "  • Search scope: from current root directory and below".to_string(),
            "  • Normal mode: searches ONLY directories (fast)".to_string(),
            "  • File viewer mode (s): searches both files and directories".to_string(),
            "  • Searches through the ENTIRE tree (including collapsed nodes)".to_string(),
            "  • Shows all results in a separate panel at the bottom".to_string(),
            "  • Select a result to automatically expand and jump to it in the tree".to_string(),
            "  • Case-insensitive substring matching".to_string(),
            "  • Cyan border indicates which panel has focus".to_string(),
            "".to_string(),
            "FILE VIEWER MODE (press 's' to toggle)".to_string(),
            "  When enabled:".to_string(),
            "    • Shows files in addition to directories".to_string(),
            "    • Displays file preview panel on the right".to_string(),
            "    • Shows file content (first 1000 lines)".to_string(),
            "    • Displays file information (size, lines, permissions)".to_string(),
            "".to_string(),
            "  File Preview Navigation:".to_string(),
            "    Ctrl+j       Scroll down in file preview".to_string(),
            "    Ctrl+k       Scroll up in file preview".to_string(),
            "    Scroll wheel Scroll file preview (when mouse over preview area)".to_string(),
            "".to_string(),
            "MOUSE SUPPORT".to_string(),
            "  Click          Select item in tree".to_string(),
            "  Double-click   Expand/collapse directory".to_string(),
            "  Scroll wheel   Navigate tree (when mouse over tree area)".to_string(),
            "                 Scroll file preview (when mouse over preview area)".to_string(),
            "  Drag           Resize split view (drag the vertical divider)".to_string(),
            "".to_string(),
            "FILE PREVIEW FEATURES".to_string(),
            "  • Text files: First 1000 lines displayed".to_string(),
            "  • Binary files: Detected and marked as [Binary file]".to_string(),
            "  • File information bar at the bottom:".to_string(),
            "    - Filename".to_string(),
            "    - Size (B, KB, MB, GB)".to_string(),
            "    - Number of lines".to_string(),
            "    - Permissions in Unix format (e.g., -rw-r--r-- (0644))".to_string(),
            "  • Adjustable split view (20-80% range)".to_string(),
            "".to_string(),
            "TIPS & TRICKS".to_string(),
            "  • Files are filtered out by default - press 's' to see them".to_string(),
            "  • Use mouse to quickly navigate - double-click to expand folders".to_string(),
            "  • Press 'c' to copy any path to clipboard - works for both files and directories".to_string(),
            "  • The selected path is printed to stdout on exit".to_string(),
            "  • UI is rendered to stderr, so path output is clean for scripting".to_string(),
            "  • Directories are shown in white, files in gray".to_string(),
            "  • Icons: ▶ (collapsed), ▼ (expanded)".to_string(),
            "".to_string(),
            "Press 'i' again to close this help screen".to_string(),
            "".to_string(),
            "================================================================================".to_string(),
            "SHELL INTEGRATION - BASH/ZSH WRAPPER".to_string(),
            "================================================================================".to_string(),
            "".to_string(),
            "To enable directory navigation with 'dt' command, add one of these functions".to_string(),
            "to your shell configuration file (~/.bashrc or ~/.zshrc):".to_string(),
            "".to_string(),
            "# For Bash (~/.bashrc) or Zsh (~/.zshrc):".to_string(),
            "dt() {".to_string(),
            "  # If flags are passed, just run dtree directly without cd".to_string(),
            "  case \"$1\" in".to_string(),
            "    -h|--help|-v|--version)".to_string(),
            "      command dtree \"$@\"".to_string(),
            "      return".to_string(),
            "      ;;".to_string(),
            "  esac".to_string(),
            "".to_string(),
            "  # If path argument provided, check if it exists".to_string(),
            "  if [ -n \"$1\" ] && [ ! -d \"$1\" ]; then".to_string(),
            "    echo \"Error: Directory '$1' does not exist\" >&2".to_string(),
            "    return 1".to_string(),
            "  fi".to_string(),
            "".to_string(),
            "  local result=$(command dtree \"$@\")".to_string(),
            "  # Only cd if result is a valid directory (ignores errors)".to_string(),
            "  if [ -n \"$result\" ] && [ -d \"$result\" ]; then".to_string(),
            "    cd \"$result\" || return".to_string(),
            "  fi".to_string(),
            "}".to_string(),
            "".to_string(),
            "SETUP STEPS:".to_string(),
            "  1. Copy the function above to your ~/.bashrc or ~/.zshrc".to_string(),
            "  2. Save the file".to_string(),
            "  3. Reload: source ~/.bashrc (or source ~/.zshrc)".to_string(),
            "  4. Use: dt [optional-start-path]".to_string(),
            "".to_string(),
            "HOW IT WORKS:".to_string(),
            "  • The wrapper captures dtree's output (selected directory path)".to_string(),
            "  • If you press Enter on a directory, shell changes to that directory".to_string(),
            "  • If you press q/Esc, shell stays in the current directory".to_string(),
            "".to_string(),
            "USAGE EXAMPLES:".to_string(),
            "  dt                    # Start from current directory".to_string(),
            "  dt ~/projects         # Start from ~/projects directory".to_string(),
            "  dt /var/log           # Start from /var/log directory".to_string(),
        ]
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<Option<PathBuf>> {
        // Режим поиска - отдельная обработка
        if self.search_mode {
            match key.code {
                KeyCode::Esc => {
                    self.exit_search_mode();
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Enter => {
                    // Выполняем поиск и выходим из режима ввода
                    self.perform_search();
                    self.search_mode = false;
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char(c) => {
                    // Добавляем символ к запросу
                    self.search_query.push(c);
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Backspace => {
                    // Удаляем последний символ
                    self.search_query.pop();
                    return Ok(Some(PathBuf::new()));
                }
                _ => return Ok(Some(PathBuf::new())),
            }
        }

        // Обработка Ctrl+j/k для скролла в области просмотра файла
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('j') => {
                    // Скролл вниз в области просмотра (работает для файлов и help)
                    if self.show_files {
                        let content_len = if self.show_help {
                            Self::get_help_content().len()
                        } else {
                            self.file_content.len()
                        };

                        if self.file_scroll < content_len.saturating_sub(1) {
                            self.file_scroll += 1;
                        }
                    }
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char('k') => {
                    // Скролл вверх в области просмотра (работает для файлов и help)
                    if self.show_files {
                        self.file_scroll = self.file_scroll.saturating_sub(1);
                    }
                    return Ok(Some(PathBuf::new()));
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') => {
                // Если панель результатов открыта, закрываем её
                if self.show_search_results {
                    self.close_search_results();
                    return Ok(Some(PathBuf::new()));
                } else {
                    return Ok(None);
                }
            }
            KeyCode::Esc => {
                // Закрываем панель результатов если открыта, иначе выходим
                if self.show_search_results {
                    self.close_search_results();
                    return Ok(Some(PathBuf::new()));
                } else {
                    return Ok(None);
                }
            }
            KeyCode::Char('/') => {
                // Входим в режим поиска
                self.search_mode = true;
                self.search_query.clear();
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Tab => {
                // Переключение фокуса между панелями (если результаты видны)
                if self.show_search_results {
                    self.focus_on_search = !self.focus_on_search;
                }
                return Ok(Some(PathBuf::new()));
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.focus_on_search {
                    // Navigation in search results panel
                    if self.search_results_selected < self.search_results.len().saturating_sub(1) {
                        self.search_results_selected += 1;
                    }
                } else {
                    // Navigation in tree
                    if self.selected < self.flat_list.len().saturating_sub(1) {
                        self.selected += 1;
                        // Update file content on navigation
                        if self.show_files {
                            let path = self.get_selected_node().map(|n| n.borrow().path.clone());
                            if let Some(p) = path {
                                let _ = self.load_file_content(&p);
                                self.show_help = false;
                            }
                        }
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.focus_on_search {
                    // Navigation in search results panel
                    self.search_results_selected = self.search_results_selected.saturating_sub(1);
                } else {
                    // Navigation in tree
                    self.selected = self.selected.saturating_sub(1);
                    // Update file content on navigation
                    if self.show_files {
                        let path = self.get_selected_node().map(|n| n.borrow().path.clone());
                        if let Some(p) = path {
                            let _ = self.load_file_content(&p);
                            self.show_help = false;
                        }
                    }
                }
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if key.code == KeyCode::Enter {
                    if self.focus_on_search && self.show_search_results {
                        // Enter в панели результатов - выбираем результат и переходим к нему
                        self.select_search_result();
                        return Ok(Some(PathBuf::new()));
                    } else {
                        // Enter in tree - select directory (only if it's a directory)
                        if let Some(node) = self.get_selected_node() {
                            let node_borrowed = node.borrow();
                            if node_borrowed.is_dir {
                                return Ok(Some(node_borrowed.path.clone()));
                            }
                        }
                    }
                } else {
                    // Right - expand (directories only)
                    if !self.focus_on_search {
                        if let Some(node) = self.get_selected_node() {
                            let node_borrowed = node.borrow();
                            if node_borrowed.is_dir {
                                let path = node_borrowed.path.clone();
                                drop(node_borrowed);
                                self.toggle_node(&path)?;
                            }
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(node) = self.get_selected_node() {
                    let node_borrowed = node.borrow();
                    if node_borrowed.is_dir {
                        let path = node_borrowed.path.clone();
                        drop(node_borrowed);
                        self.toggle_node(&path)?;
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                // Go to parent directory
                let parent_path = {
                    let root_borrowed = self.root.borrow();
                    root_borrowed.path.parent().map(|p| p.to_path_buf())
                };

                if let Some(parent_path) = parent_path {
                    let current_path = self.root.borrow().path.clone();

                    let mut new_root = TreeNode::new(parent_path, 0)?;
                    new_root.load_children(self.show_files)?;
                    new_root.is_expanded = true;

                    self.root = Rc::new(RefCell::new(new_root));
                    self.rebuild_flat_list();

                    // Find and select previous directory
                    for (i, node) in self.flat_list.iter().enumerate() {
                        if node.borrow().path == current_path {
                            self.selected = i;
                            break;
                        }
                    }
                }
            }
            KeyCode::Char('s') => {
                // Переключаем режим показа файлов
                self.show_files = !self.show_files;
                self.show_help = false; // Закрываем help при переключении режима
                self.reload_tree()?;

                // Load current file content when enabling the mode
                if self.show_files {
                    let path = self.get_selected_node().map(|n| n.borrow().path.clone());
                    if let Some(p) = path {
                        let _ = self.load_file_content(&p);
                    }
                }
            }
            KeyCode::Char('i') => {
                // Переключаем показ help
                self.show_help = !self.show_help;
                self.file_scroll = 0; // Сбрасываем скролл при открытии help

                // Включаем режим файлов, если он еще не включен
                if self.show_help && !self.show_files {
                    self.show_files = true;
                    self.reload_tree()?;
                }
            }
            KeyCode::Char('c') => {
                // Copy path to clipboard
                if let Some(node) = self.get_selected_node() {
                    if let Ok(mut clipboard) = Clipboard::new() {
                        let _ = clipboard.set_text(node.borrow().path.display().to_string());
                    }
                }
            }
            _ => {}
        }

        Ok(Some(PathBuf::new())) // Продолжаем работу
    }

    fn toggle_node(&mut self, path: &Path) -> Result<()> {
        Self::toggle_node_recursive(&self.root, path, self.show_files)?;
        self.rebuild_flat_list();
        Ok(())
    }

    fn toggle_node_recursive(node: &TreeNodeRef, target_path: &Path, show_files: bool) -> Result<bool> {
        let mut node_borrowed = node.borrow_mut();
        if node_borrowed.path == target_path {
            node_borrowed.toggle_expand(show_files)?;
            return Ok(true);
        }

        let children = node_borrowed.children.clone();
        drop(node_borrowed);

        for child in &children {
            if Self::toggle_node_recursive(child, target_path, show_files)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn reload_tree(&mut self) -> Result<()> {
        // Reload tree with new file display mode
        Self::reload_node_recursive(&self.root, self.show_files)?;
        self.rebuild_flat_list();
        Ok(())
    }

    fn reload_node_recursive(node: &TreeNodeRef, show_files: bool) -> Result<()> {
        let mut node_borrowed = node.borrow_mut();
        if node_borrowed.is_expanded && node_borrowed.is_dir {
            // Clear children and reload with new mode
            node_borrowed.children.clear();
            node_borrowed.load_children(show_files)?;

            // Recursively reload child nodes
            let children = node_borrowed.children.clone();
            drop(node_borrowed);

            for child in &children {
                Self::reload_node_recursive(child, show_files)?;
            }
        }
        Ok(())
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Проверяем клик по области дерева
                if mouse.column >= self.tree_area_start && mouse.column < self.tree_area_end
                    && mouse.row >= self.tree_area_top && mouse.row < self.tree_area_top + self.tree_area_height {

                    // Calculate element index (accounting for border +1)
                    let clicked_row = mouse.row.saturating_sub(self.tree_area_top + 1) as usize;

                    if clicked_row < self.flat_list.len() {
                        // Check for double click
                        let now = Instant::now();
                        let is_double_click = if let Some((last_time, last_idx)) = self.last_click_time {
                            clicked_row == last_idx && now.duration_since(last_time) < Duration::from_millis(500)
                        } else {
                            false
                        };

                        if is_double_click {
                            // Double click - expand/collapse
                            let node = &self.flat_list[clicked_row];
                            let node_borrowed = node.borrow();
                            if node_borrowed.is_dir {
                                let path = node_borrowed.path.clone();
                                drop(node_borrowed);
                                self.toggle_node(&path)?;
                            }
                            self.last_click_time = None;
                        } else {
                            // Single click - select element
                            self.selected = clicked_row;
                            self.last_click_time = Some((now, clicked_row));

                            // Update file content if viewer mode is enabled
                            if self.show_files {
                                let path = self.flat_list[clicked_row].borrow().path.clone();
                                let _ = self.load_file_content(&path);
                                self.show_help = false; // Close help when viewing file
                            }
                        }
                    }
                } else if self.show_files {
                    // Проверяем клик по разделителю
                    let divider_col = (self.terminal_width * self.split_position) / 100;
                    if mouse.column.abs_diff(divider_col) <= 2 {
                        self.dragging = true;
                    }
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.dragging && self.terminal_width > 0 {
                    // Конвертируем позицию мыши в проценты
                    let new_pos = (mouse.column as u16 * 100) / self.terminal_width;

                    // Ограничиваем диапазон 20-80%
                    self.split_position = new_pos.clamp(20, 80);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.dragging = false;
            }
            MouseEventKind::ScrollUp => {
                // Проверяем, над какой областью происходит скролл
                if self.show_files && mouse.column >= self.viewer_area_start
                    && mouse.row >= self.viewer_area_top
                    && mouse.row < self.viewer_area_top + self.viewer_area_height {
                    // Скролл в области просмотра файла
                    self.file_scroll = self.file_scroll.saturating_sub(1);
                } else {
                    // Scroll up in tree
                    self.selected = self.selected.saturating_sub(1);

                    // Update file content
                    if self.show_files {
                        let path = self.flat_list.get(self.selected).map(|n| n.borrow().path.clone());
                        if let Some(p) = path {
                            let _ = self.load_file_content(&p);
                            self.show_help = false; // Close help when viewing file
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                // Check which area is being scrolled
                if self.show_files && mouse.column >= self.viewer_area_start
                    && mouse.row >= self.viewer_area_top
                    && mouse.row < self.viewer_area_top + self.viewer_area_height {
                    // Scroll in file viewer area or help
                    let content_len = if self.show_help {
                        Self::get_help_content().len()
                    } else {
                        self.file_content.len()
                    };

                    let content_height = self.viewer_area_height.saturating_sub(2) as usize;
                    let lines_to_show = content_height.saturating_sub(2);
                    let max_scroll = content_len.saturating_sub(lines_to_show);

                    if self.file_scroll < max_scroll {
                        self.file_scroll += 1;
                    }
                } else {
                    // Scroll down in tree
                    if self.selected < self.flat_list.len().saturating_sub(1) {
                        self.selected += 1;

                        // Update file content
                        if self.show_files {
                            let path = self.flat_list.get(self.selected).map(|n| n.borrow().path.clone());
                            if let Some(p) = path {
                                let _ = self.load_file_content(&p);
                                self.show_help = false; // Close help when viewing file
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn load_file_content(&mut self, path: &Path) -> Result<()> {
        const MAX_LINES: usize = 1000;

        self.file_content.clear();
        self.file_scroll = 0;
        self.current_file_path = path.to_path_buf();
        self.current_file_size = 0;
        self.current_file_permissions = 0;

        // Проверяем, что это файл
        if !path.is_file() {
            self.file_content.push("[Directory]".to_string());
            return Ok(());
        }

        // Получаем метаданные файла
        if let Ok(metadata) = std::fs::metadata(path) {
            self.current_file_size = metadata.len();
            self.current_file_permissions = metadata.permissions().mode();
        }

        // Пытаемся открыть файл
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => {
                self.file_content.push(format!("[Error: {}]", e));
                return Ok(());
            }
        };

        let reader = BufReader::new(file);
        let mut line_count = 0;

        for line in reader.lines() {
            if line_count >= MAX_LINES {
                self.file_content.push(format!("\n[... truncated at {} lines ...]", MAX_LINES));
                break;
            }

            match line {
                Ok(content) => {
                    self.file_content.push(content);
                    line_count += 1;
                }
                Err(_) => {
                    // Возможно бинарный файл
                    self.file_content.clear();
                    self.file_content.push("[Binary file]".to_string());
                    break;
                }
            }
        }

        if self.file_content.is_empty() {
            self.file_content.push("[Empty file]".to_string());
        }

        Ok(())
    }

    pub fn render(&mut self, frame: &mut Frame) {
        // Сохраняем ширину терминала для обработки мыши
        self.terminal_width = frame.area().width;

        let main_area = frame.area();

        // Если режим поиска активен, резервируем место для поисковой строки
        let (content_area, search_bar_area) = if self.search_mode {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(3),
                ])
                .split(main_area);
            (chunks[0], Some(chunks[1]))
        } else {
            (main_area, None)
        };

        // Если показываем результаты поиска, делим экран вертикально
        let (tree_area, search_results_area) = if self.show_search_results {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(70),
                    Constraint::Percentage(30),
                ])
                .split(content_area);
            (chunks[0], Some(chunks[1]))
        } else {
            (content_area, None)
        };

        // Если режим просмотра файлов включен, делим экран горизонтально
        if self.show_files {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(self.split_position),
                    Constraint::Percentage(100 - self.split_position),
                ])
                .split(tree_area);

            // Сохраняем границы области дерева
            self.tree_area_start = chunks[0].x;
            self.tree_area_end = chunks[0].x + chunks[0].width;

            // Левая панель - дерево каталогов
            self.render_tree(frame, chunks[0]);

            // Правая панель - просмотр файла
            self.render_file_viewer(frame, chunks[1]);
        } else {
            // Полноэкранный режим - только дерево
            self.tree_area_start = tree_area.x;
            self.tree_area_end = tree_area.x + tree_area.width;
            self.render_tree(frame, tree_area);
        }

        // Рендерим панель результатов поиска если она активна
        if let Some(area) = search_results_area {
            self.render_search_results(frame, area);
        }

        // Рендерим поисковую строку если режим ввода активен
        if let Some(area) = search_bar_area {
            self.render_search_bar(frame, area);
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Save tree area coordinates for mouse handling
        self.tree_area_top = area.y;
        self.tree_area_height = area.height;

        let items: Vec<ListItem> = self.flat_list.iter().map(|node| {
            let node_borrowed = node.borrow();
            let indent = "  ".repeat(node_borrowed.depth);
            let icon = if node_borrowed.is_dir {
                if node_borrowed.is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let text = format!("{}{}{}", indent, icon, node_borrowed.name);

            // Directories - white, files - gray
            let style = if node_borrowed.is_dir {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(text).style(style)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(self.selected));

        let title = " Directory Tree (↑↓/jk: navigate | /: search | c: copy | Enter: select | q: quit | i: help) ";

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_search_bar(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let search_text = format!("Search: {}", self.search_query);

        let paragraph = Paragraph::new(search_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(" Enter to search, Esc to cancel "))
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(paragraph, area);
    }

    fn render_search_results(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Format results for display
        let root_path = self.root.borrow().path.clone();
        let root_parent = root_path.parent().unwrap_or(&root_path);

        let items: Vec<ListItem> = self.search_results.iter().map(|path| {
            let display_path = path.strip_prefix(root_parent)
                .unwrap_or(path)
                .display()
                .to_string();

            let style = Style::default().fg(Color::White);
            ListItem::new(display_path).style(style)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(self.search_results_selected));

        let title = format!(" Search Results: {} found (Enter to select, q to close) ",
            self.search_results.len());

        let border_style = if self.focus_on_search {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style))
            .highlight_style(Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_file_viewer(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Сохраняем координаты области просмотра для обработки мыши
        self.viewer_area_start = area.x;
        self.viewer_area_top = area.y;
        self.viewer_area_height = area.height;

        let content_height = area.height.saturating_sub(2) as usize; // -2 для рамки

        // Если показываем help, используем его содержимое
        let content_to_display = if self.show_help {
            Self::get_help_content()
        } else {
            self.file_content.clone()
        };

        // Формируем отображаемые строки с учетом скролла
        // Оставляем место для разделителя (1 строка) + информации о файле (1 строка)
        let lines_to_show = content_height.saturating_sub(2);

        let mut visible_lines: Vec<Line> = content_to_display
            .iter()
            .skip(self.file_scroll)
            .take(lines_to_show)
            .map(|line| Line::from(line.as_str()))
            .collect();

        // Добавляем разделитель и информацию о файле в конец (только если не help)
        if !self.show_help && !self.current_file_path.as_os_str().is_empty() {
            let file_info = self.format_file_info();
            let separator = "─".repeat(area.width.saturating_sub(2) as usize);

            visible_lines.push(Line::from(
                Span::styled(separator, Style::default().fg(Color::DarkGray))
            ));
            visible_lines.push(Line::from(
                Span::styled(file_info, Style::default().fg(Color::DarkGray))
            ));
        }

        let scroll_info = if content_to_display.len() > lines_to_show {
            format!(" [↕ {}/{}]", self.file_scroll + 1, content_to_display.len())
        } else {
            String::new()
        };

        let title = if self.show_help {
            format!(" Help{} ", scroll_info)
        } else {
            format!(" File Viewer{} ", scroll_info)
        };

        let paragraph = Paragraph::new(visible_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title));

        frame.render_widget(paragraph, area);
    }

    fn format_file_info(&self) -> String {
        if self.current_file_path.as_os_str().is_empty() {
            return String::new();
        }

        let file_name = self.current_file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown");

        // Форматируем размер файла
        let size_str = format_file_size(self.current_file_size);

        // Получаем количество строк
        let lines_count = self.file_content.len();
        let lines_info = if lines_count >= 1000 {
            format!("{}+ lines", lines_count)
        } else {
            format!("{} lines", lines_count)
        };

        // Форматируем права доступа
        let permissions_str = format_permissions(self.current_file_permissions);

        format!(" {} | {} | {} | {}", file_name, size_str, lines_info, permissions_str)
    }

    fn perform_search(&mut self) {
        self.search_results.clear();
        self.search_results_selected = 0;

        if self.search_query.is_empty() {
            self.show_search_results = false;
            return;
        }

        let query_lower = self.search_query.to_lowercase();

        // Собираем все пути которые соответствуют поиску, обходя всё дерево
        Self::search_recursive(&self.root, &query_lower, &mut self.search_results, self.show_files);

        // Показываем панель результатов если что-то найдено
        self.show_search_results = !self.search_results.is_empty();
        self.focus_on_search = self.show_search_results;
    }

    fn select_search_result(&mut self) {
        if self.search_results.is_empty() || self.search_results_selected >= self.search_results.len() {
            return;
        }

        let selected_path = self.search_results[self.search_results_selected].clone();

        // Разворачиваем путь к выбранному элементу
        let _ = Self::expand_path_to_node(&mut self.root, &selected_path, self.show_files);

        // Перестраиваем плоский список
        self.rebuild_flat_list();

        // Find and select element in tree
        for (idx, node) in self.flat_list.iter().enumerate() {
            if node.borrow().path == selected_path {
                self.selected = idx;
                break;
            }
        }

        // Переключаем фокус на дерево
        self.focus_on_search = false;

        // Обновляем содержимое файла если режим просмотра включен
        if self.show_files {
            let _ = self.load_file_content(&selected_path);
            self.show_help = false;
        }
    }

    // Recursive search through entire tree
    // show_files = true: search both files and directories
    // show_files = false: search only directories
    fn search_recursive(node: &TreeNodeRef, query: &str, results: &mut Vec<PathBuf>, show_files: bool) {
        let mut node_borrowed = node.borrow_mut();
        let name_lower = node_borrowed.name.to_lowercase();

        // Check current node
        // If show_files = false, search only directories
        // If show_files = true, search both files and directories
        if show_files || node_borrowed.is_dir {
            if name_lower.contains(query) {
                results.push(node_borrowed.path.clone());
            }
        }

        // If this is a directory, load children and search recursively
        if node_borrowed.is_dir {
            if node_borrowed.children.is_empty() {
                let _ = node_borrowed.load_children(show_files);
            }

            // Recursively search in children
            let children = node_borrowed.children.clone();
            drop(node_borrowed);

            for child in &children {
                Self::search_recursive(child, query, results, show_files);
            }
        }
    }

    // Expands all parent directories up to the specified path
    fn expand_path_to_node(node: &TreeNodeRef, target_path: &PathBuf, show_files: bool) -> Result<bool> {
        let mut node_borrowed = node.borrow_mut();

        // If this is the target node, do nothing
        if &node_borrowed.path == target_path {
            return Ok(true);
        }

        // Check if target_path is a descendant of current node
        if !target_path.starts_with(&node_borrowed.path) {
            return Ok(false);
        }

        // Load children if needed
        if node_borrowed.children.is_empty() && node_borrowed.is_dir {
            node_borrowed.load_children(show_files)?;
        }

        // Expand current node
        node_borrowed.is_expanded = true;

        // Recursively search in children
        let children = node_borrowed.children.clone();
        drop(node_borrowed);

        for child in &children {
            if Self::expand_path_to_node(child, target_path, show_files)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn exit_search_mode(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
    }

    fn close_search_results(&mut self) {
        self.show_search_results = false;
        self.search_results.clear();
        self.search_results_selected = 0;
        self.focus_on_search = false;
    }
}

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn format_permissions(mode: u32) -> String {
    // Извлекаем биты прав доступа (последние 9 бит)
    let perms = mode & 0o777;

    // Определяем тип файла
    let file_type = if mode & 0o170000 == 0o040000 {
        'd' // directory
    } else if mode & 0o170000 == 0o120000 {
        'l' // symbolic link
    } else {
        '-' // regular file
    };

    // Форматируем права для владельца, группы и остальных
    let user = format_permission_triplet((perms >> 6) & 0o7);
    let group = format_permission_triplet((perms >> 3) & 0o7);
    let other = format_permission_triplet(perms & 0o7);

    format!("{}{}{}{} ({:04o})", file_type, user, group, other, perms)
}

fn format_permission_triplet(triplet: u32) -> String {
    let r = if triplet & 0o4 != 0 { 'r' } else { '-' };
    let w = if triplet & 0o2 != 0 { 'w' } else { '-' };
    let x = if triplet & 0o1 != 0 { 'x' } else { '-' };
    format!("{}{}{}", r, w, x)
}
