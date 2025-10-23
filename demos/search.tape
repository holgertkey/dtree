# VHS tape for Search demo

Output ../docs/assets/search.gif

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

# Enter search mode
Type "/"
Sleep 500ms

# Type search query
Type "src"
Sleep 500ms

# Execute search
Enter
Sleep 1.5s

# Navigate results
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms

# Jump to selected result
Enter
Sleep 1s

# Try fuzzy search
Type "/"
Sleep 500ms
Type "/cfg"
Sleep 500ms
Enter
Sleep 1.5s

# Navigate fuzzy results
Type "j"
Sleep 400ms

# Jump to result
Enter
Sleep 1s

# Exit
Escape
Sleep 500ms
