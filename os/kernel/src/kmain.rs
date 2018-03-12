#![feature(lang_items)]
#![feature(core_intrinsics)]
#![feature(const_fn)]
#![feature(asm)]
#![feature(optin_builtin_traits)]
#![feature(decl_macro)]
#![feature(repr_align)]
#![feature(attr_literals)]
#![feature(never_type)]
#![feature(ptr_internals)]
#![feature(pointer_methods)]

extern crate pi;
extern crate stack_vec;

pub mod lang_items;
pub mod mutex;
pub mod console;
pub mod shell;

const GPIO_BASE: usize = 0x3F000000 + 0x200000;

const GPIO_FSEL1: *mut u32 = (GPIO_BASE + 0x04) as *mut u32;
const GPIO_SET0: *mut u32 = (GPIO_BASE + 0x1C) as *mut u32;
const GPIO_CLR0: *mut u32 = (GPIO_BASE + 0x28) as *mut u32;

#[no_mangle]
pub extern "C" fn kmain() {
    // // FIXME: Start the shell.

    // // STEP 1: Set GPIO Pin 16 as output.
    // // additionally test another GPIO pin 20
    // let mut gpio_pin_16_out = pi::gpio::Gpio::new( 16 ).into_output();
    // let mut gpio_pin_20_out = pi::gpio::Gpio::new( 20 ).into_output();
    
    // // STEP 2: Continuously set and clear GPIO
    // loop {
    //     pi::timer::spin_sleep_ms(500);
    //     gpio_pin_16_out.set();
    //     gpio_pin_20_out.clear();

    //     pi::timer::spin_sleep_ms(500);
    //     gpio_pin_16_out.clear();
    //     gpio_pin_20_out.set();
    // }

    use std::io::Read;
    use std::io::Write;
    
    let mut uart = pi::uart::MiniUart::new();

    let mut buf = [0;1024];
    loop {
        uart.read( & mut buf[..] ).is_ok();
        uart.write( &buf[..] ).is_ok();
        uart.write( b"<-" ).is_ok();
    }
}
