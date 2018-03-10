// FIXME: Make me pass! Diff budget: 25 lines.

use std::cmp::PartialEq;

#[derive(Debug)]
pub enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16)
}

impl PartialEq for Duration {
    fn eq( & self, o: & Self ) -> bool {
        let a = match *self {
            Duration::MilliSeconds(ref x) => {*x as u64},
            Duration::Seconds(ref x) => {*x as u64 * 1000},
            Duration::Minutes(ref x) => {*x as u64 * 1000*60},
        };
        let b = match *o {
            Duration::MilliSeconds(ref x) => {*x as u64},
            Duration::Seconds(ref x) => {*x as u64 * 1000},
            Duration::Minutes(ref x) => {*x as u64 * 1000*60},
        };
        a == b
    }
}

fn main() {
    assert_eq!(Duration::Seconds(120), Duration::Minutes(2));
    assert_eq!(Duration::Seconds(420), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(420000), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(43000), Duration::Seconds(43));
}
