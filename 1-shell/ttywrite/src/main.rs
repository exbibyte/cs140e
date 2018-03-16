extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use xmodem::{Xmodem, Progress};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

use std::io::{ Read, Cursor, BufReader };

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    use std::fs::File;
    use std::io::{self};

    let opt = Opt::from_args();
    let mut serial = serial::open(&opt.tty_path).expect("path points to invalid TTY");
    let mut settings = serial.read_settings().unwrap();
    settings.set_baud_rate( opt.baud_rate ).is_ok();
    settings.set_flow_control( opt.flow_control );
    settings.set_stop_bits( opt.stop_bits );
    settings.set_char_size( opt.char_width );
    serial.write_settings( & settings ).is_ok();
    serial.set_timeout( Duration::from_secs(opt.timeout) ).is_ok();
    let mut input_buf : Vec< u8 > = vec![];
    match opt.input {
        None => {
            let stdin = io::stdin();
            let mut h = stdin.lock();
            let n = h.read_to_end( & mut input_buf ).expect( "read from stdin unsuccessful" );
            println!( "bytes read from stdin: {}", n );
        },
        Some( ref x ) => {
            let mut f = File::open( x ).expect("file not found");
            let bytes_read = f.read_to_end( & mut input_buf ).expect( "reading file unsuccessful" );
            println!( "bytes read from file: {}", bytes_read );
        },
    }
    let mut buf = Cursor::new( input_buf );
    if opt.raw {
        //send raw
        io::copy( & mut buf, & mut serial ).expect("serial write unsuccessful");
    } else {
        //send via XMODEM protocol
        Xmodem::transmit_with_progress( & mut buf, & mut serial, progress_fn ).expect("xmodem write unsuccessful");
    }
}

fn progress_fn(progress: Progress) {
    println!("Progress: {:?}", progress);
}
