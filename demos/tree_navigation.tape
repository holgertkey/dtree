# VHS tape for Tree Navigation demo

Output ../docs/assets/tree_navigation.gif

Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set TypingSpeed 100ms
Set PlaybackSpeed 1.0

# Ensure dtree, ttyd, and system commands are in PATH
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"

# Hide startup commands - don't show cd and dtree launch
Hide
Type "cd /home/holger/github.com/holgertkey/dtree"
Enter
Type "/home/holger/bin/dtree"
Enter
Sleep 1s
Show

# Show navigation
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Expand directory
Type "l"
Sleep 500ms

# Navigate expanded content
Type "z"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Collapse
Type "h"
Sleep 500ms

# Navigate to different directory
Type "k"
Sleep 300ms
Type "k"
Sleep 300ms

# Expand another directory
Type "l"
Sleep 500ms

# Enter directory (change root)
Type "j"
Sleep 300ms
Enter
Sleep 1s

# Go to parent
Type "u"
Sleep 1s

# Exit
Type "q"
Sleep 500ms
