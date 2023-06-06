Require motus

# Where should we write the GIF?
Output ../static/motus-demo-report.gif

# Set the theme
Set Theme "Dracula+"

# Set the font
Set FontFamily "monolisa"
Set FontSize 15

# Set the window style and size
Set Width 1200
Set Height 800
Set MarginFill "#0D1117"
Set BorderRadius 10
Set WindowBar Colorful

# Set the framerate to 60fps
Set Framerate 60

# Illustrate the base memorable command
Type "motus --analyze memorable"

# Pause for dramatic effect
Sleep 500ms

# Run the command by pressing enter
Enter

# Admire the output for a bit
Sleep 10s