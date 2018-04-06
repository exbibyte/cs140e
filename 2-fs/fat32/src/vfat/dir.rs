use std::ffi::OsStr;
use std::char::decode_utf16;
use std::borrow::Cow;
use std::io;

use traits;
use util::VecExt;
use vfat::{VFat, Shared, File, Cluster, Entry};
use vfat::{Metadata, Attributes, Timestamp, Time, Date};

#[derive(Debug)]
pub struct Dir {
    // FIXME: Fill me in.
    pub first_cluster: Cluster,
    pub vfat: Shared<VFat>,
    pub meta: Metadata,
    pub short_file_name: String,
    pub lfn: String,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    // FIXME: Fill me in.
    file_name: [ u8; 8 ],
    file_extension: [ u8; 3 ],
    pub meta: Metadata,
    size_file: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    // FIXME: Fill me in.
    sequence_num: u8,
    name_chars_0: [ u8; 10 ],
    attrib: Attributes,
    entry_type: u8,
    checksum_dos_file_name: u8,
    name_chars_1: [ u8; 12 ],
    signature: u16,
    name_chars_2: [ u8; 4 ],
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    // FIXME: Fill me in.
    _data_0: [ u8; 11 ],
    attrib: Attributes,
    _data_1: [ u8; 20 ],
}

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl Dir {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry> {
        use std::str;
        use traits::Dir;

        let name_str = match name.as_ref().to_str() {
            Some(x) => { x },
            _ => { return Err( io::Error::new( io::ErrorKind::InvalidInput, "input name is invalid" ) ) },
        };

        for x in self.entries()? {
            let matched = match x {
                Entry::File(ref f) => {
                    if f.short_file_name.as_str().eq_ignore_ascii_case( name_str ) {
                        true
                    } else if f.lfn.as_str().eq_ignore_ascii_case( name_str ) {
                        true
                    } else {
                        false
                    }
                },
                Entry::Dir(ref d) => {
                    if d.short_file_name.as_str().eq_ignore_ascii_case( name_str ) {
                        true
                    } else if d.lfn.as_str().eq_ignore_ascii_case( name_str ) {
                        true
                    } else {
                        false
                    }
                },
            };
            if matched {
                return Ok(x)
            }
        }

        Err( io::Error::new( io::ErrorKind::NotFound, "non-existent entry") )
    }
    pub fn new_dir( vfat: & Shared<VFat> ) -> Self {
        Self {
            first_cluster: vfat.borrow().root_dir_cluster,
            vfat: vfat.clone(),
            meta: Metadata::default(),
            short_file_name: String::new(),
            lfn: String::new(),
        }
    }
}

// FIXME: Implement `trait::Dir` for `Dir`.
impl traits::Dir for Dir {
    /// The type of entry stored in this directory.
    type Entry = Entry;

    /// An type that is an iterator over the entries in this directory.
    type Iter = VFatEntryIterator;

    /// Returns an interator over the entries in this directory.
    fn entries(&self) -> io::Result<Self::Iter> {
        let mut d = Vec::new();
        let mut fs = self.vfat.borrow_mut();
        let r = fs.read_chain( self.first_cluster, & mut d )?;
        // println!("read bytes: {}, data stored: {}", r, d.len() );
        assert_eq!( r, d.len() );
        let bytes_per_cluster = fs.bytes_per_sector as usize * fs.sectors_per_cluster as usize;
        Ok(
            VFatEntryIterator {
                idx: 0,
                vfat: self.vfat.clone(),
                data: d,
                bytes_per_cluster: bytes_per_cluster,
            }
        )
    }
}

pub struct VFatEntryIterator {
    idx: usize,
    vfat: Shared<VFat>,
    data: Vec<u8>,
    bytes_per_cluster: usize,
}

//todo
impl Iterator for VFatEntryIterator {
    type Item = Entry;

    fn next( & mut self ) -> Option< Self::Item > {
        use std::slice;
        use std::str;
        use std::mem;

        let mut lfn_buf : Vec<u8> = vec![];

        let mut offset = self.idx * mem::size_of::< VFatDirEntry >();
        let num_entries = ( self.data.len() - offset ) / mem::size_of::< VFatDirEntry >();
        assert!( num_entries > 0 );

        loop {

            offset = self.idx * mem::size_of::< VFatDirEntry >();

            // println!("looping... at offset: {}. data len: {}. idx: {}", offset, self.data.len(), self.idx );
            if offset >= self.data.len() {
                // println!("returning none");                
                return None
            } else {

                // let entries = unsafe { slice::from_raw_parts( self.data[ offset.. ].as_ptr() as * const VFatDirEntry, num_entries ) };
                // let entries = unsafe { slice::from_raw_parts( ( self.data.as_ptr() as * const VFatDirEntry ).offset( self.idx as isize ), 1 ) };

                let entries = unsafe { slice::from_raw_parts( ( self.data.as_ptr() as * const VFatDirEntry ).offset( self.idx as isize ), 1 ) };

                //take care of different cases of the VFatDirEntry
                let unknown_entry = unsafe { entries[0].unknown };

                match unknown_entry._data_0[0] { //check the first byte
                    0x00 => { //previous entry was the last entry
                        return None
                    }, 
                    0xE5 => { //unused/deleted entry
                        self.idx += 1;
                        continue;
                    }, 
                    _ => {},
                }

                // println!("idx: {}", self.idx );
                self.idx += 1;
                
                let attrib = & unknown_entry.attrib; //peek at attrib field of the unknown entry
                if attrib.0 == 0x0F {
                    let lfn_entry = unsafe { entries[0].long_filename };
                    match lfn_entry.sequence_num & 0b11111 {
                        entry_num @ 0x01...0x1F => {
                            let index = entry_num as usize - 1;
                            let start = index * ( 10 + 12 + 4 );
                            let end = entry_num as usize * ( 10 + 12 + 4 );
                            if end > lfn_buf.len() {
                                lfn_buf.resize( end, 0 );
                            }
                            lfn_buf[ start..
                                     start + 10 ].copy_from_slice( &lfn_entry.name_chars_0[..] );
                            lfn_buf[ start + 10..
                                     start + 10 + 12 ].copy_from_slice( &lfn_entry.name_chars_1[..] );
                            lfn_buf[ start + 10 + 12..
                                     start + 10 + 12 + 4 ].copy_from_slice( &lfn_entry.name_chars_2[..] );
                        },
                        _ => {},
                    }
                } else {
                    let regular_entry = unsafe { entries[0].regular };

                    // let mut temp = regular_entry.file_name.clone();
                    // if temp[0] == 0x05 {
                    //     temp[0] = 0xE5;
                    // }
                    // let short_name = str::from_utf8( &temp ).unwrap().trim_right();
                    let short_file_name = regular_entry.file_name.clone();
                    let short_file_ext = regular_entry.file_extension.clone();
                    // let short_name = str::from_utf8( &regular_entry.file_name ).unwrap().trim_right();
                    // let short_ext = str::from_utf8( &regular_entry.file_extension ).unwrap().trim_right();
                    let short_name = str::from_utf8( &short_file_name ).unwrap().trim_right();
                    let short_ext = str::from_utf8( &short_file_ext ).unwrap().trim_right();
                    
                    let short_file_name = if short_ext.len() > 0 {
                        let mut s = String::from( short_name );
                        s.push_str( &"." );
                        s.push_str( short_ext );
                        s
                    } else {
                        String::from( short_name )
                    };

                    use std::mem;
                    use std::slice;
                    
                    assert!( lfn_buf.len() % 2 == 0 );

                    let lfn = if lfn_buf.len() > 0 {
                        // println!("size of lfn_buf: {}", lfn_buf.len() );
                        let lfn_utf16 = unsafe { slice::from_raw_parts( mem::transmute::< * const u8, * const u16 >(  &lfn_buf[0] as * const u8 ), lfn_buf.len() / 2 )};
                        //find early termination character
                        let lfn_end = lfn_utf16.iter().position( |x| *x == 0x00 || *x == 0xFF );
                        if let Some(x) = lfn_end {
                            String::from_utf16( &lfn_utf16[..x] ).unwrap()
                        } else {
                            String::from_utf16( &lfn_utf16 ).unwrap()
                        }
                    } else {
                        String::new()
                    };

                    use traits::Metadata;

                    if regular_entry.meta.is_directory() {
                        // println!("returning directory");
                        return Some( Entry::Dir(
                            Dir {
                                first_cluster: Cluster::from( regular_entry.meta.first_cluster_num() ),
                                vfat: self.vfat.clone(),
                                short_file_name: short_file_name,
                                lfn: lfn,
                                meta: regular_entry.meta.clone(),
                            }
                        ) )
                    } else {
                        // println!("returning file");
                        return Some( Entry::File(
                            File {
                                first_cluster: Cluster::from( regular_entry.meta.first_cluster_num() ),
                                vfat: self.vfat.clone(),
                                short_file_name: short_file_name,
                                lfn: lfn,
                                meta: regular_entry.meta.clone(),
                                size: regular_entry.size_file as usize,
                                bytes_per_cluster: self.bytes_per_cluster,
                                current_offset: 0,
                                current_cluster: Cluster::from( regular_entry.meta.first_cluster_num() ),
                            }
                        ) )
                    }
                }
            }
        }
    }
}

