use std::path::PathBuf;
use std::fs;
use anyhow::Result;

#[derive(Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
    pub children: Vec<TreeNode>,
}

impl TreeNode {
    pub fn new(path: PathBuf, depth: usize) -> Result<Self> {
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

    pub fn load_children(&mut self) -> Result<()> {
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

    pub fn toggle_expand(&mut self) -> Result<()> {
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
