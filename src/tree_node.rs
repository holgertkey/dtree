use std::path::PathBuf;
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;
use anyhow::Result;

pub type TreeNodeRef = Rc<RefCell<TreeNode>>;

pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub depth: usize,
    pub children: Vec<TreeNodeRef>,
    pub has_error: bool,           // Indicates read/access errors
    pub error_message: Option<String>, // Optional error description
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
            has_error: false,
            error_message: None,
        })
    }

    pub fn load_children(&mut self, show_files: bool) -> Result<()> {
        if !self.is_dir || !self.children.is_empty() {
            return Ok(());
        }

        // Try to read directory
        let entries = match fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(e) => {
                // Mark this node as having an error
                self.has_error = true;
                self.error_message = Some(format!("Cannot read: {}", e));
                return Ok(()); // Don't propagate error, just mark the node
            }
        };

        let mut error_count = 0;
        let mut skipped_entries = Vec::new();

        // Process entries, tracking errors
        for entry in entries {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    let is_dir = path.is_dir();

                    // Show directories always, files only if show_files == true
                    if is_dir || show_files {
                        match TreeNode::new(path.clone(), self.depth + 1) {
                            Ok(node) => {
                                self.children.push(Rc::new(RefCell::new(node)));
                            }
                            Err(e) => {
                                error_count += 1;
                                skipped_entries.push(format!("{}: {}",
                                    path.file_name().unwrap_or_default().to_string_lossy(), e));
                            }
                        }
                    }
                }
                Err(e) => {
                    error_count += 1;
                    skipped_entries.push(format!("unknown entry: {}", e));
                }
            }
        }

        // If we had errors, mark the node and store summary
        if error_count > 0 {
            self.has_error = true;
            if error_count <= 3 {
                self.error_message = Some(skipped_entries.join(", "));
            } else {
                self.error_message = Some(format!("{} entries inaccessible", error_count));
            }
        }

        // Sort: directories first, then files, sorted by name within each group
        self.children.sort_by(|a, b| {
            let a_borrowed = a.borrow();
            let b_borrowed = b.borrow();
            match (a_borrowed.is_dir, b_borrowed.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a_borrowed.name.cmp(&b_borrowed.name),
            }
        });

        Ok(())
    }

    pub fn toggle_expand(&mut self, show_files: bool) -> Result<()> {
        if !self.is_dir {
            return Ok(());
        }

        if self.is_expanded {
            self.is_expanded = false;
        } else {
            self.load_children(show_files)?;
            self.is_expanded = true;
        }

        Ok(())
    }
}
