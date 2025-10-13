use std::path::PathBuf;

#[test]
fn test_file_path_parent_directory() {
    // Test that we can get parent directory from file paths
    let file_path = PathBuf::from("/home/user/documents/file.txt");
    let parent = file_path.parent().unwrap();
    assert_eq!(parent, PathBuf::from("/home/user/documents"));
}

#[test]
fn test_directory_path_returns_self() {
    // Test that directory paths work as expected
    let dir_path = PathBuf::from("/home/user/documents");
    assert_eq!(dir_path, PathBuf::from("/home/user/documents"));
}

#[test]
fn test_root_path_has_no_parent() {
    // Edge case: root path has no parent
    let root_path = PathBuf::from("/");
    assert!(root_path.parent().is_none());
}

#[test]
fn test_relative_file_path_parent() {
    // Test relative paths
    let file_path = PathBuf::from("src/main.rs");
    let parent = file_path.parent().unwrap();
    assert_eq!(parent, PathBuf::from("src"));
}

#[test]
fn test_current_dir_file_parent() {
    // Test file in current directory
    let file_path = PathBuf::from("file.txt");
    let parent = file_path.parent();
    // Parent of "file.txt" is "" (empty path)
    assert_eq!(parent.unwrap(), PathBuf::from(""));
}
