#!/usr/bin/env rusticle
# Rusticle data structures: dicts, lists, nested data

puts "=== Dict as a record ==="
proc make_person {name age city} {
    return [dict create name $name age $age city $city]
}

set alice [make_person "Alice" 30 "NYC"]
set bob [make_person "Bob" 25 "SF"]
set carol [make_person "Carol" 35 "London"]

set people [list $alice $bob $carol]

puts "People:"
foreach p $people {
    puts "  $p(name), age $p(age), from $p(city)"
}

puts "\n=== Sorting ==="
set names [lmap $people {p { dict get $p name }}]
puts "Names: [lsort $names]"

puts "\n=== Building a lookup table ==="
set by_name [dict create]
foreach p $people {
    dict set by_name [dict get $p name] $p
}
puts "Lookup Alice: [dict get [dict get $by_name Alice] city]"

puts "\n=== Stack (list as stack) ==="
set stack [list]
lappend stack "first"
lappend stack "second"
lappend stack "third"
puts "Stack: $stack"
set last_idx [expr {[llength $stack] - 1}]
puts "Top: [lindex $stack $last_idx]"

puts "\n=== String processing ==="
set fields [split "Alice,30,NYC" ","]
puts "CSV fields: [join $fields { | }]"

set words [split "hello world foo bar" " "]
puts "Words: $words"
puts "Word count: [llength $words]"

puts "\n=== FizzBuzz (using lmap) ==="
# Note: expr inside foreach/lmap works via the functional style
set fizzbuzz [lmap [range 1 21] {n {
    set mod15 [expr {$n % 15}]
    set mod3 [expr {$n % 3}]
    set mod5 [expr {$n % 5}]
    if {$mod15 == 0} {
        set _ FizzBuzz
    } elseif {$mod3 == 0} {
        set _ Fizz
    } elseif {$mod5 == 0} {
        set _ Buzz
    } else {
        set _ $n
    }
}}]
puts [join $fizzbuzz ", "]

puts "\n=== Fibonacci (top-level) ==="
set a 0
set b 1
set fibs [list $a $b]
for {set i 2} {$i < 15} {incr i} {
    set temp [expr {$a + $b}]
    set a $b
    set b $temp
    lappend fibs $b
}
puts "Fibonacci: [join $fibs { }]"

puts "\n=== Done ==="
