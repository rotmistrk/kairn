#!/usr/bin/env rusticle-tk
# dashboard.tcl — progress bars + statusbar + repeating timers

set win [window create "System Dashboard"]

# Progress bars for different metrics
set cpu [progress create -title "CPU"]
set mem [progress create -title "Memory"]
set disk [progress create -title "Disk"]
set net [progress create -title "Network"]

progress set $cpu 0.35 "CPU: 35%"
progress set $mem 0.62 "Memory: 62%"
progress set $disk 0.81 "Disk: 81%"
progress set $net 0.12 "Network: 12%"

# Log area
set log [text create -content "Dashboard started.\nWaiting for updates..."]
text line-numbers $log false

# Status bar
set status [statusbar create]
statusbar left $status "Dashboard — rusticle-tk"
statusbar right $status "Ctrl-Q: Quit"

# Layout: progress bars top, status bottom, log fills
window add $win $cpu -side top -height 1
window add $win $mem -side top -height 1
window add $win $disk -side top -height 1
window add $win $net -side top -height 1
window add $win $status -side bottom -height 1
window add $win $log -side fill

set tick_count 0

proc update-dashboard {} {
    incr tick_count
    set c [expr {($tick_count * 7) % 100}]
    set m [expr {60 + ($tick_count * 3) % 30}]
    progress set $cpu [expr {$c / 100.0}] "CPU: $c%"
    progress set $mem [expr {$m / 100.0}] "Memory: $m%"
    statusbar right $status "Tick: $tick_count  Ctrl-Q: Quit"
}

# Update every 2 seconds
after 2000 -repeat { update-dashboard }

bind Ctrl-Q { app quit }

app run
