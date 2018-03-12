use core::fmt;

use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, Reserved};

use timer;
use common::IO_BASE;
use gpio::{Gpio, Function};

/// The base address for the `MU` registers.
const MU_REG_BASE: usize = IO_BASE + 0x215040;

/// The `AUXENB` register from page 9 of the BCM2837 documentation.
const AUX_ENABLES: *mut Volatile<u8> = (IO_BASE + 0x215004) as *mut Volatile<u8>;

/// Enum representing bit fields of the `AUX_MU_LSR_REG` register.
#[repr(u8)]
enum LsrStatus {
    DataReady = 1,
    TxAvailable = 1 << 5,
}

#[repr(C)]
#[allow(non_snake_case)]
struct Registers {
    // FIXME: Declare the "MU" registers from page 8.
    IO: Volatile<u8>,
    __r_io: [ Volatile<u8>; 3 ],
    IER: Volatile<u32>,
    IIR: Volatile<u8>,
    __r_iir: [ Volatile<u8>; 3 ],
    LCR: Volatile<u8>,
    __r_lcr: [ ReadVolatile<u8>; 3 ],
    MCR: Volatile<u32>,
    LSR: ReadVolatile<u32>,
    MSR: ReadVolatile<u32>,
    SCRATCH: Volatile<u8>,
    __r_scratch: [ Volatile<u8>; 3 ],
    CNTL: Volatile<u32>,
    STAT: ReadVolatile<u32>,
    BAUD: Volatile<u16>,
    __r_baud: [ Volatile<u8>; 2 ],
}

/// The Raspberry Pi's "mini UART".
pub struct MiniUart {
    registers: &'static mut Registers,
    timeout: Option<u32>,
}

impl MiniUart {
    /// Initializes the mini UART by enabling it as an auxiliary peripheral,
    /// setting the data size to 8 bits, setting the BAUD rate to ~115200 (baud
    /// divider of 270), setting GPIO pins 14 and 15 to alternative function 5
    /// (TXD1/RDXD1), and finally enabling the UART transmitter and receiver.
    ///
    /// By default, reads will never time out. To set a read timeout, use
    /// `set_read_timeout()`.
    pub fn new() -> MiniUart {
        let registers = unsafe {
            // Enable the mini UART as an auxiliary device.
            (*AUX_ENABLES).or_mask(1);
            &mut *(MU_REG_BASE as *mut Registers)
        };

        // FIXME: Implement remaining mini UART initialization.
        
        (*registers).BAUD.write( 270 );

        (*registers).LCR.or_mask( 0b11 ); //set for 8-bit mode

        //alt function, see page 102 of the ARM peripherals document
        let mut _gpio_pin_14_txd1 = Gpio::new( 14 ).into_alt( Function::Alt5 ); //TXD1
        let mut _gpio_pin_15_rxd1 = Gpio::new( 15 ).into_alt( Function::Alt5 ); //RXD1

        MiniUart {
            registers: registers,
            timeout: None,
        }
    }

    /// Set the read timeout to `milliseconds` milliseconds.
    pub fn set_read_timeout(&mut self, milliseconds: u32) {
        self.timeout = Some( milliseconds );
    }

    /// Write the byte `byte`. This method blocks until there is space available
    /// in the output FIFO.
    pub fn write_byte(&mut self, byte: u8) {
        loop {
            if (*self.registers).STAT.has_mask( 0b1 << 1 ) {
                break;
            }
        }
        (*self.registers).IO.write( byte );
    }

    /// Returns `true` if there is at least one byte ready to be read. If this
    /// method returns `true`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately. This method does not block.
    pub fn has_byte(&self) -> bool {
        (*self.registers).STAT.has_mask( 0b1 )
    }

    /// Blocks until there is a byte ready to read. If a read timeout is set,
    /// this method blocks for at most that amount of time. Otherwise, this
    /// method blocks indefinitely until there is a byte to read.
    ///
    /// Returns `Ok(())` if a byte is ready to read. Returns `Err(())` if the
    /// timeout expired while waiting for a byte to be ready. If this method
    /// returns `Ok(())`, a subsequent call to `read_byte` is guaranteed to
    /// return immediately.
    pub fn wait_for_byte(&self) -> Result<(), ()> {
        let t_start = timer::current_time();
        loop {
            match self.timeout {
                None => {},
                Some(x) => {
                    let t_now = timer::current_time();
                    if t_now >= t_start + x as u64 * 1000 {
                        return Err( () )
                    }
                },
            }
            if self.has_byte() {
                break;
            }
        }
        Ok( () )
    }

    /// Reads a byte. Blocks indefinitely until a byte is ready to be read.
    pub fn read_byte(&mut self) -> u8 {
        loop {
            if self.has_byte() {
                break;
            }
        }
        (*self.registers).IO.read()
    }
}

// FIXME: Implement `fmt::Write` for `MiniUart`. A b'\r' byte should be written
// before writing any b'\n' byte.

impl fmt::Write for MiniUart {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        for i in s.as_bytes() {
            self.write_byte(*i);
        }
        // self.write_byte(b'\r');
        // self.write_byte(b'\n');
        Ok( () )
    }
    fn write_char(&mut self, c: char) -> Result<(), fmt::Error> {
        self.write_byte( c as u8 );
        Ok( () )
    }
    // fn write_fmt(&mut self, args: fmt::Arguments) -> Result<(), fmt::Error> {
    //     // let s = std::format!( "{}", args );
    //     let mut s = [0u8; 256];
    //     fmt::write( & mut s, args ).is_ok();
    //     self.write_str( s.as_str() );
    //     Ok( () )
    // }
}

#[cfg(feature = "std")]
mod uart_io {
    use std::io;
    use super::MiniUart;

    // FIXME: Implement `io::Read` and `io::Write` for `MiniUart`.
    //
    // The `io::Read::read()` implementation must respect the read timeout by
    // waiting at most that time for the _first byte_. It should not wait for
    // any additional bytes but _should_ read as many bytes as possible. If the
    // read times out, an error of kind `TimedOut` should be returned.
    //
    // The `io::Write::write()` method must write all of the requested bytes
    // before returning.

    impl io::Read for MiniUart {
        fn read( &mut self, buf: &mut [u8] ) -> io::Result<usize> {
            let mut i = 0;
            match self.wait_for_byte() {
                Ok(_) => {
                    //read rest of available bytes
                    while self.has_byte() && i < buf.len() {
                        buf[i] = self.read_byte();
                        i += 1;
                    }
                },
                _ => {
                    return Err( io::Error::new( io::ErrorKind::TimedOut, "read timeout" ) )
                },
            }
            Ok( i )
        }
    }

    impl io::Write for MiniUart {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            for i in buf.iter() {
                self.write_byte( *i );
            }
            Ok( buf.len() )
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok( () )
        }
    }
}
