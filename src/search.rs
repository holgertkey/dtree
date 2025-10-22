// Allow many arguments for recursive search function - it needs context for deep traversal
#![allow(clippy::too_many_arguments)]

use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use crossbeam_channel::{bounded, unbounded, Sender, Receiver};
use crate::tree_node::TreeNodeRef;

/// Messages from search thread to main thread
#[derive(Debug, Clone)]
pub enum SearchMessage {
    /// Found a matching path (path, is_dir, score, match_indices)
    Result(PathBuf, bool, Option<i64>, Option<Vec<usize>>),
    /// Progress update: number of directories scanned
    Progress(usize),
    /// Search completed
    Done,
}

/// Search result with metadata
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub path: PathBuf,
    pub is_dir: bool,
    pub score: Option<i64>,        // Fuzzy match score (None for exact match)
    pub match_indices: Option<Vec<usize>>, // Character positions that matched (for highlighting)
}

/// Search functionality for finding files and directories
pub struct Search {
    pub mode: bool,
    pub query: String,
    pub fuzzy_mode: bool,  // True if query starts with '/'
    pub results: Vec<SearchResult>,
    pub selected: usize,
    pub show_results: bool,
    pub focus_on_results: bool,

    // Async search state
    pub is_searching: bool,
    pub scanned_count: usize,
    search_thread: Option<JoinHandle<()>>,
    cancel_sender: Option<Sender<()>>,
    result_receiver: Option<Receiver<SearchMessage>>,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    pub fn new() -> Self {
        Self {
            mode: false,
            query: String::new(),
            fuzzy_mode: false,
            results: Vec::new(),
            selected: 0,
            show_results: false,
            focus_on_results: false,
            is_searching: false,
            scanned_count: 0,
            search_thread: None,
            cancel_sender: None,
            result_receiver: None,
        }
    }

    /// Enter search mode
    pub fn enter_mode(&mut self) {
        self.mode = true;
        self.query.clear();
        self.fuzzy_mode = false;
    }

    /// Exit search mode
    pub fn exit_mode(&mut self) {
        self.mode = false;
        self.query.clear();
        self.fuzzy_mode = false;
    }

    /// Add character to query
    pub fn add_char(&mut self, c: char) {
        self.query.push(c);
        self.update_fuzzy_mode();
    }

    /// Remove last character from query
    pub fn backspace(&mut self) {
        self.query.pop();
        self.update_fuzzy_mode();
    }

    /// Update fuzzy mode based on query
    fn update_fuzzy_mode(&mut self) {
        self.fuzzy_mode = self.query.starts_with('/');
    }

    /// Get actual search query (without leading '/' if in fuzzy mode)
    fn get_search_query(&self) -> &str {
        if self.fuzzy_mode && self.query.len() > 1 {
            &self.query[1..]
        } else if self.fuzzy_mode {
            "" // Only '/' entered, empty query
        } else {
            &self.query
        }
    }

    /// Execute two-phase search: quick + deep background scan
    pub fn perform_search(&mut self, root: &TreeNodeRef, show_files: bool, show_hidden: bool, follow_symlinks: bool) {
        // Cancel any existing search
        self.cancel_search();

        self.results.clear();
        self.selected = 0;
        self.scanned_count = 0;

        let search_query = self.get_search_query();

        // Don't search if query is empty (e.g., user entered just '/')
        if search_query.is_empty() {
            self.show_results = false;
            self.is_searching = false;
            return;
        }

        let query_lower = search_query.to_lowercase();
        let is_fuzzy = self.fuzzy_mode;

        // Phase 1: Quick search through already loaded nodes
        self.search_loaded_nodes(root, &query_lower, show_files, show_hidden, is_fuzzy);

        // Phase 2: Deep search in background thread
        self.spawn_deep_search(root, query_lower, show_files, show_hidden, follow_symlinks, is_fuzzy);

        self.show_results = true;
        self.focus_on_results = true; // Always focus on results after search
        self.mode = false;
        self.is_searching = true;
    }

    /// Phase 1: Quick search through already loaded (visible) nodes
    fn search_loaded_nodes(&mut self, node: &TreeNodeRef, query: &str, show_files: bool, show_hidden: bool, fuzzy: bool) {
        use fuzzy_matcher::FuzzyMatcher;
        use fuzzy_matcher::skim::SkimMatcherV2;

        let node_borrowed = node.borrow();
        let name_lower = node_borrowed.name.to_lowercase();

        // Check if node is hidden (starts with .)
        let is_hidden = node_borrowed.name.starts_with('.');
        if !show_hidden && is_hidden {
            // Skip hidden files/directories if show_hidden is false
            return;
        }

        // Check current node
        if show_files || node_borrowed.is_dir {
            if fuzzy {
                // Fuzzy matching
                let matcher = SkimMatcherV2::default();
                if let Some((score, indices)) = matcher.fuzzy_indices(&name_lower, query) {
                    self.results.push(SearchResult {
                        path: node_borrowed.path.clone(),
                        is_dir: node_borrowed.is_dir,
                        score: Some(score),
                        match_indices: Some(indices),
                    });
                }
            } else {
                // Exact substring matching
                if name_lower.contains(query) {
                    self.results.push(SearchResult {
                        path: node_borrowed.path.clone(),
                        is_dir: node_borrowed.is_dir,
                        score: None,
                        match_indices: None,
                    });
                }
            }
        }

        // Recursively search already loaded children
        if node_borrowed.is_expanded {
            let children = node_borrowed.children.clone();
            drop(node_borrowed);

            for child in &children {
                self.search_loaded_nodes(child, query, show_files, show_hidden, fuzzy);
            }
        }
    }

    /// Phase 2: Spawn background thread for deep search
    fn spawn_deep_search(&mut self, root: &TreeNodeRef, query: String, show_files: bool, show_hidden: bool, follow_symlinks: bool, fuzzy: bool) {
        let (result_tx, result_rx) = unbounded();
        let (cancel_tx, cancel_rx) = bounded(1);

        // Clone root node for thread (Rc can't be sent across threads, so we need path)
        let root_path = root.borrow().path.clone();

        // Spawn search thread
        let handle = thread::spawn(move || {
            Self::deep_search_recursive(&root_path, &query, &result_tx, &cancel_rx, show_files, show_hidden, follow_symlinks, fuzzy, &mut 0);
            let _ = result_tx.send(SearchMessage::Done);
        });

        self.search_thread = Some(handle);
        self.cancel_sender = Some(cancel_tx);
        self.result_receiver = Some(result_rx);
    }

    /// Recursive deep search in background thread
    fn deep_search_recursive(
        path: &PathBuf,
        query: &str,
        result_tx: &Sender<SearchMessage>,
        cancel_rx: &Receiver<()>,
        show_files: bool,
        show_hidden: bool,
        follow_symlinks: bool,
        fuzzy: bool,
        scanned: &mut usize,
    ) {
        use fuzzy_matcher::FuzzyMatcher;
        use fuzzy_matcher::skim::SkimMatcherV2;

        // Check for cancellation
        if cancel_rx.try_recv().is_ok() {
            return;
        }

        // Check if entry is a symlink and whether to follow it
        if !follow_symlinks {
            if let Ok(metadata) = std::fs::symlink_metadata(path) {
                if metadata.is_symlink() {
                    return; // Skip symlinks if follow_symlinks is false
                }
            }
        }

        // Check if this is a directory
        let is_dir = path.is_dir();

        if !is_dir && !show_files {
            return; // Skip files if not in file viewing mode
        }

        // Check if file/directory is hidden (starts with .)
        if !show_hidden {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    return; // Skip hidden files/directories
                }
            }
        }

        // Check if name matches query
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            let name_lower = name.to_lowercase();

            if fuzzy {
                // Fuzzy matching
                let matcher = SkimMatcherV2::default();
                if let Some((score, indices)) = matcher.fuzzy_indices(&name_lower, query) {
                    let _ = result_tx.send(SearchMessage::Result(
                        path.clone(),
                        is_dir,
                        Some(score),
                        Some(indices),
                    ));
                }
            } else {
                // Exact substring matching
                if name_lower.contains(query) {
                    let _ = result_tx.send(SearchMessage::Result(
                        path.clone(),
                        is_dir,
                        None,
                        None,
                    ));
                }
            }
        }

        // If directory, scan children
        if is_dir {
            *scanned += 1;

            // Send progress update every 100 directories
            if (*scanned).is_multiple_of(100) {
                let _ = result_tx.send(SearchMessage::Progress(*scanned));
            }

            // Read directory entries
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    // Check for cancellation frequently
                    if cancel_rx.try_recv().is_ok() {
                        return;
                    }

                    let child_path = entry.path();
                    Self::deep_search_recursive(&child_path, query, result_tx, cancel_rx, show_files, show_hidden, follow_symlinks, fuzzy, scanned);
                }
            }
        }
    }

    /// Poll for new search results from background thread
    pub fn poll_results(&mut self) -> bool {
        let mut has_updates = false;
        let mut search_done = false;

        if let Some(ref rx) = self.result_receiver {
            // Process all available messages
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    SearchMessage::Result(path, is_dir, score, match_indices) => {
                        // Check if we already have this result (from quick search)
                        if !self.results.iter().any(|r| r.path == path) {
                            self.results.push(SearchResult {
                                path,
                                is_dir,
                                score,
                                match_indices,
                            });
                            has_updates = true;
                        }
                    }
                    SearchMessage::Progress(count) => {
                        self.scanned_count = count;
                        has_updates = true;
                    }
                    SearchMessage::Done => {
                        search_done = true;
                        has_updates = true;
                    }
                }
            }
        }

        // Clean up if search is done
        if search_done {
            self.is_searching = false;
            self.search_thread = None;
            self.cancel_sender = None;
            self.result_receiver = None;

            // Sort results by score in fuzzy mode (highest score first)
            if self.fuzzy_mode {
                self.results.sort_by(|a, b| {
                    match (a.score, b.score) {
                        (Some(score_a), Some(score_b)) => score_b.cmp(&score_a), // Descending order
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
        }

        has_updates
    }

    /// Cancel current search
    pub fn cancel_search(&mut self) {
        if let Some(cancel_tx) = self.cancel_sender.take() {
            let _ = cancel_tx.send(());
        }

        if let Some(handle) = self.search_thread.take() {
            let _ = handle.join();
        }

        self.result_receiver = None;
        self.is_searching = false;
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

    /// Get number of search results
    pub fn get_results_count(&self) -> usize {
        self.results.len()
    }

    /// Set selected index (with bounds checking)
    pub fn set_selected(&mut self, index: usize) {
        if index < self.results.len() {
            self.selected = index;
        }
    }

    /// Get selected result path
    pub fn get_selected_result(&self) -> Option<PathBuf> {
        self.results.get(self.selected).map(|r| r.path.clone())
    }

    /// Close search results panel
    pub fn close_results(&mut self) {
        self.cancel_search();
        self.show_results = false;
        self.results.clear();
        self.selected = 0;
        self.focus_on_results = false;
        self.scanned_count = 0;
    }

    /// Toggle focus between tree and search results
    pub fn toggle_focus(&mut self) {
        if self.show_results {
            self.focus_on_results = !self.focus_on_results;
        }
    }

    /// Check if search is active
    pub fn is_active(&self) -> bool {
        self.is_searching
    }
}

impl Drop for Search {
    fn drop(&mut self) {
        self.cancel_search();
    }
}
