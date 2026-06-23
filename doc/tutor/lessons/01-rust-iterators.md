# Lesson: Rust Iterators

Learn iterator combinators by transforming collections.

---

STEP: Create a file `src/main.rs` with a main function that prints "hello"

CHECK: file_exists src/main.rs
CHECK: file_contains src/main.rs fn main
CHECK: file_contains src/main.rs println

HINT: Use `fn main() { println!("hello"); }`

---

STEP: Create a vector of numbers 1..=10 and use `.filter()` to keep only even numbers. Print the result.

CHECK: file_contains src/main.rs filter
CHECK: file_contains src/main.rs % 2

HINT: Try `let evens: Vec<_> = (1..=10).filter(|x| x % 2 == 0).collect();`

---

STEP: Chain `.map()` after your filter to square each even number, then collect into a Vec and print it.

CHECK: file_contains src/main.rs .map(
CHECK: file_contains src/main.rs .filter(

HINT: `.filter(|x| x % 2 == 0).map(|x| x * x).collect::<Vec<_>>()`

---

STEP: Use `.fold()` to sum all the squared even numbers into a single value. Print the sum.

CHECK: file_contains src/main.rs .fold(

HINT: `.fold(0, |acc, x| acc + x)` — the first argument is the initial accumulator value.

---

STEP: Refactor: extract the iterator chain into a function `fn sum_of_even_squares(n: u32) -> u32` and call it from main.

CHECK: file_contains src/main.rs fn sum_of_even_squares
CHECK: file_contains src/main.rs sum_of_even_squares(

HINT: Move the chain into a function that takes the upper bound and returns the fold result.
