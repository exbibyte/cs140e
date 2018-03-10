// FIXME: Make me pass! Diff budget: 10 lines.
// Do not `use` any items.

// Do not change the following two lines.
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy)]
struct IntWrapper(isize);

fn max<T,S>( a: T, b: S ) -> IntWrapper where T: PartialOrd + Into<IntWrapper>,
                                              S: PartialOrd + Into<IntWrapper> {
    let w_a : IntWrapper = a.into();
    let w_b : IntWrapper = b.into();
    if w_a.0 < w_b.0 {
        w_b
    } else {
        w_a
    }
}
impl From<i32> for IntWrapper {
    fn from( i: i32 ) -> IntWrapper {
        IntWrapper( i as isize )
    }
}
impl From<usize> for IntWrapper {
    fn from( i: usize ) -> IntWrapper {
        IntWrapper( i as isize )
    }
}
impl From<u8> for IntWrapper {
    fn from( i: u8 ) -> IntWrapper {
        IntWrapper( i as isize )
    }
}
impl<'a> From<&'a i32> for IntWrapper {
    fn from(v: &'a i32) -> IntWrapper {
        IntWrapper( v.clone() as isize )
    }
}
impl<T> PartialEq<T> for IntWrapper where for<'a> IntWrapper: From< & 'a T > {
    fn eq( & self, o: & T ) -> bool {
        let other = IntWrapper::from(o);
        self.0 == other.0
    }
}

pub fn main() {
    assert_eq!(max(1usize, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(1u8, 3), 3);
    assert_eq!(max(IntWrapper(120), IntWrapper(248)), IntWrapper(248));
}
