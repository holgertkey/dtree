# VHS tape for Bookmarks demo

Output ../docs/assets/bookmarks.gif

Set FontSize 16
Set Width 1400
Set Height 700
Set Theme "Catppuccin Mocha"
Set TypingSpeed 100ms
Set PlaybackSpeed 1.0

# Ensure dtree is in PATH
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"

# Hide startup commands - don't show cd, alias, and dtree launch
Hide
Type "cd /home/holger/github.com/holgertkey/dtree"
Enter
Type "alias dtree='/home/holger/bin/dtree'"
Enter
Type "dtree"
Enter
Sleep 1s
Show

# Navigate to a directory
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "l"
Sleep 500ms
Type "j"
Sleep 300ms

# Create bookmark
Type "m"
Sleep 800ms

# Type bookmark name
Type "myproject"
Sleep 500ms
Enter
Sleep 1s

# Show bookmark list (without exiting)
Type "'"
Sleep 1.5s

# Navigate bookmarks
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Select bookmark to jump
Enter
Sleep 1s

# Exit bookmark menu if still open
Escape
Sleep 500ms

# Show bookmark management via CLI
Escape
Sleep 500ms

# Clear screen (hide command)
Hide
Type "clear"
Enter
Sleep 300ms
Show

# Show CLI bookmark list (using short command for demo)
Type "dtree -bm list"
Enter
Sleep 2s

# Exit
Escape
Sleep 500ms
