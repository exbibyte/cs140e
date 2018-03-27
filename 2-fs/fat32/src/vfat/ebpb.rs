use std::fmt;

use traits::BlockDevice;
use vfat::Error;

#[repr(C, packed)]
#[derive(Clone,Copy)]
pub struct BiosParameterBlock {
    first_three_bytes: [ u8; 3 ],
    oem_identifier: [ u8; 8 ],
    pub num_bytes_per_sector: u16, //little-endian
    pub num_sectors_per_cluster: u8,
    pub num_reserved_sectors: u16,
    pub num_file_allocation_tables: u8,
    max_num_directory_entries: [ u8; 2 ],
    total_logical_sections: [ u8; 2 ],
    media_descriptor_type: u8,
    pub num_sectors_per_fat: u16,
    num_sectors_per_track: [ u8; 2 ],
    num_heads_sides: [ u8; 2 ],
    num_hidden_sectors: [ u8; 4 ],
    total_logical_sectors: [ u8; 4 ],
    //extended BPB below
    pub sectors_per_fat: u32,
    flags: [ u8; 2 ],
    fat_version_number: [ u8; 2 ],
    pub cluster_num_root_dir: u32,
    sector_num_FSInfo: [ u8; 2 ],
    sector_num_backup_boot_sector: [ u8; 2 ],
    _reserved: [ u8; 12 ],
    drive_num: u8,
    flags_win_nt: u8,
    signature: u8,
    volumeid_serial_num: [ u8; 4 ],
    volume_label_string: [ u8; 11 ],
    system_identifier_string: [ u8; 8 ],
    boot_code: [ u8; 420 ],
    bootable_partition_signature: [ u8; 2 ],
}

impl BiosParameterBlock {
    /// Reads the FAT32 extended BIOS parameter block from sector `sector` of
    /// device `device`.
    ///
    /// # Errors
    ///
    /// If the EBPB signature is invalid, returns an error of `BadSignature`.
    pub fn from<T: BlockDevice>(
        mut device: T,
        sector: u64
    ) -> Result<BiosParameterBlock, Error> {
        
        use std::mem;
        use std::slice;
        use std::ptr;

        const s : usize = mem::size_of::<BiosParameterBlock>();
        
        let mut buf = vec![];
        let read_size = match device.read_all_sector( sector, & mut buf ) {
            Ok(x) if x == s => {},
            Ok(_) => {
                return Err( Error::Ebpb )
            },
            Err(x) => { return Err( Error::Io(x) ) }
        };

        let b = unsafe{ ptr::read( & buf[0] as * const u8 as * const BiosParameterBlock ).clone() };

        // match b.signature {
        //     0x28 | 0x29 => {},
        //     _ => { return Err( Error::BadSignature ) }
        // }

        match ( b.bootable_partition_signature[0], b.bootable_partition_signature[1] ) {
            ( 0x55, 0xAA ) => {},
            _ => { return Err( Error::BadSignature ) }
        }

        // match ( b.first_three_bytes[0], b.first_three_bytes[2] ) {
        //     ( 0xEB, 0x90 ) => {},
        //     _ => { return Err( Error::BadSignature ) }
        // }
        
        Ok( b )    
    }
}

impl fmt::Debug for BiosParameterBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        f.debug_struct( "BiosParameterBlock")
            .field("first_three_bytes", & self.first_three_bytes )
            .field("oem_identifier", & self.oem_identifier )
            .field("num_bytes_per_sector", & self.num_bytes_per_sector )
            .field("num_sectors_per_cluster", & self.num_sectors_per_cluster )
            .field("num_reserved_sectors", & self.num_reserved_sectors )
            .field("num_file_allocation_tables", & self.num_file_allocation_tables )
            .field("max_num_direectory_entries", & self.max_num_directory_entries )
            .field("total_logical_sections", & self.total_logical_sections )
            .field("media_descriptor_type", & self.media_descriptor_type )
            .field("num_sectors_per_fat", & self.num_sectors_per_fat )
            .field("num_sectors_per_track", & self.num_sectors_per_track )
            .field("num_heads_sides", & self.num_heads_sides )
            .field("num_hidden_sectors", & self.num_hidden_sectors )
            .field("total_logical_sectors", & self.total_logical_sectors )
            .field("sectors_per_fat", & self.sectors_per_fat )
            .field("flags", & self.flags )
            .field("fat_version_number", & self.fat_version_number )
            .field("cluster_num_root_dir", & self.cluster_num_root_dir )
            .field("sector_num_FSInfo", & self.sector_num_FSInfo )
            .field("sector_num_backup_boot_sector", & self.sector_num_backup_boot_sector )
            .field("drive_num", & self.drive_num )
            .field("flags_win_nt", & self.flags_win_nt )
            .field("signature", & self.signature )
            .field("volumeid_serial_num", & self.volumeid_serial_num )
            .field("volume_label_string", & self.volume_label_string )
            .field("system_identifier_string", & self.system_identifier_string )
            // .field("boot_code", & self.boot_code )
            .field("bootable_partition_signature", & self.bootable_partition_signature )
            .finish()
    }
}
