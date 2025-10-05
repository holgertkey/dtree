use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{BufRead, BufReader};
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
                self.reload_tree()?;

                // Загружаем содержимое текущего файла при включении режима
                if self.show_files {
                    let path = self.get_selected_node().map(|n| n.path.clone());
                    if let Some(p) = path {
                        let _ = self.load_file_content(&p);
                    }
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

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if !self.show_files {
            return; // Мышь работает только в режиме просмотра файлов
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Вычисляем реальную позицию разделителя в пикселях
                let divider_col = (self.terminal_width * self.split_position) / 100;

                // Проверяем, что клик рядом с разделителем (±2 символа)
                if mouse.column.abs_diff(divider_col) <= 2 {
                    self.dragging = true;
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
            _ => {}
        }
    }

    fn load_file_content(&mut self, path: &Path) -> Result<()> {
        const MAX_LINES: usize = 1000;

        self.file_content.clear();
        self.file_scroll = 0;
        self.current_file_path = path.to_path_buf();
        self.current_file_size = 0;

        // Проверяем, что это файл
        if !path.is_file() {
            self.file_content.push("[Directory]".to_string());
            return Ok(());
        }

        // Получаем метаданные файла
        if let Ok(metadata) = std::fs::metadata(path) {
            self.current_file_size = metadata.len();
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
            self.render_tree(frame, frame.area());
        }
    }

    fn render_tree(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
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

        let title = if self.show_files {
            "Tree (↑↓/jk: nav, →l: expand, ←h: collapse, u: parent, s: hide files, Ctrl+j/k: scroll, Enter: select, q: quit)"
        } else {
            "Tree (↑↓/jk: navigate, →l: expand, ←h: collapse, u/Backspace: parent, s: show files, Enter: select, q: quit)"
        };

        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(title))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, area, &mut state);
    }

    fn render_file_viewer(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let content_height = area.height.saturating_sub(2) as usize; // -2 для рамки

        // Формируем отображаемые строки с учетом скролла
        // Оставляем место для разделителя (1 строка) + информации о файле (1 строка)
        let lines_to_show = content_height.saturating_sub(2);

        let mut visible_lines: Vec<Line> = self.file_content
            .iter()
            .skip(self.file_scroll)
            .take(lines_to_show)
            .map(|line| Line::from(line.as_str()))
            .collect();

        // Добавляем разделитель и информацию о файле в конец
        if !self.current_file_path.as_os_str().is_empty() {
            let file_info = self.format_file_info();
            let separator = "─".repeat(area.width.saturating_sub(2) as usize);

            visible_lines.push(Line::from(
                Span::styled(separator, Style::default().fg(Color::DarkGray))
            ));
            visible_lines.push(Line::from(
                Span::styled(file_info, Style::default().fg(Color::DarkGray))
            ));
        }

        let scroll_info = if self.file_content.len() > lines_to_show {
            format!(" [↕ {}/{}]", self.file_scroll + 1, self.file_content.len())
        } else {
            String::new()
        };

        let title = format!("File Viewer{}", scroll_info);

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

        format!(" {} | {} | {}", file_name, size_str, lines_info)
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
