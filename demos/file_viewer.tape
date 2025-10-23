# VHS tape for File Viewer demo

Output ../docs/assets/file_viewer.gif

Set FontSize 16
Set Width 1400
Set Height 700
Set Theme "Catppuccin Mocha"
Set TypingSpeed 100ms
Set PlaybackSpeed 1.0

# Ensure dtree is in PATH
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"

# Hide startup commands - don't show cd and dtree launch
Hide
Type "cd /home/holger/github.com/holgertkey/dtree"
Enter
Type "/home/holger/bin/dtree"
Enter
Sleep 1s
Show

# Navigate to src directory
Type "j"
Sleep 300ms
Type "l"
Sleep 300ms
Type "j"
Sleep 300ms

# Enable split view
Type "s"
Sleep 1s

# Navigate through files to show syntax highlighting
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms

# Scroll file preview
Ctrl+J
Sleep 300ms
Ctrl+J
Sleep 300ms
Ctrl+J
Sleep 800ms

# Enter fullscreen viewer
Type "v"
Sleep 1s

# Toggle line numbers
Type "l"
Sleep 1s

# Scroll in fullscreen
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms

# Return to tree
Type "q"
Sleep 500ms

# Exit
Escape
Sleep 500ms
