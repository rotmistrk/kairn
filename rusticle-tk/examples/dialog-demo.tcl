#!/usr/bin/env rusticle-tk
# dialog-demo.tcl — dialog replacement

set answer [dialog confirm "Proceed with installation?"]
if {$answer} { puts "yes" } else { puts "no" }
