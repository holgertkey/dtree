# VHS tape template for dtree demos
# Copy this file and modify for your own demos

Output ../docs/assets/my_demo.gif

# Display settings
Set FontSize 16
Set Width 1400
Set Height 700
Set Theme "Catppuccin Mocha"
Set TypingSpeed 100ms
Set PlaybackSpeed 1.0

# CRITICAL: Set PATH to include system commands
# VHS runs in isolated shell, so we need full paths
Env PATH "$HOME/bin:/snap/bin:/usr/local/bin:/usr/bin:/bin"

# Hide startup commands - don't show cd and launch
Hide
Type "cd /home/holger/github.com/holgertkey/dtree"
Enter
# IMPORTANT: Use ABSOLUTE PATH to dtree
Type "/home/holger/bin/dtree"
Enter
Sleep 1.5s
Show

# Your demo actions here
# Example: Navigate tree
Type "j"
Sleep 300ms
Type "j"
Sleep 300ms
Type "l"
Sleep 500ms

# Exit
Type "q"
Sleep 500ms
