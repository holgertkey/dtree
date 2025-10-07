use std::path::PathBuf;
use crate::tree_node::TreeNodeRef;

/// Search functionality for finding files and directories
pub struct Search {
    pub mode: bool,
    pub query: String,
    pub results: Vec<PathBuf>,
    pub selected: usize,
    pub show_results: bool,
    pub focus_on_results: bool,
}

impl Search {
    pub fn new() -> Self {
        Self {
            mode: false,
            query: String::new(),
            results: Vec::new(),
            selected: 0,
            show_results: false,
            focus_on_results: false,
        }
    }

    /// Enter search mode
    pub fn enter_mode(&mut self) {
        self.mode = true;
        self.query.clear();
    }

    /// Exit search mode
    pub fn exit_mode(&mut self) {
        self.mode = false;
        self.query.clear();
    }

    /// Add character to query
    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
    }

    /// Remove last character from query
    pub fn backspace(&mut self) {
        self.query.pop();
    }

    /// Execute search
    pub fn perform_search(&mut self, root: &TreeNodeRef, show_files: bool) {
        self.results.clear();
        self.selected = 0;

        if self.query.is_empty() {
            self.show_results = false;
            return;
        }

        let query_lower = self.query.to_lowercase();
        Self::search_recursive(root, &query_lower, &mut self.results, show_files);

        self.show_results = !self.results.is_empty();
        self.focus_on_results = self.show_results;
        self.mode = false;
    }

    /// Recursive search through tree
    fn search_recursive(node: &TreeNodeRef, query: &str, results: &mut Vec<PathBuf>, show_files: bool) {
        let mut node_borrowed = node.borrow_mut();
        let name_lower = node_borrowed.name.to_lowercase();

        // Check current node
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

            let children = node_borrowed.children.clone();
            drop(node_borrowed);

            for child in &children {
                Self::search_recursive(child, query, results, show_files);
            }
        }
    }

    /// Move selection down in results
    pub fn move_down(&mut self) {
        if self.selected < self.results.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    /// Move selection up in results
    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Get selected result path
    pub fn get_selected_result(&self) -> Option<PathBuf> {
        self.results.get(self.selected).cloned()
    }

    /// Close search results panel
    pub fn close_results(&mut self) {
        self.show_results = false;
        self.results.clear();
        self.selected = 0;
        self.focus_on_results = false;
    }

    /// Toggle focus between tree and search results
    pub fn toggle_focus(&mut self) {
        if self.show_results {
            self.focus_on_results = !self.focus_on_results;
        }
    }
}
