#!/usr/bin/env rusticle-tk
# file-browser.tcl — three-panel file browser

set win [window create "Files"]
set tree [tree create -data "."]
set txt [text create -readonly true]
set status [statusbar create]

window add $win $tree -side left -width 25
window add $win $status -side bottom -height 1
window add $win $txt -side fill

proc on-file {path} {
    text load $txt $path
    statusbar left $status $path
}

tree on-select $tree on-file
bind Ctrl-Q { app quit }
app run
