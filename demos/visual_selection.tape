# VHS tape for Visual Selection Mode demo

Output ../docs/assets/visual_selection.gif

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
Type "/home/holger/bin/dtree -v README.md"
Enter
Sleep 1.5s
Show

# Scroll down a bit to show content
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 200ms
Type "j"
Sleep 800ms

# Enter visual selection mode
Type "V"
Sleep 1s

# Select lines by moving down
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Pause to show selection
Sleep 1s

# Copy selection
Type "y"
Sleep 1s

# Show that we're back to normal mode
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms

# Exit
Escape
Sleep 500ms
