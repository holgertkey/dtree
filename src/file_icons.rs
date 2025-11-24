use std::path::Path;

/// Get icon for a file or directory based on its name/extension
/// Uses Nerd Fonts icons for rich terminals
pub fn get_icon(path: &Path, is_dir: bool, use_nerd_fonts: bool) -> &'static str {
    if !use_nerd_fonts {
        // Fallback mode for terminals without nerd-fonts
        return if is_dir { "📁" } else { "📄" };
    }

    if is_dir {
        // Directory icons
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        return match name {
            ".git" => "\u{f1d3}",           //
            ".github" => "\u{f408}",        //
            ".config" => "\u{e5fc}",        //
            "node_modules" => "\u{e718}",   //
            "target" => "\u{f140}",         //
            "build" => "\u{f0ad}",          //
            "dist" => "\u{f410}",           //
            "bin" => "\u{e5fc}",            //
            "src" => "\u{f121}",            //
            "test" | "tests" => "\u{f0c8}", //
            "docs" | "doc" => "\u{f02d}",   //
            "images" | "img" => "\u{f1c5}", //
            _ => "\u{f115}",                //
        };
    }

    // File icons based on extension
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    // Check for specific filenames first
    match name {
        // Configuration files
        "Cargo.toml" | "Cargo.lock" => return "\u{e7a8}", //
        "package.json" | "package-lock.json" => return "\u{e718}", //
        "Makefile" | "makefile" => return "\u{e779}",     //
        "Dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => return "\u{f308}", //
        ".gitignore" | ".gitattributes" | ".gitmodules" => return "\u{f1d3}", //
        "LICENSE" | "LICENSE.md" | "LICENSE.txt" => return "\u{f0219}", //
        "README.md" | "README" | "readme.md" | "readme" => return "\u{f48a}", //
        ".bashrc" | ".zshrc" | ".bash_profile" => return "\u{f489}", //
        ".vimrc" | ".nvimrc" => return "\u{e62b}",        //
        _ => {}
    }

    // Check by extension
    match extension {
        // Programming languages
        "rs" => "\u{e7a8}",                           //
        "go" => "\u{e627}",                           //
        "py" | "pyc" | "pyd" | "pyo" => "\u{e606}",   //
        "js" | "mjs" | "cjs" => "\u{e74e}",           //
        "ts" => "\u{e628}",                           //
        "jsx" | "tsx" => "\u{e7ba}",                  //
        "java" => "\u{e738}",                         //
        "c" => "\u{e61e}",                            //
        "cpp" | "cc" | "cxx" | "c++" => "\u{e61d}",   //
        "h" | "hpp" | "hh" | "hxx" => "\u{f0fd}",     //
        "cs" => "\u{f81a}",                           //
        "php" => "\u{e73d}",                          //
        "rb" | "erb" => "\u{e791}",                   //
        "lua" => "\u{e620}",                          //
        "vim" => "\u{e62b}",                          //
        "sh" | "bash" | "zsh" | "fish" => "\u{f489}", //

        // Web
        "html" | "htm" => "\u{f13b}",                   //
        "css" | "scss" | "sass" | "less" => "\u{e749}", //
        "json" => "\u{e60b}",                           //
        "xml" => "\u{f05c}",                            //
        "yaml" | "yml" => "\u{f481}",                   //
        "toml" => "\u{e615}",                           //

        // Documents
        "md" | "markdown" => "\u{e73e}", //
        "txt" => "\u{f15c}",             //
        "pdf" => "\u{f1c1}",             //
        "doc" | "docx" => "\u{f1c2}",    //

        // Data
        "sql" => "\u{f1c0}",                       //
        "db" | "sqlite" | "sqlite3" => "\u{e706}", //
        "csv" => "\u{f1c3}",                       //

        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "svg" | "ico" | "webp" => "\u{f1c5}", //

        // Archives
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "\u{f410}", //

        // Audio/Video
        "mp3" | "wav" | "ogg" | "flac" | "m4a" => "\u{f001}", //
        "mp4" | "avi" | "mkv" | "mov" | "webm" => "\u{f03d}", //

        // Executables
        "exe" | "dll" | "so" | "dylib" => "\u{f489}", //

        // Git
        "diff" | "patch" => "\u{f440}", //

        _ => "\u{f15b}", //
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_directory_icons() {
        let path = PathBuf::from(".git");
        assert_eq!(get_icon(&path, true, true), "\u{f1d3}");

        let path = PathBuf::from("src");
        assert_eq!(get_icon(&path, true, true), "\u{f121}");
    }

    #[test]
    fn test_file_icons() {
        let path = PathBuf::from("main.rs");
        assert_eq!(get_icon(&path, false, true), "\u{e7a8}");

        let path = PathBuf::from("README.md");
        assert_eq!(get_icon(&path, false, true), "\u{f48a}");
    }

    #[test]
    fn test_fallback_mode() {
        let path = PathBuf::from("main.rs");
        assert_eq!(get_icon(&path, false, false), "📄");

        let path = PathBuf::from("src");
        assert_eq!(get_icon(&path, true, false), "📁");
    }
}
