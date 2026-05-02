#!/usr/bin/env rusticle-tk
# config-editor.tcl — tree of sections + table of key/value pairs

set win [window create "Config Editor"]

# Left: section tree
set sections [list create]
set tbl [table create -columns "Key Value"]
set inp [input create -prompt "Value: "]
set status [statusbar create]

# Layout: sections left, input bottom, status bottom, table fills
window add $win $sections -side left -width 20
window add $win $status -side bottom -height 1
window add $win $inp -side bottom -height 1
window add $win $tbl -side fill

# Sample config data
set section_names "General Editor Terminal Keys"
list set-items $sections $section_names

proc show-section {name} {
    table clear $tbl
    if {$name == "General"} {
        table add-row $tbl "theme dark"
        table add-row $tbl "font-size 14"
        table add-row $tbl "auto-save true"
    }
    if {$name == "Editor"} {
        table add-row $tbl "tab-width 4"
        table add-row $tbl "line-numbers true"
        table add-row $tbl "word-wrap false"
    }
    if {$name == "Terminal"} {
        table add-row $tbl "shell /bin/bash"
        table add-row $tbl "scrollback 10000"
    }
    if {$name == "Keys"} {
        table add-row $tbl "quit Ctrl-Q"
        table add-row $tbl "save Ctrl-S"
        table add-row $tbl "search Ctrl-F"
    }
    statusbar left $status "Section: $name"
}

proc edit-value {text} {
    set row [table selected $tbl]
    statusbar left $status "Set row $row to: $text"
    input clear $inp
}

list on-select $sections show-section
input on-submit $inp edit-value
bind Ctrl-Q { app quit }

statusbar left $status "Select a section"
statusbar right $status "Ctrl-Q: Quit"

# Show first section
show-section General

app run
