use std::path::{Path, PathBuf};
use std::collections::HashMap;
use anyhow::Result;
use crate::tree_node::{TreeNode, TreeNodeRef};
use std::rc::Rc;
use std::cell::RefCell;

/// Navigation logic for tree traversal and manipulation
pub struct Navigation {
    pub root: TreeNodeRef,
    pub flat_list: Vec<TreeNodeRef>,
    pub selected: usize,
    pub show_hidden: bool,
    pub follow_symlinks: bool,
    // Performance optimization: HashMap for O(1) path lookup
    path_to_index: HashMap<PathBuf, usize>,
}

impl Navigation {
    pub fn new(start_path: PathBuf, show_files: bool, show_hidden: bool, follow_symlinks: bool) -> Result<Self> {
        let mut root = TreeNode::new(start_path, 0)?;
        root.load_children(show_files, show_hidden, follow_symlinks)?;
        root.is_expanded = true;
        let root = Rc::new(RefCell::new(root));

        let mut nav = Self {
            root,
            flat_list: Vec::new(),
            selected: 0,
            show_hidden,
            follow_symlinks,
            path_to_index: HashMap::new(),
        };

        nav.rebuild_flat_list();
        Ok(nav)
    }

    /// Rebuild flat list of visible nodes and update path index
    pub fn rebuild_flat_list(&mut self) {
        self.flat_list.clear();
        self.path_to_index.clear();
        Self::collect_visible_nodes(&self.root, &mut self.flat_list);

        // Build path → index mapping for O(1) lookups
        for (idx, node) in self.flat_list.iter().enumerate() {
            let path = node.borrow().path.clone();
            self.path_to_index.insert(path, idx);
        }
    }

    fn collect_visible_nodes(node: &TreeNodeRef, result: &mut Vec<TreeNodeRef>) {
        result.push(Rc::clone(node));

        // Check if node is expanded and get children count
        let (is_expanded, children_count) = {
            let node_borrowed = node.borrow();
            (node_borrowed.is_expanded, node_borrowed.children.len())
        };

        if is_expanded {
            // Recursively collect children without cloning the entire vector
            for i in 0..children_count {
                let child = Rc::clone(&node.borrow().children[i]);
                Self::collect_visible_nodes(&child, result);
            }
        }
    }

    /// Get currently selected node
    pub fn get_selected_node(&self) -> Option<TreeNodeRef> {
        self.flat_list.get(self.selected).map(Rc::clone)
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
    /// Returns Some(error_message) if node has error after toggle, None otherwise
    pub fn toggle_node(&mut self, path: &Path, show_files: bool) -> Result<Option<String>> {
        // Try incremental update first
        if let Some(index) = self.path_to_index.get(path).copied() {
            if index < self.flat_list.len() {
                let node = &self.flat_list[index];
                let was_expanded = node.borrow().is_expanded;

                // Toggle the node
                let error_msg = {
                    let mut node_borrowed = node.borrow_mut();
                    node_borrowed.toggle_expand(show_files, self.show_hidden, self.follow_symlinks)?;
                    if node_borrowed.has_error {
                        node_borrowed.error_message.clone()
                    } else {
                        None
                    }
                };

                // Incremental update of flat_list
                if was_expanded {
                    // Node was expanded, now collapsed - remove children from flat_list
                    self.remove_descendants_from_flat_list(index);
                } else {
                    // Node was collapsed, now expanded - add children to flat_list
                    self.insert_children_into_flat_list(index);
                }

                return Ok(error_msg);
            }
        }

        // Fallback to full rebuild if node not found in flat_list
        let error_msg = Self::toggle_node_recursive(&self.root, path, show_files, self.show_hidden, self.follow_symlinks)?;
        self.rebuild_flat_list();
        Ok(error_msg)
    }

    fn toggle_node_recursive(node: &TreeNodeRef, target_path: &Path, show_files: bool, show_hidden: bool, follow_symlinks: bool) -> Result<Option<String>> {
        // Check if this is the target node
        {
            let mut node_borrowed = node.borrow_mut();
            if node_borrowed.path == target_path {
                node_borrowed.toggle_expand(show_files, show_hidden, follow_symlinks)?;
                // Check if node has error after toggle
                let error_msg = if node_borrowed.has_error {
                    node_borrowed.error_message.clone()
                } else {
                    None
                };
                return Ok(error_msg);
            }
        }

        // Recursively search children without cloning
        // We need to drop the borrow before recursing
        let children_count = node.borrow().children.len();
        for i in 0..children_count {
            let child = Rc::clone(&node.borrow().children[i]);
            if let Some(error_msg) = Self::toggle_node_recursive(&child, target_path, show_files, show_hidden, follow_symlinks)? {
                return Ok(Some(error_msg));
            }
        }

        Ok(None)
    }

    /// Reload tree with new show_files setting
    pub fn reload_tree(&mut self, show_files: bool) -> Result<()> {
        Self::reload_node_recursive(&self.root, show_files, self.show_hidden, self.follow_symlinks)?;
        self.rebuild_flat_list();
        Ok(())
    }

    fn reload_node_recursive(node: &TreeNodeRef, show_files: bool, show_hidden: bool, follow_symlinks: bool) -> Result<()> {
        // Check if we need to reload this node
        let should_reload = {
            let node_borrowed = node.borrow();
            node_borrowed.is_expanded && node_borrowed.is_dir
        };

        if should_reload {
            // Clear children and reload with new mode
            {
                let mut node_borrowed = node.borrow_mut();
                node_borrowed.children.clear();
                node_borrowed.load_children(show_files, show_hidden, follow_symlinks)?;
            }

            // Recursively reload child nodes without cloning
            let children_count = node.borrow().children.len();
            for i in 0..children_count {
                let child = Rc::clone(&node.borrow().children[i]);
                Self::reload_node_recursive(&child, show_files, show_hidden, follow_symlinks)?;
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
            new_root.load_children(show_files, self.show_hidden, self.follow_symlinks)?;
            new_root.is_expanded = true;

            self.root = Rc::new(RefCell::new(new_root));
            self.rebuild_flat_list();

            // Find and select previous directory using HashMap (O(1) instead of O(n))
            if let Some(&idx) = self.path_to_index.get(&current_path) {
                self.selected = idx;
            }
        }

        Ok(())
    }

    /// Navigate to arbitrary directory (for bookmarks)
    /// Returns Some(error_message) if directory cannot be accessed, None otherwise
    pub fn go_to_directory(&mut self, target_path: PathBuf, show_files: bool) -> Result<Option<String>> {
        if !target_path.is_dir() {
            return Ok(None);
        }

        // Save current state in case we need to restore it
        let old_root = Rc::clone(&self.root);
        let old_selected = self.selected;

        let mut new_root = TreeNode::new(target_path, 0)?;
        new_root.load_children(show_files, self.show_hidden, self.follow_symlinks)?;
        new_root.is_expanded = true;

        // Check if the new root has an error
        if new_root.has_error {
            // Restore previous state - don't change directory
            self.root = old_root;
            self.selected = old_selected;
            return Ok(new_root.error_message);
        }

        // Success - update to new root
        self.root = Rc::new(RefCell::new(new_root));
        self.rebuild_flat_list();
        self.selected = 0;

        Ok(None)
    }

    /// Expand path to node (for search results)
    pub fn expand_path_to_node(&mut self, target_path: &PathBuf, show_files: bool) -> Result<()> {
        Self::expand_path_recursive(&self.root, target_path, show_files, self.show_hidden, self.follow_symlinks)?;
        self.rebuild_flat_list();

        // Find and select element in tree using HashMap (O(1) instead of O(n))
        if let Some(&idx) = self.path_to_index.get(target_path) {
            self.selected = idx;
        }

        Ok(())
    }

    fn expand_path_recursive(node: &TreeNodeRef, target_path: &PathBuf, show_files: bool, show_hidden: bool, follow_symlinks: bool) -> Result<bool> {
        // Check if this is the target node or if target is a descendant
        {
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
                node_borrowed.load_children(show_files, show_hidden, follow_symlinks)?;
            }

            // Expand current node
            node_borrowed.is_expanded = true;
        }

        // Recursively search in children without cloning
        let children_count = node.borrow().children.len();
        for i in 0..children_count {
            let child = Rc::clone(&node.borrow().children[i]);
            if Self::expand_path_recursive(&child, target_path, show_files, show_hidden, follow_symlinks)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Remove all descendants of node at given index from flat_list (when collapsing)
    fn remove_descendants_from_flat_list(&mut self, parent_index: usize) {
        let parent_depth = self.flat_list[parent_index].borrow().depth;

        // Find the range of descendants to remove
        // All nodes after parent with depth > parent_depth are descendants
        let mut remove_count = 0;
        for i in (parent_index + 1)..self.flat_list.len() {
            if self.flat_list[i].borrow().depth > parent_depth {
                remove_count += 1;
            } else {
                break; // Found a sibling or ancestor, stop
            }
        }

        // Remove descendants
        if remove_count > 0 {
            self.flat_list.drain((parent_index + 1)..(parent_index + 1 + remove_count));
        }

        // Rebuild path_to_index mapping
        self.rebuild_path_index();
    }

    /// Insert children of node at given index into flat_list (when expanding)
    fn insert_children_into_flat_list(&mut self, parent_index: usize) {
        let node = &self.flat_list[parent_index];

        // Collect all visible descendants of the newly expanded node
        let mut new_nodes = Vec::new();
        let (is_expanded, children_count) = {
            let node_borrowed = node.borrow();
            (node_borrowed.is_expanded, node_borrowed.children.len())
        };

        if is_expanded {
            for i in 0..children_count {
                let child = Rc::clone(&node.borrow().children[i]);
                Self::collect_visible_nodes(&child, &mut new_nodes);
            }
        }

        // Insert new nodes after parent
        if !new_nodes.is_empty() {
            let insert_pos = parent_index + 1;
            self.flat_list.splice(insert_pos..insert_pos, new_nodes);
        }

        // Rebuild path_to_index mapping
        self.rebuild_path_index();
    }

    /// Rebuild only the path_to_index HashMap (faster than full rebuild)
    fn rebuild_path_index(&mut self) {
        self.path_to_index.clear();
        for (idx, node) in self.flat_list.iter().enumerate() {
            let path = node.borrow().path.clone();
            self.path_to_index.insert(path, idx);
        }
    }
}
