#![feature(const_fn)]
// FIXME: Make me compile! Diff budget: 3 lines.
const VAR: i32 = add(34, 10);

const fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() { }
