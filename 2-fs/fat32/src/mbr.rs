use std::{fmt, io};

use traits::BlockDevice;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct CHS {
    starting_head: u8,
    starting_sector_cylinder: [ u8; 2 ],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct PartitionEntry {
    boot_indicator: u8,
    _CHS_0: CHS,
    pub partition_type: u8,
    _CHS_1: CHS,
    pub relative_sector: u32,
    total_sectors_in_partition: u32,
}

/// The master boot record (MBR).
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct MasterBootRecord {
    bootstrap: [ u8; 436 ],
    disk_id: [ u8; 10 ],
    pub table_entries: [ PartitionEntry; 4 ],
    signature: [ u8; 2 ],
}

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
    pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {

        use std::ptr;
        
        let mut buf = vec![];
        let read_size = match device.read_all_sector( 0, & mut buf ) {
            Ok(x) => { x },
            Err(x) => { return Err( Error::Io(x) ) }
        };

        let b = unsafe{ ptr::read( & buf[0] as * const u8 as * const MasterBootRecord ).clone() };

        if b.signature != [ 0x55, 0xAA ] {
            return Err( Error::BadSignature )
        }

        for i in 0..4 {
            match b.table_entries[i].boot_indicator {
                0x80 | 0x00 => {},
                _ => { return Err( Error::UnknownBootIndicator(i as u8) ) },
            }
        }

        Ok( b )
    }
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct( "MasterBootRecord")
            // .field("bootstrap", & self.bootstrap )
            .field("disk_id", & self.disk_id )
            .field("table_entry_0", & self.table_entries[0] )
            .field("table_entry_1", & self.table_entries[1] )
            .field("table_entry_2", & self.table_entries[2] )
            .field("table_entry_3", & self.table_entries[3] )
            .field("signature", & self.signature )
            .finish()
    }
}
