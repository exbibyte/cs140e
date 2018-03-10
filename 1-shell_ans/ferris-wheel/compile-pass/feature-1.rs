#![feature(i128_type,i128)]
// FIXME: Make me compile! Diff budget: 2 lines.
// Do not modify this definition.
use std::u128;
enum Duration {
    MicroSeconds(u128),
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}

fn main() { }
