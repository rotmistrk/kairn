#!/usr/bin/env rusticle
# Rusticle pipes and functional programming

puts "=== Pipe operator ==="
set result ["  Hello World  " | string trim | string toupper]
puts "piped: $result"

puts "\n=== Lambda and lmap ==="
set nums [range 1 6]
puts "nums: $nums"

set doubled [lmap $nums {x { expr {$x * 2} }}]
puts "doubled: $doubled"

set squares [lmap $nums {x { expr {$x * $x} }}]
puts "squares: $squares"

puts "\n=== lfilter ==="
set evens [lfilter [range 1 20] {x { expr {$x % 2 == 0} }}]
puts "evens 1..19: $evens"

puts "\n=== lreduce ==="
set sum [lreduce [range 1 11] 0 {acc x { expr {$acc + $x} }}]
puts "sum 1..10: $sum"

set product [lreduce [range 1 6] 1 {acc x { expr {$acc * $x} }}]
puts "product 1..5 (5!): $product"

puts "\n=== Composing pipelines ==="
# Generate numbers, filter, transform, reduce
set result [lreduce [lmap [lfilter [range 1 20] {x { expr {$x % 3 == 0} }}] {x { expr {$x * $x} }}] 0 {acc x { expr {$acc + $x} }}]
puts "sum of squares of multiples of 3 in 1..19: $result"

puts "\n=== String processing ==="
set words %[ "hello", "WORLD", "Rusticle", "tcl" ]
set upper [lmap $words {w { string toupper $w }}]
puts "uppercased: $upper"

set lengths [lmap $words {w { string length $w }}]
puts "lengths: $lengths"

puts "\n=== Dict processing ==="
set people %[
    %{ name: "Alice", age: 30 },
    %{ name: "Bob", age: 25 },
    %{ name: "Carol", age: 35 }
]

puts "People:"
foreach person $people {
    puts "  $person(name) is $person(age) years old"
}

puts "\n=== Done ==="
