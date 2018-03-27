use std::fmt;
use vfat::*;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    /// The FAT entry corresponds to an unused (free) cluster.
    Free,
    /// The FAT entry/cluster is reserved.
    Reserved,
    /// The FAT entry corresponds to a valid data cluster. The next cluster in
    /// the chain is `Cluster`.
    Data(Cluster),
    /// The FAT entry corresponds to a bad (disk failed) cluster.
    Bad,
    /// The FAT entry corresponds to a valid data cluster. The corresponding
    /// cluster is the last in its chain.
    Eoc(u32)
}

#[repr(C, packed)]
pub struct FatEntry(pub u32);

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub fn status(&self) -> Status {
        let val = !( 0xF << 28 ) & self.0;
        match val {
            0 => { Status::Free },
            1 => { Status::Reserved },
            v @ 2...0x0FFFFFEF => { Status::Data( Cluster::from( v ) ) },
            0x0FFFFFF0...0x0FFFFFF5 => { Status::Reserved },
            0x0FFFFFF6 => { Status::Reserved },
            0x0FFFFFF7 => { Status::Bad },
            0x0FFFFFF8...0x0FFFFFFF => { Status::Eoc( self.0 ) },
            _ => { unreachable!(); }
        }
    }
}

impl fmt::Debug for FatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FatEntry")
            .field("value", &self.0)
            .field("status", &self.status())
            .finish()
    }
}
