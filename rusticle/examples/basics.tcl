#!/usr/bin/env rusticle
# Rusticle basics: variables, types, control flow

puts "=== Variables ==="
set name "rusticle"
set version 1
set pi 3.14159
set active true
puts "name=$name version=$version pi=$pi active=$active"

puts "\n=== Modern assignment ==="
set x = 42
set greeting = "hello world"
puts "x=$x greeting=$greeting"

puts "\n=== Structured literals ==="
set config %{
    name: "kairn",
    version: 1,
    features: %[ "editor", "lsp", "git" ]
}
puts "config name: $config(name)"
puts "features: $config(features)"
puts "first feature: $config(features)(0)"

puts "\n=== List operations ==="
set nums %[ 10, 20, 30, 40, 50 ]
puts "nums: $nums"
puts "length: $nums.len"
puts "third: $nums(2)"
puts "slice 1..3: [lrange $nums 1 3]"

puts "\n=== Destructuring ==="
set a, b, c = [list 1 2 3]
puts "a=$a b=$b c=$c"

puts "\n=== Control flow ==="
foreach item %[ "alpha", "beta", "gamma" ] {
    puts "  item: $item"
}

set sum 0
for {set i 1} {$i <= 10} {incr i} {
    set sum [expr {$sum + $i}]
}
puts "sum 1..10 = $sum"

puts "\n=== Procedures ==="
proc factorial {n} {
    set result 1
    for {set i 2} {$i <= $n} {incr i} {
        set result [expr {$result * $i}]
    }
    return $result
}
puts "5! = [factorial 5]"
puts "10! = [factorial 10]"

puts "\n=== Pattern matching ==="
proc describe {val} {
    match $val {
        "ok"    { return "success" }
        "err"   { return "failure" }
        _       { return "unknown: $val" }
    }
}
puts "ok -> [describe ok]"
puts "err -> [describe err]"
puts "foo -> [describe foo]"

puts "\n=== Done ==="
