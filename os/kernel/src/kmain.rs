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

    pi::timer::spin_sleep_ms(1000);
    
    ALLOCATOR.initialize();

    // FIXME: Start the shell.

    let mut gpio_16_out = pi::gpio::Gpio::new(16).into_output();
    gpio_16_out.set();

    shell::shell( "~>" );

}
