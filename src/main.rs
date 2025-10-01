use std::path::{Path, PathBuf};
use std::fs;
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Color, Modifier, Style},
    Terminal, Frame,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use anyhow::Result;

#[derive(Clone)]
struct TreeNode {
    path: PathBuf,
    name: String,
    is_dir: bool,
    is_expanded: bool,
    depth: usize,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(path: PathBuf, depth: usize) -> Result<Self> {
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        
        let is_dir = path.is_dir();
        
        Ok(TreeNode {
            path,
            name,
            is_dir,
            is_expanded: false,
            depth,
            children: Vec::new(),
        })
    }
    
    fn load_children(&mut self) -> Result<()> {
        if !self.is_dir || !self.children.is_empty() {
            return Ok(());
        }
        
        let entries = fs::read_dir(&self.path)?;
        
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            // Показываем только директории
            if path.is_dir() {
                if let Ok(node) = TreeNode::new(path, self.depth + 1) {
                    self.children.push(node);
                }
            }
        }
        
        // Сортируем по имени
        self.children.sort_by(|a, b| a.name.cmp(&b.name));
        
        Ok(())
    }
    
    fn toggle_expand(&mut self) -> Result<()> {
        if !self.is_dir {
            return Ok(());
        }
        
        if self.is_expanded {
            self.is_expanded = false;
        } else {
            self.load_children()?;
            self.is_expanded = true;
        }
        
        Ok(())
    }
}

struct App {
    root: TreeNode,
    flat_list: Vec<usize>, // Индексы видимых узлов
    selected: usize,
    all_nodes: Vec<TreeNode>,
}

impl App {
    fn new(start_path: PathBuf) -> Result<Self> {
        let mut root = TreeNode::new(start_path, 0)?;
        root.load_children()?;
        root.is_expanded = true;
        
        let mut app = App {
            root,
            flat_list: Vec::new(),
            selected: 0,
            all_nodes: Vec::new(),
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

    fn handle_key(&mut self, key: KeyEvent) -> Result<Option<PathBuf>> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Ok(None);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.selected < self.all_nodes.len().saturating_sub(1) {
                    self.selected += 1;
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right | KeyCode::Enter => {
                if key.code == KeyCode::Enter {
                    // Enter - выбираем директорию
                    if let Some(node) = self.get_selected_node() {
                        return Ok(Some(node.path.clone()));
                    }
                } else {
                    // Right - разворачиваем
                    if let Some(node) = self.get_selected_node() {
                        let path = node.path.clone();
                        self.toggle_node(&path)?;
                    }
                }
            }
            KeyCode::Char('h') | KeyCode::Left => {
                if let Some(node) = self.get_selected_node() {
                    let path = node.path.clone();
                    self.toggle_node(&path)?;
                }
            }
            _ => {}
        }
        
        Ok(Some(PathBuf::new())) // Продолжаем работу
    }
    
    fn toggle_node(&mut self, path: &Path) -> Result<()> {
        Self::toggle_node_recursive(&mut self.root, path)?;
        self.rebuild_flat_list();
        Ok(())
    }

    fn toggle_node_recursive(node: &mut TreeNode, target_path: &Path) -> Result<bool> {
        if node.path == target_path {
            node.toggle_expand()?;
            return Ok(true);
        }

        for child in &mut node.children {
            if Self::toggle_node_recursive(child, target_path)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
    
    fn render(&mut self, frame: &mut Frame) {
        let items: Vec<ListItem> = self.all_nodes.iter().map(|node| {
            let indent = "  ".repeat(node.depth);
            let icon = if node.is_dir {
                if node.is_expanded { "▼ " } else { "▶ " }
            } else {
                "  "
            };
            
            let text = format!("{}{}{}", indent, icon, node.name);
            ListItem::new(text)
        }).collect();
        
        let mut state = ListState::default();
        state.select(Some(self.selected));
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Directory Tree (↑↓/jk: navigate, →l: expand, ←h: collapse, Enter: select, q: quit)"))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
            .highlight_symbol(">> ");
        
        frame.render_stateful_widget(list, frame.area(), &mut state);
    }
}

fn main() -> Result<()> {
    let start_path = std::env::current_dir()?;

    // Setup terminal
    enable_raw_mode()?;
    std::io::stderr().execute(EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let mut terminal = Terminal::new(backend)?;
    
    let mut app = App::new(start_path)?;
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal BEFORE printing the path
    disable_raw_mode()?;
    std::io::stderr().execute(LeaveAlternateScreen)?;

    match result? {
        Some(path) => {
            // Выводим путь в stdout для bash-обёртки
            println!("{}", path.display());
            Ok(())
        }
        None => Ok(()),
    }
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stderr>>, app: &mut App) -> Result<Option<PathBuf>> {
    loop {
        terminal.draw(|f| app.render(f))?;
        
        if let Event::Key(key) = event::read()? {
            match app.handle_key(key)? {
                Some(path) if !path.as_os_str().is_empty() => {
                    return Ok(Some(path));
                }
                None => {
                    return Ok(None);
                }
                _ => {}
            }
        }
    }
}
