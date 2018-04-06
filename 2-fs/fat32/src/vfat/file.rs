use std::cmp::{min, max};
use std::io::{self, SeekFrom};

use traits;
use vfat::{VFat, Shared, Cluster, Metadata};

#[derive(Debug)]
pub struct File {
    // FIXME: Fill me in.
    pub first_cluster: Cluster,
    pub vfat: Shared<VFat>,
    pub meta: Metadata,
    pub size: usize,
    pub short_file_name: String,
    pub lfn: String,
    pub bytes_per_cluster: usize,
    pub current_offset: usize, //in bytes from start of the file
    pub current_cluster: Cluster,
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.

impl io::Seek for File {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {

        use vfat::Status;
        use traits::Dir;
        
        let p = match pos {
            SeekFrom::Start(x) => { x as u64 },
            SeekFrom::Current(x) => { ( self.current_offset as i64 ).wrapping_add( x as i64 ) as u64 },
            SeekFrom::End(x) => { ( self.size as i64 ).wrapping_add( x as i64 ) as u64 },
        } as usize;

        if p >= self.size {
            return Err( io::Error::new( io::ErrorKind::InvalidInput, "seeking past end of the file" ) )
        } else {
            //get the cluster location of the desired position and update internal state
            let n = p / self.bytes_per_cluster;
            let mut fs = self.vfat.borrow_mut();
            let mut cluster = self.first_cluster;
            for _ in 0..n {
                match fs.fat_entry( cluster )?.status() {
                    Status::Data(x) => {
                        cluster = x;
                    },
                    _ => {
                        return Err( io::Error::new( io::ErrorKind::InvalidInput, "seeking did not find enough clusters" ) )
                    }
                }
            }
            self.current_offset = p;
            self.current_cluster = cluster;
            return Ok( self.current_offset as u64 )
        }
    }
}

/// Trait implemented by files in the file system.
impl traits::File for File {
    /// Writes any buffered data to disk.
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!();
    }

    /// Returns the size of the file in bytes.
    fn size(&self) -> u64 {
        self.size as u64
    }
}

impl io::Read for File {
    fn read( & mut self, buf: & mut  [u8] ) -> io::Result<usize> {

        use vfat::Status;
        use traits::Dir;

        let len_read_max = {
            let bytes_left = self.size - self.current_offset;
            if buf.len() < bytes_left {
                buf.len()
            } else {
                bytes_left
            }
        };

        let mut offset_in_current_cluster = self.current_offset % self.bytes_per_cluster;

        let mut fs = self.vfat.borrow_mut();

        let mut cluster = self.current_cluster;

        let mut read = 0;

        // let mut read_cluster = 0;
        
        while read < len_read_max {
            let r = fs.read_cluster( cluster, offset_in_current_cluster, & mut buf[read..len_read_max] )?;
            // read_cluster += r;
            // let exit = if read_cluster + offset_in_current_cluster == self.bytes_per_cluster {
            let exit = if r + offset_in_current_cluster == self.bytes_per_cluster {
                //move to next cluster
                match fs.fat_entry( cluster )?.status() {
                    Status::Data(x) => {
                        cluster = x;
                        // read_cluster = 0;
                        false
                    },
                    Status::Eoc(_) => {
                        true
                    },
                    _ => {
                        return Err( io::Error::new( io::ErrorKind::InvalidData, "next cluster invalid" ) );
                    }
                }
            } else {
                false
            };

            // println!("read bytes: {}", r );
            read += r;
            offset_in_current_cluster = 0; //zero offset after first read

            if exit {
                break;
            }
        }
        self.current_cluster = cluster;
        self.current_offset += len_read_max;
        Ok( len_read_max )
    }
    
}

//don't need these for now
impl io::Write for File {
    fn write( & mut self, buf: & [u8] ) -> io::Result<usize> {
        unimplemented!();
    }
    fn flush( & mut self ) -> io::Result< () > {
        unimplemented!();
    }
}
