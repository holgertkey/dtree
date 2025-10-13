use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use crossbeam_channel::{unbounded, Receiver, Sender};

/// Message types for communication between main thread and size calculation thread
#[derive(Debug)]
pub enum SizeMessage {
    /// Result found (path, size in bytes)
    Result(PathBuf, u64),
    /// Calculation done for a path
    Done(PathBuf),
}

/// Task message for worker thread
#[derive(Debug)]
enum TaskMessage {
    Calculate(PathBuf),
    Shutdown,
}

/// Cache for directory sizes with async calculation support
pub struct DirSizeCache {
    /// Cache mapping path to size in bytes
    cache: HashMap<PathBuf, u64>,
    /// Paths currently being calculated
    calculating: Arc<Mutex<Vec<PathBuf>>>,
    /// Channel for receiving calculation results
    result_receiver: Option<Receiver<SizeMessage>>,
    /// Channel for sending calculation tasks to worker
    task_sender: Option<Sender<TaskMessage>>,
    /// Handle to background worker thread
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl DirSizeCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            calculating: Arc::new(Mutex::new(Vec::new())),
            result_receiver: None,
            task_sender: None,
            worker_handle: None,
        }
    }

    /// Initialize worker thread if not already running
    fn ensure_worker_running(&mut self) {
        if self.worker_handle.is_some() {
            return; // Worker already running
        }

        let (task_tx, task_rx) = unbounded();
        let (result_tx, result_rx) = unbounded();

        let calculating = Arc::clone(&self.calculating);

        // Spawn worker thread
        let handle = thread::spawn(move || {
            worker_loop(task_rx, result_tx, calculating);
        });

        self.task_sender = Some(task_tx);
        self.result_receiver = Some(result_rx);
        self.worker_handle = Some(handle);
    }

    /// Get cached size for a path
    pub fn get(&self, path: &Path) -> Option<u64> {
        self.cache.get(path).copied()
    }

    /// Check if a path is currently being calculated
    pub fn is_calculating(&self, path: &Path) -> bool {
        if let Ok(calculating) = self.calculating.lock() {
            calculating.contains(&path.to_path_buf())
        } else {
            false
        }
    }

    /// Start async calculation for a directory
    pub fn calculate_async(&mut self, path: PathBuf) {
        // Don't calculate if already in cache or being calculated
        if self.cache.contains_key(&path) || self.is_calculating(&path) {
            return;
        }

        // Ensure worker is running
        self.ensure_worker_running();

        // Mark as calculating
        if let Ok(mut calculating) = self.calculating.lock() {
            calculating.push(path.clone());
        }

        // Send task to worker
        if let Some(sender) = &self.task_sender {
            let _ = sender.send(TaskMessage::Calculate(path));
        }
    }

    /// Poll for calculation results
    /// Returns true if there were updates
    pub fn poll_results(&mut self) -> bool {
        let mut updated = false;

        if let Some(receiver) = &self.result_receiver {
            // Process all available messages
            while let Ok(msg) = receiver.try_recv() {
                match msg {
                    SizeMessage::Result(path, size) => {
                        self.cache.insert(path, size);
                        updated = true;
                    }
                    SizeMessage::Done(path) => {
                        // Remove from calculating list
                        if let Ok(mut calculating) = self.calculating.lock() {
                            calculating.retain(|p| p != &path);
                        }
                    }
                }
            }
        }

        updated
    }

    /// Cancel ongoing calculations and shutdown worker
    pub fn cancel(&mut self) {
        if let Some(sender) = &self.task_sender {
            let _ = sender.send(TaskMessage::Shutdown);
        }

        self.task_sender = None;
        self.result_receiver = None;

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }

        if let Ok(mut calculating) = self.calculating.lock() {
            calculating.clear();
        }
    }

    /// Clear the cache and shutdown worker
    pub fn clear(&mut self) {
        self.cancel();
        self.cache.clear();
    }

    /// Format size in human-readable format
    pub fn format_size(size: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        if size >= TB {
            format!("{:.1}T", size as f64 / TB as f64)
        } else if size >= GB {
            format!("{:.1}G", size as f64 / GB as f64)
        } else if size >= MB {
            format!("{:.1}M", size as f64 / MB as f64)
        } else if size >= KB {
            format!("{:.1}K", size as f64 / KB as f64)
        } else {
            format!("{}B", size)
        }
    }
}

impl Drop for DirSizeCache {
    fn drop(&mut self) {
        self.cancel();
    }
}

/// Worker thread loop that processes calculation tasks
fn worker_loop(
    task_rx: Receiver<TaskMessage>,
    result_tx: Sender<SizeMessage>,
    calculating: Arc<Mutex<Vec<PathBuf>>>,
) {
    loop {
        match task_rx.recv() {
            Ok(TaskMessage::Calculate(path)) => {
                // Calculate size
                let size = calculate_dir_size(&path);

                // Send results
                let _ = result_tx.send(SizeMessage::Result(path.clone(), size));
                let _ = result_tx.send(SizeMessage::Done(path));
            }
            Ok(TaskMessage::Shutdown) | Err(_) => {
                // Shutdown requested or channel closed
                if let Ok(mut calc) = calculating.lock() {
                    calc.clear();
                }
                break;
            }
        }
    }
}

/// Calculate total size of a directory recursively
fn calculate_dir_size(path: &Path) -> u64 {
    let mut total_size = 0u64;

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    // Recursively calculate subdirectory size
                    total_size += calculate_dir_size(&entry.path());
                }
            }
        }
    }

    total_size
}
