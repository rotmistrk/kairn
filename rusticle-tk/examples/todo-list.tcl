#!/usr/bin/env rusticle-tk
# todo-list.tcl — add/remove items with input + list

set win [window create "TODO List"]

set items {}
set todo [list create]
set inp [input create -prompt "New task: "]
set status [statusbar create]

window add $win $inp -side top -height 1
window add $win $status -side bottom -height 1
window add $win $todo -side fill

statusbar left $status "Enter: add  d: delete  Ctrl-Q: quit"
statusbar right $status "0 items"

proc refresh {} {
    list set-items $todo $items
    set n [llength $items]
    statusbar right $status "$n items"
}

proc add-task {text} {
    if {$text == ""} { return }
    lappend items $text
    refresh
    input clear $inp
}

proc delete-selected {} {
    set idx [list index $todo]
    set items [lreplace $items $idx $idx]
    refresh
}

input on-submit $inp add-task
bind d { delete-selected }
bind Ctrl-Q { app quit }

# Seed with sample items
lappend items "Buy groceries"
lappend items "Write rusticle-tk demos"
lappend items "Review pull request"
refresh

app run
