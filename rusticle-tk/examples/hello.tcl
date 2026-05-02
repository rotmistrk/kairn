#!/usr/bin/env rusticle-tk
# hello.tcl — minimal rusticle-tk app

set win [window create "Hello"]
set txt [text create -content "Hello, rusticle-tk!"]
window add $win $txt -side fill
set status [statusbar create]
statusbar left $status "Ready"
window add $win $status -side bottom -height 1
bind Ctrl-Q { app quit }
app run
