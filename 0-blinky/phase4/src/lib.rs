#![feature(compiler_builtins_lib, lang_items, asm, pointer_methods)]
#![no_builtins]
#![no_std]

extern crate compiler_builtins;

pub mod lang_items;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[inline(never)]
fn spin_sleep_ms(ms: usize) {
    for _ in 0..(ms * 600) {
        unsafe { asm!("nop" :::: "volatile"); }
    }
}

#[no_mangle]
pub unsafe extern "C" fn kmain() {

    // STEP 1: Set GPIO Pin 16 as output.
    GPIO_FSEL1.write_volatile( GPIO_FSEL1.read_volatile() | 0b001 << 18 );

    // STEP 2: Continuously set and clear GPIO 16.
    loop {
        spin_sleep_ms(250);
        GPIO_SET0.write_volatile( GPIO_SET0.read_volatile() | 0x01 << 16 );
        spin_sleep_ms(250);
        GPIO_CLR0.write_volatile( GPIO_CLR0.read_volatile() | 0x01 << 16 );
    }

}
