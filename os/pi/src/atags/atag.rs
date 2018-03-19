use atags::raw;

extern crate std;

pub use atags::raw::{Core, Mem};

/// An ATAG.
#[derive(Debug, Copy, Clone)]
pub enum Atag {
    Core(raw::Core),
    Mem(raw::Mem),
    Cmd(&'static str),
    Unknown(u32),
    None
}

impl Atag {
    /// Returns `Some` if this is a `Core` ATAG. Otherwise returns `None`.
    pub fn core(self) -> Option<Core> {
        match self {
            Atag::Core(x) => Some( x ),
            _ => None,
        }
    }

    /// Returns `Some` if this is a `Mem` ATAG. Otherwise returns `None`.
    pub fn mem(self) -> Option<Mem> {
        match self {
            Atag::Mem(x) => Some( x ),
            _ => None,
        }
    }

    /// Returns `Some` with the command line string if this is a `Cmd` ATAG.
    /// Otherwise returns `None`.
    pub fn cmd(self) -> Option<&'static str> {
        match self {
            Atag::Cmd(x) => Some( x ),
            _ => None,
        }
    }
}

// FIXME: Implement `From<raw::Core>`, `From<raw::Mem>`, and `From<&raw::Cmd>`
// for `Atag`. These implementations should be used by the `From<&raw::Atag> for
// Atag` implementation below.

impl<'a> From<&'a raw::Atag> for Atag {
    fn from(atag: &raw::Atag) -> Atag {

        use self::std::str;
        use self::std::slice;
        
        // FIXME: Complete the implementation below.

        unsafe {
            match (atag.tag, &atag.kind) {
                (raw::Atag::CORE, &raw::Kind { core }) => Atag::Core( core ),
                (raw::Atag::MEM, &raw::Kind { mem }) => Atag::Mem( mem ),
                (raw::Atag::CMDLINE, &raw::Kind { ref cmd }) => {
                    //search for end of the string
                    let start = cmd.cmd as * mut u8;
                    let mut c = cmd.cmd as * mut u8;
                    while *c != '\0' as u8 {
                        c = c.add(1);
                    }
                    let len = c as usize - start as usize;
                    Atag::Cmd(
                        //cast to & str
                        str::from_utf8( slice::from_raw_parts( start, len ) ).unwrap_or_default()
                    )
                },
                (raw::Atag::NONE, _) => Atag::None,
                (id, _) => Atag::Unknown(id)
            }
        }
    }
}
