use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::tree_node::{TreeNode, TreeNodeRef};
use std::rc::Rc;
use std::cell::RefCell;

/// Navigation logic for tree traversal and manipulation
pub struct Navigation {
    pub root: TreeNodeRef,
    pub flat_list: Vec<TreeNodeRef>,
    pub selected: usize,
}

impl Navigation {
    pub fn new(start_path: PathBuf, show_files: bool) -> Result<Self> {
        let mut root = TreeNode::new(start_path, 0)?;
        root.load_children(show_files)?;
        root.is_expanded = true;
        let root = Rc::new(RefCell::new(root));

        let mut nav = Self {
            root,
            flat_list: Vec::new(),
            selected: 0,
        };

        nav.rebuild_flat_list();
        Ok(nav)
    }

    /// Rebuild flat list of visible nodes
    pub fn rebuild_flat_list(&mut self) {
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

    /// Get currently selected node
    pub fn get_selected_node(&self) -> Option<TreeNodeRef> {
        self.flat_list.get(self.selected).map(|n| Rc::clone(n))
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected < self.flat_list.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Toggle node expansion at path
    pub fn toggle_node(&mut self, path: &Path, show_files: bool) -> Result<()> {
        Self::toggle_node_recursive(&self.root, path, show_files)?;
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

    /// Reload tree with new show_files setting
    pub fn reload_tree(&mut self, show_files: bool) -> Result<()> {
        Self::reload_node_recursive(&self.root, show_files)?;
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

    /// Navigate to parent directory
    pub fn go_to_parent(&mut self, show_files: bool) -> Result<()> {
        let parent_path = {
            let root_borrowed = self.root.borrow();
            root_borrowed.path.parent().map(|p| p.to_path_buf())
        };

        if let Some(parent_path) = parent_path {
            let current_path = self.root.borrow().path.clone();

            let mut new_root = TreeNode::new(parent_path, 0)?;
            new_root.load_children(show_files)?;
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

        Ok(())
    }

    /// Navigate to arbitrary directory (for bookmarks)
    pub fn go_to_directory(&mut self, target_path: PathBuf, show_files: bool) -> Result<()> {
        if !target_path.is_dir() {
            return Ok(());
        }

        let mut new_root = TreeNode::new(target_path, 0)?;
        new_root.load_children(show_files)?;
        new_root.is_expanded = true;

        self.root = Rc::new(RefCell::new(new_root));
        self.rebuild_flat_list();
        self.selected = 0;

        Ok(())
    }

    /// Expand path to node (for search results)
    pub fn expand_path_to_node(&mut self, target_path: &PathBuf, show_files: bool) -> Result<()> {
        Self::expand_path_recursive(&self.root, target_path, show_files)?;
        self.rebuild_flat_list();

        // Find and select element in tree
        for (idx, node) in self.flat_list.iter().enumerate() {
            if node.borrow().path == *target_path {
                self.selected = idx;
                break;
            }
        }

        Ok(())
    }

    fn expand_path_recursive(node: &TreeNodeRef, target_path: &PathBuf, show_files: bool) -> Result<bool> {
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
            if Self::expand_path_recursive(child, target_path, show_files)? {
                return Ok(true);
            }
        }

        Ok(false)
    }
}
