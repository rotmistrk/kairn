#!/usr/bin/env rusticle-tk
# widget-gallery.tcl — every widget type on one screen

set win [window create "Widget Gallery"]

# Tab bar at top
set tabs [tabbar create]
tabbar add $tabs "Gallery"
tabbar add $tabs "About" -modified true
window add $win $tabs -side top -height 1

# Tree on the left
set tree [tree create -data "."]
window add $win $tree -side left -width 25

# Status bar at bottom
set status [statusbar create]
statusbar left $status "Widget Gallery — rusticle-tk"
statusbar right $status "Ctrl-Q: Quit"
window add $win $status -side bottom -height 1

# Input line above status
set inp [input create -prompt "Command: "]
window add $win $inp -side bottom -height 1

# Progress bar
set prog [progress create -title "Loading..."]
progress set $prog 0.65 "65%"
window add $win $prog -side bottom -height 1

# Table in the right area
set tbl [table create -columns "Widget Status Lines"]
table add-row $tbl "TextArea OK 5"
table add-row $tbl "ListView OK 3"
table add-row $tbl "TreeView OK 10"
table add-row $tbl "InputLine OK 1"
table add-row $tbl "TabBar OK 1"
table add-row $tbl "StatusBar OK 1"
table add-row $tbl "Table OK 7"
table add-row $tbl "ProgressBar OK 1"
window add $win $tbl -side right -width 35

# Main text area fills the rest
set txt [text create -content "Welcome to the Widget Gallery!\n\nThis demo shows every widget type:\n- TabBar (top)\n- TreeView (left)\n- Table (right)\n- TextArea (center)\n- InputLine (bottom)\n- ProgressBar (bottom)\n- StatusBar (bottom)\n\nPress Ctrl-Q to quit."]
text line-numbers $txt true
window add $win $txt -side fill

# Key bindings
bind Ctrl-Q { app quit }

# Event handlers
tree on-select $tree on_tree_select
input on-submit $inp on_command

proc on_tree_select {path} {
    statusbar left $status "Selected: $path"
}

proc on_command {text} {
    statusbar left $status "Command: $text"
    input clear $inp
}

app run
