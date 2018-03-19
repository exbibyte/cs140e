#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]

#![feature(exclusive_range_pattern)]
#![feature(alloc, allocator_api, global_allocator)]

#![feature(never_type)]
#![feature(ptr_internals)]
#![feature(pointer_methods)]

#[macro_use]
#[allow(unused_imports)]
extern crate alloc;
extern crate pi;
extern crate stack_vec;
extern crate fat32;

pub mod allocator;
pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;
pub mod fs;

#[cfg(not(test))]
use allocator::Allocator;
use fs::FileSystem;

#[cfg(not(test))]
#[global_allocator]
pub static ALLOCATOR: Allocator = Allocator::uninitialized();

pub static FILE_SYSTEM: FileSystem = FileSystem::uninitialized();

#[no_mangle]
#[cfg(not(test))]
pub extern "C" fn kmain() {

    use console::{ kprintln };
    use pi::atags::*;
    use std::iter;

    pi::timer::spin_sleep_ms(1000);

    kprintln!( "iterating through ATAGS.." );

    let mut atags : pi::atags::Atags = pi::atags::Atags::get();

    let mut i = atags.current().unwrap();
    loop {
        match i {
            pi::atags::Atag::Core(x) => {
                kprintln!( "atag core: {:#?}", x );
            },
            pi::atags::Atag::Mem(x) => {
                kprintln!( "atag mem: {:#?}", x );
            },
            pi::atags::Atag::Cmd(x) => {
                kprintln!( "atag cmd: {:#?}", x );
            },
            pi::atags::Atag::None => {
                kprintln!( "atag none " );
            },
            pi::atags::Atag::Unknown( x ) => {
                kprintln!( "unknown atag: {:#?}", x );
            },
        }
        match atags.next() {
            Some( x ) => { i = x; },
            None => { break; },
        }
    }

    kprintln!( "initializing allocators.." );
    
    ALLOCATOR.initialize();

    // let mut v = vec![];
    // for i in 0..10 {
    //     v.push(i);
    //     kprintln!("{:?}", v);
    // }

    // let s = String::from( "hi!" );
    
    kprintln!( "starting shell.." );

    let mut gpio_16_out = pi::gpio::Gpio::new(16).into_output();
    gpio_16_out.set();

    shell::shell( "~>" );

}
