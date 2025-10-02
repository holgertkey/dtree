use std::path::{Path, PathBuf};
use ratatui::{
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Modifier, Style},
    Frame,
};
use crossterm::event::{KeyCode, KeyEvent};
use anyhow::Result;

use crate::tree_node::TreeNode;

pub struct App {
    root: TreeNode,
    flat_list: Vec<usize>, // Индексы видимых узлов
    selected: usize,
    all_nodes: Vec<TreeNode>,
}

impl App {
    pub fn new(start_path: PathBuf) -> Result<Self> {
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

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<Option<PathBuf>> {
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
            KeyCode::Char('u') | KeyCode::Backspace => {
                // Переходим к родительской директории
                if let Some(parent) = self.root.path.parent() {
                    let parent_path = parent.to_path_buf();
                    let current_path = self.root.path.clone();

                    let mut new_root = TreeNode::new(parent_path, 0)?;
                    new_root.load_children()?;
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

    pub fn render(&mut self, frame: &mut Frame) {
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
                .title("Directory Tree (↑↓/jk: navigate, →l: expand, ←h: collapse, u/Backspace: parent, Enter: select, q: quit)"))
            .highlight_style(Style::default()
                .add_modifier(Modifier::DIM))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, frame.area(), &mut state);
    }
}
