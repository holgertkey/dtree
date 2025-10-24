use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

// Import the necessary modules from dtree
// Note: These will need to be public for testing
// We'll test the public API behavior

#[test]
fn test_tail_mode_toggle() {
    // Create a temporary directory and file for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create a test file with multiple lines
    let mut file = File::create(&test_file).unwrap();
    for i in 1..=100 {
        writeln!(file, "Line {}", i).unwrap();
    }

    // Create a FileViewer instance
    let mut file_viewer = dtree::file_viewer::FileViewer::new();

    // Initial state: tail_mode should be false
    assert!(!file_viewer.tail_mode, "Initial tail_mode should be false");

    // Load file in head mode (default)
    let max_lines = 20;
    file_viewer.load_file_with_width(&test_file, Some(80), max_lines, false, "base16-ocean.dark").unwrap();

    // Verify we loaded first N lines (head mode)
    assert!(!file_viewer.tail_mode, "tail_mode should still be false after loading in head mode");
    assert!(!file_viewer.content.is_empty(), "Content should not be empty");
    // First line should be "Line 1"
    assert!(file_viewer.content[0].starts_with("Line 1"), "First line should be 'Line 1' in head mode");

    // Enable tail mode
    file_viewer.enable_tail_mode();
    assert!(file_viewer.tail_mode, "tail_mode should be true after enable_tail_mode()");

    // Reload file in tail mode
    file_viewer.load_file_with_width(&test_file, Some(80), max_lines, false, "base16-ocean.dark").unwrap();

    // Verify we loaded last N lines (tail mode)
    assert!(file_viewer.tail_mode, "tail_mode should persist after reload");
    // The content should contain lines from the end of the file
    // With 100 total lines and max_lines=20, we should see lines starting around line 81
    // Note: There's an extra line at the beginning showing truncation info
    let content_without_header = if file_viewer.content[0].starts_with("[...") {
        &file_viewer.content[1..]
    } else {
        &file_viewer.content[..]
    };

    // Last actual content line should be close to "Line 100"
    let last_line = content_without_header.last().unwrap();
    assert!(last_line.contains("Line 100") || last_line.contains("truncated"),
            "Last lines should be from end of file, got: {}", last_line);

    // Test toggle back to head mode
    file_viewer.enable_head_mode();
    assert!(!file_viewer.tail_mode, "tail_mode should be false after enable_head_mode()");

    // Reload in head mode
    file_viewer.load_file_with_width(&test_file, Some(80), max_lines, false, "base16-ocean.dark").unwrap();
    assert!(file_viewer.content[0].starts_with("Line 1"), "Should be back to showing first lines");
}

#[test]
fn test_tail_mode_scroll_position() {
    // Create a temporary directory and file for testing
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create a test file with multiple lines
    let mut file = File::create(&test_file).unwrap();
    for i in 1..=50 {
        writeln!(file, "Line {}", i).unwrap();
    }

    let mut file_viewer = dtree::file_viewer::FileViewer::new();

    // Load file and enable tail mode
    file_viewer.enable_tail_mode();
    file_viewer.load_file_with_width(&test_file, Some(80), 20, false, "base16-ocean.dark").unwrap();

    // Initial scroll should be 0 after loading
    assert_eq!(file_viewer.scroll, 0, "Initial scroll should be 0 after file load");

    // Simulate End key behavior: scroll to end
    let visible_height = 10;
    file_viewer.scroll_to_end(visible_height);

    // Scroll should now be set to show the last visible_height lines
    let expected_scroll = file_viewer.content.len().saturating_sub(visible_height);
    assert_eq!(file_viewer.scroll, expected_scroll,
               "scroll_to_end should set scroll to content.len() - visible_height");
}

#[test]
fn test_tail_mode_persistence_across_reloads() {
    // Test that tail_mode persists when reloading the same file
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    let mut file = File::create(&test_file).unwrap();
    for i in 1..=100 {
        writeln!(file, "Line {}", i).unwrap();
    }

    let mut file_viewer = dtree::file_viewer::FileViewer::new();

    // Enable tail mode and load file
    file_viewer.enable_tail_mode();
    file_viewer.load_file_with_width(&test_file, Some(80), 20, false, "base16-ocean.dark").unwrap();
    assert!(file_viewer.tail_mode, "tail_mode should be true after first load");

    // Reload file again (simulating navigation or refresh)
    file_viewer.load_file_with_width(&test_file, Some(80), 20, false, "base16-ocean.dark").unwrap();
    assert!(file_viewer.tail_mode, "tail_mode should persist across reloads");

    // Disable tail mode
    file_viewer.enable_head_mode();
    file_viewer.load_file_with_width(&test_file, Some(80), 20, false, "base16-ocean.dark").unwrap();
    assert!(!file_viewer.tail_mode, "tail_mode should be false after enable_head_mode");
}

#[test]
fn test_can_use_tail_mode() {
    let temp_dir = TempDir::new().unwrap();

    // Test with text file
    let text_file = temp_dir.path().join("text.txt");
    let mut file = File::create(&text_file).unwrap();
    writeln!(file, "Hello world").unwrap();

    let mut file_viewer = dtree::file_viewer::FileViewer::new();
    file_viewer.load_file_with_width(&text_file, Some(80), 20, false, "base16-ocean.dark").unwrap();

    // Text file should support tail mode
    assert!(file_viewer.can_use_tail_mode(), "Text files should support tail mode");
    assert!(!file_viewer.is_binary, "Text file should not be marked as binary");

    // Test with empty path (like help content)
    let empty_viewer = dtree::file_viewer::FileViewer::new();
    assert!(!empty_viewer.can_use_tail_mode(), "Empty path should not support tail mode");
}
