use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Maximum calculation time per directory (5 seconds)
const CALCULATION_TIMEOUT: Duration = Duration::from_secs(5);

/// Maximum number of files to process per directory (to prevent hanging)
const MAX_FILES_TO_PROCESS: usize = 10000;

/// Message types for communication between main thread and size calculation thread
#[derive(Debug)]
pub enum SizeMessage {
    /// Result found (path, size in bytes, is_partial)
    Result(PathBuf, u64, bool),
    /// Calculation done for a path
    Done(PathBuf),
}

/// Task message for worker thread
#[derive(Debug)]
enum TaskMessage {
    Calculate(PathBuf),
    Shutdown,
}

/// Result of size calculation
struct CalculationResult {
    size: u64,
    is_partial: bool, // true if calculation was interrupted
}

/// Cache for directory sizes with async calculation support
pub struct DirSizeCache {
    /// Cache mapping path to (size, is_partial)
    cache: HashMap<PathBuf, (u64, bool)>,
    /// Paths currently being calculated
    calculating: Arc<Mutex<Vec<PathBuf>>>,
    /// Channel for receiving calculation results
    result_receiver: Option<Receiver<SizeMessage>>,
    /// Channel for sending calculation tasks to worker
    task_sender: Option<Sender<TaskMessage>>,
    /// Handle to background worker thread
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl Default for DirSizeCache {
    fn default() -> Self {
        Self::new()
    }
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
    pub fn get(&self, path: &Path) -> Option<(u64, bool)> {
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
                    SizeMessage::Result(path, size, is_partial) => {
                        self.cache.insert(path, (size, is_partial));
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
    pub fn format_size(size: u64, is_partial: bool) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        const TB: u64 = GB * 1024;

        let prefix = if is_partial { ">" } else { "" };

        if size >= TB {
            format!("{}{:.1}T", prefix, size as f64 / TB as f64)
        } else if size >= GB {
            format!("{}{:.1}G", prefix, size as f64 / GB as f64)
        } else if size >= MB {
            format!("{}{:.1}M", prefix, size as f64 / MB as f64)
        } else if size >= KB {
            format!("{}{:.1}K", prefix, size as f64 / KB as f64)
        } else {
            format!("{}{}B", prefix, size)
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
                // Calculate size with timeout and file limit
                let start_time = Instant::now();
                let mut file_count = 0;

                let result = calculate_dir_size_limited(&path, start_time, &mut file_count);

                // Send results
                let _ = result_tx.send(SizeMessage::Result(
                    path.clone(),
                    result.size,
                    result.is_partial,
                ));
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

/// Calculate total size of a directory recursively with limits
fn calculate_dir_size_limited(
    path: &Path,
    start_time: Instant,
    file_count: &mut usize,
) -> CalculationResult {
    let mut total_size = 0u64;
    let mut is_partial = false;

    // Check timeout
    if start_time.elapsed() > CALCULATION_TIMEOUT {
        return CalculationResult {
            size: total_size,
            is_partial: true,
        };
    }

    // Check file limit
    if *file_count >= MAX_FILES_TO_PROCESS {
        return CalculationResult {
            size: total_size,
            is_partial: true,
        };
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            // Periodic checks
            if (*file_count).is_multiple_of(100) {
                // Check timeout every 100 files
                if start_time.elapsed() > CALCULATION_TIMEOUT {
                    return CalculationResult {
                        size: total_size,
                        is_partial: true,
                    };
                }
            }

            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                    *file_count += 1;

                    // Check file limit
                    if *file_count >= MAX_FILES_TO_PROCESS {
                        return CalculationResult {
                            size: total_size,
                            is_partial: true,
                        };
                    }
                } else if metadata.is_dir() {
                    // Recursively calculate subdirectory size
                    let subdir_result =
                        calculate_dir_size_limited(&entry.path(), start_time, file_count);

                    total_size += subdir_result.size;

                    // If subdirectory was partial, mark this as partial too
                    if subdir_result.is_partial {
                        is_partial = true;
                        // Don't continue if we hit a limit
                        break;
                    }
                }
            }
        }
    }

    CalculationResult {
        size: total_size,
        is_partial,
    }
}
