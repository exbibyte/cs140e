mod raw;
mod atag;

pub use self::atag::*;

/// The address at which the firmware loads the ATAGS.
const ATAG_BASE: usize = 0x100;

/// An iterator over the ATAGS on this system.
pub struct Atags {
    ptr: &'static raw::Atag,
}

impl Atags {
    /// Returns an instance of `Atags`, an iterator over ATAGS on this system.
    pub fn get() -> Atags {
        Atags {
            ptr: unsafe { &*(ATAG_BASE as *const raw::Atag) }
        }
    }
    pub fn current( & self ) -> Option<Atag> {
        match self.ptr.current() {
            Some( x ) => Some( Atag::from( x ) ),
            _ => None
        }
    }
}

impl Iterator for Atags {
    type Item = Atag;

    fn next(&mut self) -> Option<Atag> {
        //convert from raw type to wrapped type
        match self.ptr.next() {
            Some( x ) => {
                self.ptr = x;
                Some( Atag::from( x ) )
            },
            _ => None
        }
    }
}
