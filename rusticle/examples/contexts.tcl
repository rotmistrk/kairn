#!/usr/bin/env rusticle
# Rusticle contexts, typed declarations, and error handling

puts "=== Context blocks ==="
context config {
    declare mode : enum {vi emacs classic}
    declare tab_width : int
    declare auto_save : bool

    set mode vi
    set tab_width 4
    set auto_save true
}

puts "Mode: $config::mode"
puts "Tab width: $config::tab_width"
puts "Auto save: $config::auto_save"

# Change a value
set config::mode emacs
puts "Changed mode to: $config::mode"

puts "\n=== Try/catch ==="
proc safe_divide {a b} {
    if {$b == 0} {
        error "division by zero"
    }
    return [expr {$a / $b}]
}

try {
    puts "10 / 3 = [safe_divide 10 3]"
    puts "10 / 0 = [safe_divide 10 0]"
} on error {msg} {
    puts "Caught error: $msg"
} finally {
    puts "Cleanup done"
}

puts "\n=== Optional chaining ==="
set data %{
    user: %{
        name: "alice",
        prefs: %{ theme: "dark" }
    }
}

puts "theme: [dict get [dict get [dict get $data user] prefs] theme]"

puts "\n=== Heredoc ==="
set name "rusticle"
set greeting <<END
    Hello from $name!
    This is a multi-line string.
END
puts $greeting

puts "\n=== Done ==="
