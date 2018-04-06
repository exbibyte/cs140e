use std::io;
use std::path::Path;
use std::mem::size_of;
use std::cmp::min;

use util::SliceExt;
use mbr::MasterBootRecord;
use vfat::{Shared, Cluster, File, Dir, Entry, FatEntry, Error, Status};
use vfat::{BiosParameterBlock, CachedDevice, Partition};
use traits::{FileSystem, BlockDevice};

#[derive(Debug)]
pub struct VFat {
    device: CachedDevice,
    pub bytes_per_sector: u16,
    pub sectors_per_cluster: u8,
    sectors_per_fat: u32,
    fat_start_sector: u64,
    data_start_sector: u64,
    pub root_dir_cluster: Cluster,
}

impl VFat {
    pub fn from<T>(mut device: T) -> Result<Shared<VFat>, Error>
        where T: BlockDevice + 'static
    {
        let mbr = MasterBootRecord::from( & mut device )?;

        //find the first FAT
        for i in 0..4 {
            match mbr.table_entries[i].partition_type {
                0x0B | 0x0C => {
                    
                    let bpb = BiosParameterBlock::from( & mut device, mbr.table_entries[i].relative_sector as _ )?;

                    if bpb.num_bytes_per_sector == 0 {
                        return Err( Error::Io( io::Error::new( io::ErrorKind::Other, "logic sector size invalid" ) ) )
                    }
                    
                    let sectors = if bpb.num_sectors_per_fat == 0 {
                        bpb.sectors_per_fat as u32
                    } else {
                        bpb.num_sectors_per_fat as u32
                    };

                    let p = Partition { start: mbr.table_entries[i].relative_sector as u64,
                                        sector_size: bpb.num_bytes_per_sector as u64 };

                    let device = CachedDevice::new( device, p );

                    let vfat = VFat {
                        device: device,
                        bytes_per_sector: bpb.num_bytes_per_sector,
                        sectors_per_cluster: bpb.num_sectors_per_cluster,
                        sectors_per_fat: sectors,
                        fat_start_sector: mbr.table_entries[i].relative_sector as u64 + bpb.num_reserved_sectors as u64,
                        data_start_sector: mbr.table_entries[i].relative_sector as u64 + bpb.num_reserved_sectors as u64 + sectors as u64 * bpb.num_file_allocation_tables as u64,
                        root_dir_cluster: Cluster::from( bpb.cluster_num_root_dir ),
                    };
                    
                    return Ok( Shared::new( vfat ) )
                },
                _ => {},
            }
        }
        
        Err( Error::Io( io::Error::new( io::ErrorKind::NotFound, "vfat not found") ) )
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.

    pub fn read_cluster(
        &mut self,
        cluster: Cluster,
        offset: usize, //offset in bytes
        buf: &mut [u8]
    ) -> io::Result<usize> {

        let sector_size = self.device.sector_size() as usize;
        let len_bytes_cluster = sector_size * self.sectors_per_cluster as usize;
        
        let mut sector = self.data_start_sector as usize +
            (cluster.cluster_num() as usize - 2usize ) * self.sectors_per_cluster as usize + //data clusters starts at 2
            offset as usize / self.bytes_per_sector as usize;

        //amount of data to read
        let len_to_read = if buf.len() < len_bytes_cluster - offset {
            buf.len()
        } else {
            len_bytes_cluster - offset
        };

        //starting offset of the read
        let mut bytes_remain = offset % self.bytes_per_sector as usize;

        let mut read = 0;
        loop {

            if read >= len_to_read {
                break;
            }
            
            let sector_data : &[u8] = self.device.get( sector as u64 )?;
            
            let device_read = sector_data.len();

            //amount of data to be read from the current sector
            let len_copy = if len_to_read - read < sector_size - bytes_remain {
                len_to_read - read
            } else {
                sector_size - bytes_remain
            };
            
            buf[ read.. read + len_copy ].copy_from_slice( &sector_data[ bytes_remain.. bytes_remain + len_copy ] );

            bytes_remain = 0; //zero the offset after first read
            sector += 1;
            read += len_copy;
        }

        Ok( read )
    }

    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.

    pub fn read_chain(
        &mut self,
        start: Cluster,
        buf: &mut Vec<u8>
    ) -> io::Result<usize> {

        let bytes_per_cluster = self.bytes_per_sector as usize * self.sectors_per_cluster as usize;
        let mut read = 0;
        let mut current = start;
        
        buf.clear();

        let mut cycle_detect = None;
        
        //check status of current fat entry
        match self.fat_entry( current )?.status() {
            Status::Data(x) => {
                cycle_detect = Some( x );
            },
            Status::Eoc(x) => {},
            _ => { return Err( io::Error::new( io::ErrorKind::InvalidData,
                                               "Invalid cluster chain" ) )
            },
        }
        
        loop {

            // println!("read chain loop");
            
            if let Some(x) = cycle_detect {
                if current.cluster_num() == x.cluster_num() {
                    return Err( io::Error::new( io::ErrorKind::InvalidData,
                                                "FAT cluster chain has a cycle" ) )
                }
            }

            buf.resize( read + bytes_per_cluster, 0 );

            let offset = 0;
            let bytes_read = self.read_cluster( current, offset, & mut buf[read..] )?;
            read += bytes_read;

            //advance to next cluster
            match self.fat_entry( current )?.status() {
                Status::Data( x ) => {
                    current = x;
                },
                Status::Eoc( x ) => {
                    break; //done
                },
                _ => { return Err( io::Error::new( io::ErrorKind::InvalidData,
                                                   "Invalid cluster chain" ) )
                },
            }

            //advance the cycle detector twice as fast
            for _ in 0..2 {
                if let Some( x ) = cycle_detect {
                    match self.fat_entry( x )?.status() {
                        Status::Data( y ) => {
                            cycle_detect = Some( y );
                        },
                        Status::Eoc(_) => {
                            cycle_detect = None;
                        },
                        _ => { return Err( io::Error::new( io::ErrorKind::InvalidData,
                                                           "Invalid cluster chain" ) )
                        },
                    }
                }
            }
        }

        Ok( read )
    }

    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.

    pub fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {

        use std::mem;
        use std::slice;
        
        const s : usize = mem::size_of::<FatEntry>();
        let origin = self.fat_start_sector;
        let sector_whole = cluster.cluster_num() * s / self.bytes_per_sector as usize;
        let bytes_remainder = cluster.cluster_num() * s % self.bytes_per_sector as usize;
        let sector_offset = origin + sector_whole as u64;
        let cached_sector_slice : &[u8] = self.device.get( sector_offset )?;
        let fat_entry = unsafe { slice::from_raw_parts( & cached_sector_slice[bytes_remainder] as * const u8 as * const FatEntry, 1 ) };
        
        Ok( &fat_entry[0] )
    }
}

impl<'a> FileSystem for &'a Shared<VFat> {

    type File = File;
    type Dir = Dir;
    type Entry = Entry;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {

        use std::path::Component;
        
        let p = path.as_ref();
        if !p.is_absolute() {
            return Err( io::Error::new( io::ErrorKind::InvalidInput, "path is not absolute" ) )
        }
        let mut v = Vec::new();
        for c in p.components() {
            match c {
                Component::RootDir => {
                    v.clear();
                    let d = Dir::new_dir( self );
                    {
                        use traits::Dir;
                        v.push( Entry::Dir(d) );
                    }
                },
                Component::Normal(x) => {
                    let e = if let Some(entry) = v.last() {
                        use traits::Entry;
                        if let Some(d) = entry.as_dir() {
                            Some( d.find(x)? )
                        } else {
                            None
                        }
                    } else {
                        None                 
                    };
                    match e {
                        Some(y) => {
                            v.push(y);
                        },
                        None => {
                            return Err( io::Error::new( io::ErrorKind::NotFound, "file cannot be found" ) )
                        }
                    }
                },
                Component::ParentDir => { v.pop(); },
                Component::CurDir => {},
                _ => { unimplemented!(); },
            }
        }
        match v.into_iter().last() {
            Some(x) => { Ok(x) },
            _ => { Err( io::Error::new( io::ErrorKind::NotFound, "file cannot be found" ) ) },
        }
    }

    fn create_file<P: AsRef<Path>>(self, _path: P) -> io::Result<Self::File> {
        unimplemented!("read only file system")
    }

    fn create_dir<P>(self, _path: P, _parents: bool) -> io::Result<Self::Dir>
        where P: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn rename<P, Q>(self, _from: P, _to: Q) -> io::Result<()>
        where P: AsRef<Path>, Q: AsRef<Path>
    {
        unimplemented!("read only file system")
    }

    fn remove<P: AsRef<Path>>(self, _path: P, _children: bool) -> io::Result<()> {
        unimplemented!("read only file system")
    }
}
