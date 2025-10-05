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

use crate::tree_node::TreeNode;

pub struct App {
    root: TreeNode,
    flat_list: Vec<usize>, // Индексы видимых узлов
    selected: usize,
    all_nodes: Vec<TreeNode>,
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
}

impl App {
    pub fn new(start_path: PathBuf) -> Result<Self> {
        let mut root = TreeNode::new(start_path, 0)?;
        root.load_children(false)?;
        root.is_expanded = true;

        let mut app = App {
            root,
            flat_list: Vec::new(),
            selected: 0,
            all_nodes: Vec::new(),
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
        };

        app.rebuild_flat_list();
        Ok(app)
    }

    fn rebuild_flat_list(&mut self) {
        self.all_nodes.clear();
        self.flat_list.clear();
        let root = self.root.clone();
        Self::collect_visible_nodes(&root, &mut self.all_nodes);
        self.flat_list = (0..self.all_nodes.len()).collect();
    }

    fn collect_visible_nodes(node: &TreeNode, result: &mut Vec<TreeNode>) {
        result.push(node.clone());

        if node.is_expanded {
            for child in &node.children {
                Self::collect_visible_nodes(child, result);
            }
        }
    }

    fn get_selected_node(&self) -> Option<&TreeNode> {
        self.all_nodes.get(self.selected)
    }

    fn get_help_content() -> Vec<String> {
        vec![
            "DTREE - Interactive Directory Tree Navigator".to_string(),
            "".to_string(),
            "DESCRIPTION".to_string(),
            "  dtree is a lightweight TUI application for interactive directory tree".to_string(),
            "  navigation. It provides a visual tree view with file preview capabilities.".to_string(),
            "".to_string(),
            "KEYBOARD NAVIGATION".to_string(),
            "  ↑/k, ↓/j       Navigate up/down through the tree".to_string(),
            "  →/l            Expand directory (show subdirectories)".to_string(),
            "  ←/h            Collapse directory (hide subdirectories)".to_string(),
            "  u, Backspace   Go to parent directory (change root)".to_string(),
            "  Enter          Select directory and exit (cd to selected)".to_string(),
            "  q, Esc         Quit without selection".to_string(),
            "".to_string(),
            "FILE VIEWER".to_string(),
            "  s              Toggle file viewer mode".to_string(),
            "                 When enabled: shows files and preview panel".to_string(),
            "                 When disabled: shows only directories".to_string(),
            "  Ctrl+j         Scroll down in file preview".to_string(),
            "  Ctrl+k         Scroll up in file preview".to_string(),
            "".to_string(),
            "MOUSE SUPPORT".to_string(),
            "  Click          Select item in tree / Drag splitter".to_string(),
            "  Double-click   Expand/collapse directory".to_string(),
            "  Scroll wheel   Navigate tree or scroll file preview".to_string(),
            "                 (depends on cursor position)".to_string(),
            "".to_string(),
            "INFORMATION".to_string(),
            "  i              Show this help screen".to_string(),
            "".to_string(),
            "FILE PREVIEW FEATURES".to_string(),
            "  • First 1000 lines of text files".to_string(),
            "  • File information: name, size, lines, permissions".to_string(),
            "  • Binary file detection".to_string(),
            "  • Adjustable split view (drag the divider)".to_string(),
            "".to_string(),
            "USAGE EXAMPLE".to_string(),
            "  Add this function to your ~/.bashrc:".to_string(),
            "".to_string(),
            "    dt() {".to_string(),
            "      local result=$(command dtree \"$@\")".to_string(),
            "      if [ -n \"$result\" ]; then".to_string(),
            "        cd \"$result\" || return".to_string(),
            "      fi".to_string(),
            "    }".to_string(),
            "".to_string(),
            "  Then use: dt".to_string(),
            "".to_string(),
            "TIPS".to_string(),
            "  • Files are filtered out by default - press 's' to see them".to_string(),
            "  • Use mouse to quickly navigate - double-click to expand".to_string(),
            "  • The selected path is printed to stdout on exit".to_string(),
            "  • UI is rendered to stderr, so path output is clean".to_string(),
            "".to_string(),
            "Press 'i' again to close this help screen".to_string(),
        ]
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<Option<PathBuf>> {
        // Обработка Ctrl+j/k для скролла в области просмотра файла
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('j') => {
                    // Скролл вниз в области просмотра
                    if self.show_files && !self.file_content.is_empty() {
                        if self.file_scroll < self.file_content.len().saturating_sub(1) {
                            self.file_scroll += 1;
                        }
                    }
                    return Ok(Some(PathBuf::new()));
                }
                KeyCode::Char('k') => {
                    // Скролл вверх в области просмотра
                    if self.show_files {
                        self.file_scroll = self.file_scroll.saturating_sub(1);
                    }
                    return Ok(Some(PathBuf::new()));
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Ok(None);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.all_nodes.len().saturating_sub(1) {
                    self.selected += 1;
                    // Обновляем содержимое файла при навигации
                    if self.show_files {
                        let path = self.get_selected_node().map(|n| n.path.clone());
                        if let Some(p) = path {
                            let _ = self.load_file_content(&p);
                        }
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                // Обновляем содержимое файла при навигации
                if self.show_files {
                    let path = self.get_selected_node().map(|n| n.path.clone());
                    if let Some(p) = path {
                        let _ = self.load_file_content(&p);
                    }
                }
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if key.code == KeyCode::Enter {
                    // Enter - выбираем директорию (только если это директория)
                    if let Some(node) = self.get_selected_node() {
                        if node.is_dir {
                            return Ok(Some(node.path.clone()));
                        }
                    }
                } else {
                    // Right - разворачиваем (только директории)
                    if let Some(node) = self.get_selected_node() {
                        if node.is_dir {
                            let path = node.path.clone();
                            self.toggle_node(&path)?;
                        }
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(node) = self.get_selected_node() {
                    if node.is_dir {
                        let path = node.path.clone();
                        self.toggle_node(&path)?;
                    }
                }
            }
            KeyCode::Char('u') | KeyCode::Backspace => {
                // Переходим к родительской директории
                if let Some(parent) = self.root.path.parent() {
                    let parent_path = parent.to_path_buf();
                    let current_path = self.root.path.clone();

                    let mut new_root = TreeNode::new(parent_path, 0)?;
                    new_root.load_children(self.show_files)?;
                    new_root.is_expanded = true;

                    self.root = new_root;
                    self.rebuild_flat_list();

                    // Находим и выбираем предыдущую директорию
                    for (i, node) in self.all_nodes.iter().enumerate() {
                        if node.path == current_path {
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

                // Загружаем содержимое текущего файла при включении режима
                if self.show_files {
                    let path = self.get_selected_node().map(|n| n.path.clone());
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
            _ => {}
        }

        Ok(Some(PathBuf::new())) // Продолжаем работу
    }

    fn toggle_node(&mut self, path: &Path) -> Result<()> {
        Self::toggle_node_recursive(&mut self.root, path, self.show_files)?;
        self.rebuild_flat_list();
        Ok(())
    }

    fn toggle_node_recursive(node: &mut TreeNode, target_path: &Path, show_files: bool) -> Result<bool> {
        if node.path == target_path {
            node.toggle_expand(show_files)?;
            return Ok(true);
        }

        for child in &mut node.children {
            if Self::toggle_node_recursive(child, target_path, show_files)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn reload_tree(&mut self) -> Result<()> {
        // Перезагружаем дерево с новым режимом показа файлов
        Self::reload_node_recursive(&mut self.root, self.show_files)?;
        self.rebuild_flat_list();
        Ok(())
    }

    fn reload_node_recursive(node: &mut TreeNode, show_files: bool) -> Result<()> {
        if node.is_expanded && node.is_dir {
            // Очищаем детей и перезагружаем с новым режимом
            node.children.clear();
            node.load_children(show_files)?;

            // Рекурсивно перезагружаем дочерние узлы
            for child in &mut node.children {
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

                    // Вычисляем индекс элемента (учитываем рамку +1)
                    let clicked_row = mouse.row.saturating_sub(self.tree_area_top + 1) as usize;

                    if clicked_row < self.all_nodes.len() {
                        // Проверка на двойной клик
                        let now = Instant::now();
                        let is_double_click = if let Some((last_time, last_idx)) = self.last_click_time {
                            clicked_row == last_idx && now.duration_since(last_time) < Duration::from_millis(500)
                        } else {
                            false
                        };

                        if is_double_click {
                            // Двойной клик - разворачиваем/сворачиваем
                            let node = &self.all_nodes[clicked_row];
                            if node.is_dir {
                                let path = node.path.clone();
                                self.toggle_node(&path)?;
                            }
                            self.last_click_time = None;
                        } else {
                            // Одиночный клик - выбираем элемент
                            self.selected = clicked_row;
                            self.last_click_time = Some((now, clicked_row));

                            // Обновляем содержимое файла если режим просмотра включен
                            if self.show_files {
                                let path = self.all_nodes[clicked_row].path.clone();
                                let _ = self.load_file_content(&path);
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
                    // Скролл вверх в дереве
                    self.selected = self.selected.saturating_sub(1);

                    // Обновляем содержимое файла
                    if self.show_files {
                        let path = self.all_nodes.get(self.selected).map(|n| n.path.clone());
                        if let Some(p) = path {
                            let _ = self.load_file_content(&p);
                        }
                    }
                }
            }
            MouseEventKind::ScrollDown => {
                // Проверяем, над какой областью происходит скролл
                if self.show_files && mouse.column >= self.viewer_area_start
                    && mouse.row >= self.viewer_area_top
                    && mouse.row < self.viewer_area_top + self.viewer_area_height {
                    // Скролл в области просмотра файла
                    let content_height = self.viewer_area_height.saturating_sub(2) as usize;
                    let lines_to_show = content_height.saturating_sub(2);
                    let max_scroll = self.file_content.len().saturating_sub(lines_to_show);

                    if self.file_scroll < max_scroll {
                        self.file_scroll += 1;
                    }
                } else {
                    // Скролл вниз в дереве
                    if self.selected < self.all_nodes.len().saturating_sub(1) {
                        self.selected += 1;

                        // Обновляем содержимое файла
                        if self.show_files {
                            let path = self.all_nodes.get(self.selected).map(|n| n.path.clone());
                            if let Some(p) = path {
                                let _ = self.load_file_content(&p);
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

        // Если режим просмотра файлов включен, делим экран
        if self.show_files {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(self.split_position),
                    Constraint::Percentage(100 - self.split_position),
                ])
                .split(frame.area());

            // Сохраняем границы области дерева
            self.tree_area_start = chunks[0].x;
            self.tree_area_end = chunks[0].x + chunks[0].width;

            // Левая панель - дерево каталогов
            self.render_tree(frame, chunks[0]);

            // Правая панель - просмотр файла
            self.render_file_viewer(frame, chunks[1]);
        } else {
            // Полноэкранный режим - только дерево
            let area = frame.area();
            self.tree_area_start = area.x;
            self.tree_area_end = area.x + area.width;
            self.render_tree(frame, area);
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        // Сохраняем координаты области дерева для обработки мыши
        self.tree_area_top = area.y;
        self.tree_area_height = area.height;

        let items: Vec<ListItem> = self.all_nodes.iter().map(|node| {
            let indent = "  ".repeat(node.depth);
            let icon = if node.is_dir {
                if node.is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };

            let text = format!("{}{}{}", indent, icon, node.name);

            // Директории - белым, файлы - серым
            let style = if node.is_dir {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            ListItem::new(text).style(style)
        }).collect();

        let mut state = ListState::default();
        state.select(Some(self.selected));

        let title = " Directory Tree (↑↓/jk: navigate | Enter: select | q: quit | i: help) ";

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
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
