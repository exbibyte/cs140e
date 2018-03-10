// FIXME: Make me compile! Diff budget: 2 lines.

// Do not modify the inner type &'a T.
struct RefWrapper<'a, T>(&'a T) where T: 'a;

// Do not modify the inner type &'b RefWrapper<'a, T>.
struct RefWrapperWrapper<'a, 'b, T>(&'b RefWrapper<'a, T>) where T: 'a, 'a: 'b;

pub fn main() { }
