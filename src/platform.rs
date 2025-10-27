use std::process::Command;
use anyhow::Result;

/// Open an external program with the given path
/// This function handles platform differences in launching external programs
#[cfg(unix)]
pub fn open_external_program(program: &str, path: &str) -> Result<()> {
    // Unix: use sh shell with proper TTY handling
    // Properly quote the path to handle spaces and special characters
    let shell_cmd = format!("{} '{}' < /dev/tty > /dev/tty 2> /dev/tty",
                            program,
                            path.replace("'", "'\\''"));

    Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()?;

    Ok(())
}

#[cfg(windows)]
pub fn open_external_program(program: &str, path: &str) -> Result<()> {
    // On Windows, handle different program types
    // For file managers like explorer: use cmd /C start
    // For editors: direct execution

    if program.contains("explorer") || program.contains("start") {
        // File manager: use cmd /C start to open without waiting
        Command::new("cmd")
            .args(["/C", "start", "", path])
            .spawn()?; // spawn instead of status to avoid waiting
    } else {
        // Editor or other program: direct execution
        Command::new(program)
            .arg(path)
            .status()?;
    }

    Ok(())
}

/// Check if a path is absolute according to platform conventions
#[cfg(unix)]
pub fn is_absolute_path(path: &str) -> bool {
    // Unix: starts with / or . (relative indicator)
    path.starts_with('/') || path.starts_with('.')
}

#[cfg(windows)]
pub fn is_absolute_path(path: &str) -> bool {
    // Windows: C:\, D:\, \\server\share (UNC), or contains path separator
    path.len() >= 2 && (
        (path.chars().nth(1) == Some(':')) ||  // C:\, D:\
        path.starts_with("\\\\") ||             // \\server\share
        path.contains(std::path::MAIN_SEPARATOR)  // Any path with separators
    )
}

/// Normalize path separators for the current platform
#[cfg(unix)]
pub fn normalize_path_separator(path: &str) -> String {
    // Unix: keep as-is (forward slashes)
    path.to_string()
}

#[cfg(windows)]
pub fn normalize_path_separator(path: &str) -> String {
    // Windows: convert forward slashes to backslashes
    path.replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_absolute_path() {
        #[cfg(unix)]
        {
            assert!(is_absolute_path("/home/user"));
            assert!(is_absolute_path("./relative"));
            assert!(is_absolute_path("../parent"));
            assert!(!is_absolute_path("relative"));
            assert!(!is_absolute_path("dir/subdir"));
        }

        #[cfg(windows)]
        {
            assert!(is_absolute_path("C:\\Users\\user"));
            assert!(is_absolute_path("D:\\Projects"));
            assert!(is_absolute_path("\\\\server\\share"));
            assert!(is_absolute_path("relative\\path"));
            assert!(!is_absolute_path("relative"));
        }
    }

    #[test]
    fn test_normalize_path_separator() {
        #[cfg(unix)]
        {
            assert_eq!(normalize_path_separator("path/to/file"), "path/to/file");
            assert_eq!(normalize_path_separator("path\\to\\file"), "path\\to\\file");
        }

        #[cfg(windows)]
        {
            assert_eq!(normalize_path_separator("path/to/file"), "path\\to\\file");
            assert_eq!(normalize_path_separator("path\\to\\file"), "path\\to\\file");
        }
    }
}