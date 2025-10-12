use std::path::PathBuf;
use std::thread::{self, JoinHandle};
use crossbeam_channel::{bounded, unbounded, Sender, Receiver};
use crate::tree_node::TreeNodeRef;

/// Messages from search thread to main thread
#[derive(Debug, Clone)]
pub enum SearchMessage {
    /// Found a matching path (path, is_dir)
    Result(PathBuf, bool),
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
}

/// Search functionality for finding files and directories
pub struct Search {
    pub mode: bool,
    pub query: String,
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

impl Search {
    pub fn new() -> Self {
        Self {
            mode: false,
            query: String::new(),
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

    /// Execute two-phase search: quick + deep background scan
    pub fn perform_search(&mut self, root: &TreeNodeRef, show_files: bool) {
        // Cancel any existing search
        self.cancel_search();

        self.results.clear();
        self.selected = 0;
        self.scanned_count = 0;

        if self.query.is_empty() {
            self.show_results = false;
            self.is_searching = false;
            return;
        }

        let query_lower = self.query.to_lowercase();

        // Phase 1: Quick search through already loaded nodes
        self.search_loaded_nodes(root, &query_lower, show_files);

        // Phase 2: Deep search in background thread
        self.spawn_deep_search(root, query_lower, show_files);

        self.show_results = true;
        self.focus_on_results = !self.results.is_empty();
        self.mode = false;
        self.is_searching = true;
    }

    /// Phase 1: Quick search through already loaded (visible) nodes
    fn search_loaded_nodes(&mut self, node: &TreeNodeRef, query: &str, show_files: bool) {
        let node_borrowed = node.borrow();
        let name_lower = node_borrowed.name.to_lowercase();

        // Check current node
        if show_files || node_borrowed.is_dir {
            if name_lower.contains(query) {
                self.results.push(SearchResult {
                    path: node_borrowed.path.clone(),
                    is_dir: node_borrowed.is_dir,
                });
            }
        }

        // Recursively search already loaded children
        if node_borrowed.is_expanded {
            let children = node_borrowed.children.clone();
            drop(node_borrowed);

            for child in &children {
                self.search_loaded_nodes(child, query, show_files);
            }
        }
    }

    /// Phase 2: Spawn background thread for deep search
    fn spawn_deep_search(&mut self, root: &TreeNodeRef, query: String, show_files: bool) {
        let (result_tx, result_rx) = unbounded();
        let (cancel_tx, cancel_rx) = bounded(1);

        // Clone root node for thread (Rc can't be sent across threads, so we need path)
        let root_path = root.borrow().path.clone();

        // Spawn search thread
        let handle = thread::spawn(move || {
            Self::deep_search_recursive(&root_path, &query, &result_tx, &cancel_rx, show_files, &mut 0);
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
        scanned: &mut usize,
    ) {
        // Check for cancellation
        if cancel_rx.try_recv().is_ok() {
            return;
        }

        // Check if this is a directory
        let is_dir = path.is_dir();

        if !is_dir && !show_files {
            return; // Skip files if not in file viewing mode
        }

        // Check if name matches query
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.to_lowercase().contains(query) {
                let _ = result_tx.send(SearchMessage::Result(path.clone(), is_dir));
            }
        }

        // If directory, scan children
        if is_dir {
            *scanned += 1;

            // Send progress update every 100 directories
            if *scanned % 100 == 0 {
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
                    Self::deep_search_recursive(&child_path, query, result_tx, cancel_rx, show_files, scanned);
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
                    SearchMessage::Result(path, is_dir) => {
                        // Check if we already have this result (from quick search)
                        if !self.results.iter().any(|r| r.path == path) {
                            self.results.push(SearchResult { path, is_dir });
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
