#!/usr/bin/env rusticle-tk
# log-viewer.tcl — text viewer with filter input and keybindings
#
# Usage: rusticle-tk log-viewer.tcl [file]

set win [window create "Log Viewer"]

set log [text create -readonly true]
text line-numbers $log true

set filter [input create -prompt "Filter: "]
set status [statusbar create]

# Layout: filter top, status bottom, log fills
window add $win $filter -side top -height 1
window add $win $status -side bottom -height 1
window add $win $log -side fill

# Load file from argv or show help
proc load-file {path} {
    text load $log $path
    statusbar left $status "Loaded: $path"
    statusbar right $status "Ctrl-Q: Quit  F5: Reload"
}

proc show-help {} {
    text set $log "Log Viewer — rusticle-tk demo\n\nUsage: rusticle-tk log-viewer.tcl <file>\n\nKeys:\n  Ctrl-Q  Quit\n  F5      Reload file\n  Tab     Focus filter\n  Enter   Apply filter"
    statusbar left $status "No file loaded"
    statusbar right $status "Ctrl-Q: Quit"
}

# Filter handler — search within text
input on-submit $filter do-filter
proc do-filter {text} {
    statusbar left $status "Filter: $text"
}

# Key bindings
bind Ctrl-Q { app quit }
bind F5 { load-file $current_file }
bind Tab { input focus $filter }

# Load initial file
set current_file ""
if {[llength $argv] > 0} {
    set current_file [lindex $argv 0]
    load-file $current_file
} else {
    show-help
}

app run
