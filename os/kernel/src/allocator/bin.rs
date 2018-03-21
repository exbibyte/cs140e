use std::fmt;
use alloc::heap::{AllocErr, Layout};

use allocator::util::*;
use allocator::linked_list::LinkedList;

const K : usize = 13;

/// A simple allocator that allocates based on size classes.
#[derive(Debug)]
pub struct Allocator {
    start: usize,
    end: usize,
    global_start: usize,
    freelists: [ LinkedList; K ], //bins of 2^i for i in [1,K]
    global_freelist: LinkedList,
    global_busylist: LinkedList,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {

        use std::mem;
        
        // #[cfg(test)]
        // println!( "bin initializing allocator start, end: {}, {}", start, end );
//        let start_offset = align_up( start, 2 << (K-1) );

        let total = end - start;
        let alloc_bin = total / 2; //reserve half for binned freelists

        let bin = alloc_bin / (K-1); //split approximately equal amount for each bin size

        let mut freelists = [ LinkedList::new(); K ];

        let mut offset = start;
        
        for i in 1..K {
            //setup bins for current bin size
            let s = 2 << i;

            offset = align_up( offset, mem::size_of::<usize>() );

            let n =  bin / s;
            
            for j in 0..n {
                // println!("init memory addr for bin: {:#?}, size: {}", (offset +  j * s) as * mut usize, s );
                unsafe { freelists[i-1].push( ( offset +  j * s ) as * mut usize ); }
            }
            offset += n * s;
        }
        
        // #[cfg(test)]
        // println!( "initializing freelist offset: {}", offset );

        let mut global_freelist = LinkedList::new();
        unsafe{ global_freelist.push( offset as * mut usize ); }

        let mut global_busylist = LinkedList::new();

        Self {
            freelists: freelists,
            start: start,
            global_start: offset,
            end: end,
            global_freelist: global_freelist,
            global_busylist: global_busylist,
        }
    }

    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning `Err` indicates that either memory is exhausted
    /// (`AllocError::Exhausted`) or `layout` does not meet this allocator's
    /// size or alignment constraints (`AllocError::Unsupported`).
    pub fn alloc(&mut self, layout: Layout) -> Result<*mut u8, AllocErr> {
        
        use std::mem;
        
        if layout.size() == 0 {
            return Err( AllocErr::Unsupported { details: "zero sized allocation" } )
        }

        let align_adj = if mem::size_of::<usize>() > layout.align() {
            mem::size_of::<usize>()
        } else {
            layout.align()
        };

        let mut size_constraint = if mem::size_of::<usize>() > layout.size() {
            mem::size_of::<usize>()
        } else {
            layout.size()
        };

        size_constraint = align_up( size_constraint, mem::size_of::<usize>() );

        // #[cfg(test)]
        // println!( "align_adj: {}", align_adj );
        // #[cfg(test)]
        // println!( "layout_size: {}", size_constraint );

        if layout.align() == mem::size_of::<usize>() &&
            layout.size().count_zeros() == 1 &&
            layout.size() < (2 << (K-1))
        {
            assert!( size_constraint.trailing_zeros() > 0 );
            let idx_list = size_constraint.trailing_zeros() - 1;
            
            let mut freelist = self.freelists[ idx_list as usize ];
            match freelist.pop() {
                Some( x ) => {
                    //found slot
                    // #[cfg(test)]
                    // println!( "allocate through bin: {:#?}", x );
                    return Ok( x as * mut u8 )
                },
                None => {
                    // #[cfg(test)]
                    // println!( "allocate through global pool" );
                    //try from global pool
                }
            }
        }
        
        //First fit allocator
        //Allocate an extra pointer (to act as the header) plus the amount requested by the caller.
        //The returned address is the +1 pointer offset away from the actual allocated memory.
        
        let ( fragment_start, alloc_start, fragment_end ) = {
            let mut iter_free = self.global_freelist.iter();
            let mut iter_busy = self.global_busylist.iter();

            let mut addr_free = iter_free.next();
            let mut addr_busy = iter_busy.next();

            let alloc_info = loop {

                // #[cfg(test)]
                // println!( "looping.." );

                let ( allocate, continue_free, continue_busy ) = match ( &addr_free, &addr_busy ) {
                    ( &Some(ref f), &Some(ref b) ) => {
                        // #[cfg(test)]
                        // println!( "case 0: addr_free: {:#?}, addr_busy: {:#?}", *f, *b );
                        if (*b as usize) < (*f as usize) {
                            //continue search busylist
                            ( None, false, true )
                        } else {

                            //reserve an extra pointer for header

                            let f_content = *f as usize + mem::size_of::<usize>();
                            
                            let start = align_up( f_content, align_adj );
                            let f_header = start - mem::size_of::<usize>(); //actual start of address for allocation
                            
                            let frag_start = if f_header != *f as usize {
                                Some( *f as usize )
                            } else {
                                None
                            };

                            // #[cfg(test)]
                            // println!( "case 0: continued" );
                            let e = start.saturating_add( size_constraint );
                            if e <= *b as usize {
                                //allocate
                                if e != *b as usize {
                                    ( Some( ( frag_start, f_header, Some( e ) ) ), false, false )
                                } else {
                                    ( Some( ( frag_start, f_header, None ) ), false, false )
                                }
                            } else {
                                //continue search freelist
                                ( None, true, false )
                            }
                        }
                    },
                    ( &Some(ref f), &None ) => { //case for where there is no blocks in busylist
                        // #[cfg(test)]
                        // println!( "case 1: addr_free: {:#?}", *f );
                        //check size constraint and allocate if possible

                        //reserve an extra pointer for header
                        let f_content = *f as usize + mem::size_of::<usize>();
                        
                        let start = align_up( f_content, align_adj );
                        let f_header = start - mem::size_of::<usize>();
                        
                        let frag_start = if f_header != *f as usize {
                            Some( *f as usize )
                        } else {
                            None
                        };

                        // #[cfg(test)]
                        // println!( "case 1 continued: addr_free: {:#?}", *f );
                        
                        let end = start.saturating_add( size_constraint );
                        if end > self.end {
                            return Err( AllocErr::Exhausted { request: layout } )
                        } else if end < self.end {
                            ( Some( ( frag_start, f_header, Some(end) ) ), false, false )
                        } else {
                            ( Some( ( frag_start, f_header, None ) ), false, false )
                        }
                    },
                    _  => { //case for where freelist is empty
                        // #[cfg(test)]
                        // println!( "case 2: freelist empty" );
                        return Err( AllocErr::Exhausted { request: layout } )
                    },
                };

                if let Some(x) = allocate {
                    // #[cfg(test)]
                    // println!("allocating...");
                    break x;
                } else {
                    if continue_free {
                        // #[cfg(test)]
                        // println!("next free block");
                        addr_free = iter_free.next();
                    }
                    if continue_busy {
                        // #[cfg(test)]
                        // println!("next busy block");
                        addr_busy = iter_busy.next();
                    }
                }
            };
            alloc_info
        };

        // #[cfg(test)]
        // println!( "aloc info: {:?}", (fragment_start, alloc_start, fragment_end ) );

        if let None = fragment_start {
            for i in self.global_freelist.iter_mut() {
                // #[cfg(test)]
                // println!( "popping off alloc_start: {:#?}", alloc_start);
                if i.value() as usize == alloc_start {
                    i.pop();
                    break;
                }
            }
        }

        // #[cfg(test)]
        // println!( "freelist length: {:#?}", self.global_freelist.iter().count() );
        
        if let Some(x) = fragment_end {
            // #[cfg(test)]
            // println!( "inserting for fragment_end: {:#?}", x );
            unsafe { self.global_freelist.insert_ascending( x as * mut usize, size_constraint ); }
        }
        
        unsafe { self.global_busylist.insert_ascending( alloc_start as * mut usize, size_constraint ); }

        //return +1 pointer offset for actual caller use 
        let ret = ( alloc_start + mem::size_of::<usize>() ) as * mut usize as * mut u8;

        assert!( (alloc_start as usize) < self.end );
        assert!( (alloc_start as usize) >= self.global_start );
        
        // #[cfg(test)]
        // println!("bin allocated addr: {:?}", ret );
        // #[cfg(test)]
        // println!( "busylist: {:?}", self.global_busylist );
        // #[cfg(test)]
        // println!( "freelist: {:?}", self.global_freelist );
        
        return Ok( ret )
    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    pub fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {

        use std::mem;

        // #[cfg(test)]
        // println!( "bin deallocating" );

        let mut size_constraint = if mem::size_of::<usize>() > layout.size() {
            mem::size_of::<usize>()
        } else {
            layout.size()
        };

        size_constraint = align_up( size_constraint, mem::size_of::<usize>() );

        if (ptr as usize) < self.global_start {
            for i in 1..K {
                if size_constraint <= (2 << i) {
                    //deallocate to this bin
                    let idx_list = i - 1;
                    let mut freelist = self.freelists[ idx_list ];
                    unsafe { freelist.push( ptr as * mut usize ); }
                    // #[cfg(test)]
                    // println!( "deallocate through bins" );
                    return ()
                }
            }
        }

        // #[cfg(test)]
        // println!( "deallocate through global pool" );
        
        //deallocation to global pool and coelesce neighbouring blocks if possible

        //get the header of the block by offsetting -1 pointer size from ptr
        let ptr_header = unsafe { (ptr as * mut usize).sub(1) };
        
        let mut busy_found = None;
        let mut busy_next = None;
        let mut busy_prev = None;
        for i in self.global_busylist.iter_mut() {
            if i.value() as usize == ptr_header as * mut usize as usize {
                let val = i.pop() as usize;
                busy_found = Some( val );
            } else
                if (i.value() as usize) < ptr_header as * mut usize as usize {
                busy_prev = Some( i.value() as usize );
            } else {
                busy_next = Some( i.value() as usize );
                break;
            }
        }
        
        if let None = busy_found {
            panic!( "item to free not busy_found" );
        }
        
        let mut free_next = None;
        let mut free_prev = None;
        for i in self.global_freelist.iter() {
            if (i as usize) < (ptr_header as * mut usize as usize) {
                free_prev = Some( i as usize );
            }
            if (i as usize) > (ptr_header as * mut usize as usize) {
                free_next = Some( i as usize );
                break;
            }
            if (i as usize) == (ptr_header as * mut usize as usize) {
                panic!( "item to free also present in existing freelist" );
            }
        }

        let mut put_back_to_freelist = ptr_header as * mut usize as usize;

        match ( free_next, busy_next ) {
            ( Some(f), Some(b) ) if f < b => {
                //coelesce neighbouring blocks by removing it
                for i in self.global_freelist.iter_mut() {
                    if i.value() as usize == f {
                        i.pop();
                    }
                }
            },
            ( Some(f), None ) => {
                //coelesce neighbouring blocks by removing it
                for i in self.global_freelist.iter_mut() {
                    if i.value() as usize == f {
                        i.pop();
                    }
                }
            },
            _ => {},
        }
        
        match ( free_prev, busy_prev ) {
            ( Some(f), Some(b) ) if f > b => {
                //coelesce neighbouring blocks by removing it
                for i in self.global_freelist.iter_mut() {
                    if i.value() as usize == f {
                        i.pop();
                    }
                }
                put_back_to_freelist = f;
            },
            ( Some(f), None ) => {
                //coelesce neighbouring blocks by removing it
                for i in self.global_freelist.iter_mut() {
                    if i.value() as usize == f {
                        i.pop();
                    }
                }
                put_back_to_freelist = f;
            },
            _ => {},
        }

        unsafe{ self.global_freelist.insert_ascending( put_back_to_freelist as * mut usize, layout.size() ); }

        // #[cfg(test)]
        // println!( "busylist: {:?}", self.global_busylist );
        // #[cfg(test)]
        // println!( "freelist: {:?}", self.global_freelist );
    }
}
//
// FIXME: Implement `Debug` for `Allocator`.
