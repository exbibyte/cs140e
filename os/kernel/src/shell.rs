use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use pi::timer;
use std::path::PathBuf;
use fs;
use fat32::traits;
use cmds;
use fs::FileSystem;

use FILE_SYSTEM;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.

pub struct Command<'a> {
    pub args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        self.args[0]
    }
}

///process and branch to specific command handlers
fn flush_to_cmd<'b>( fs: & FileSystem,
                        input: & str, fs_path: & mut PathBuf ) {
    let mut b = [ ""; 512 ];
    match Command::parse( input, & mut b[..] ) {
        Err( Error::Empty ) => {
            
        },
        Err( _ ) => {
            
        },
        Ok( ref x ) => {
            match x.path() {
                "echo" => {
                    < cmds::CmdEcho as cmds::ShellCmd >::execute( fs, fs_path, x.path(), &x.args.as_slice()[1..] );
                },
                "pwd" => {
                    < cmds::CmdPwd as cmds::ShellCmd >::execute( fs, fs_path, x.path(), &x.args.as_slice()[1..] );   
                },
                "ls" => {
                    < cmds::CmdLs as cmds::ShellCmd >::execute( fs, fs_path, x.path(), &x.args.as_slice()[1..] );
                },
                "cd" => {
                    < cmds::CmdCd as cmds::ShellCmd >::execute( fs, fs_path, x.path(), &x.args.as_slice()[1..] );
                },
                "cat" => {
                    < cmds::CmdCat as cmds::ShellCmd >::execute( fs, fs_path, x.path(), &x.args.as_slice()[1..] );
                },                
                _ => {
                    kprintln!("unknown command");
                },
            }
        },
    }
}

fn shift_left( cursor: & mut usize, end: & mut usize, buf: & mut [u8] ) {
    use std::str;

    *cursor -= 1;
    {
        let mut c = CONSOLE.lock();
        c.write_byte( 0x08 ); //shift back
    }
    
    let d = *end - *cursor;

    if d > 0 {
        for i in 0..d {
            buf[*cursor+i] = buf[*cursor+i+1];
            let s = str::from_utf8( &buf[*cursor+i..*cursor+i+1] ).unwrap_or_default();
            kprint!("{}", s );
        }
        kprint!("{}", " ");
        {
            let mut c = CONSOLE.lock();        
            for i in 0..d {
                c.write_byte( 0x08 ); //shift back
            }
        }
        *end -= 1;
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str, fs: & FileSystem ) -> ! {

    use std::io::Read;
    use std::io::Write;
    use std::str;
    use std::fmt;
    use fat32::traits::{ FileSystem, Entry, Dir, File };
    
    let mut buf = [0u8; 512];

    //indices for cursor and end
    let mut idx_cursor = 0;
    let mut idx_end = 0;

    kprint!( "{}", BANNER );
    
    kprint!( "{}", prefix );

    match FILE_SYSTEM.open( "/" ) {
        Ok(_) => {},
        Err(e) => {
            kprintln!( "error opening at /: {}", e );
            panic!();
        }
    };

    let mut fs_path = PathBuf::new();
    fs_path.push("/");
    
    loop {
        
        let bytes_len = {
            let mut c = CONSOLE.lock();
            c.read( & mut buf[idx_end..512] ).expect("num of bytes read")
        };

        let offset = idx_end;
        for i in offset..offset + bytes_len {
            if buf[i] == b'\r' || buf[i] == b'\n' {
                kprintln!();
                {
                    let s = str::from_utf8( & buf[0..idx_end] ).unwrap_or_default();
                    flush_to_cmd( &FILE_SYSTEM, s, & mut fs_path );
                }
                kprint!( "{}", prefix );
                for i in 0..idx_end {
                    buf[i] = 0x00;
                }
                idx_cursor = 0;
                idx_end = 0;
                break;
            } else if buf[i] >= 32 && buf[i] <= 126 {
                let s = str::from_utf8( &buf[i..i+1] ).unwrap_or_default();
                idx_cursor += 1;
                idx_end += 1;
                kprint!( "{}", s ); //print character
            } else if buf[i] == 0x7F || buf[i] == 0x08 { //delete or backspace
                buf[i] == 0x20;
                if idx_cursor != 0 {
                    shift_left( & mut idx_cursor, & mut idx_end, & mut buf );
                }
            } else {
                let s = str::from_utf8( &buf[i..] ).unwrap_or_default();
                kprintln!( "unrecognized: {}", s );
                idx_end = 0;
                idx_cursor = 0;
            }
        }
    }
}

const BANNER: & str =
r##"
...,t1ti.......................................................................................
...,.:.i..................;L,     .   ;........................................................
...,.1.i...,........ .::     . .  tt:, 1 , i...........  ....  .  .............................
..:,.L.;,......... f .        ,t;t;iti; tt:i;;f......   ..        ...   .......................
,;., ;C1........t.......t1;::;;LC.8i,1;:t,Lf..L..........                  ....................
,f,,i;11:::,:;f......:::t..i80fLi8880iitL:L,,C..... ...        .  ...........................,,
:1::;,i1:::0........1Cf0;88800888808C8.if1.,::f  f1fCf     ......,,...,..,,,..,..,,,.,,,..,,,,,
iG;;;;;;if...   ....C80CG88f888C880G8888tt1G.ti:, i:,.t.     ......,,,.,,,,,,,,,,,,,,,,,,,,:,:,
;;;;ii1G... .    .   ;88880888L8818888fLft;i1;.t.1  L1.;.,.,,,,,,,,,,,,,,::::::::::,:::::,:::::
80008...   . :,:..      ft.88t08f8800fC:.::i,:,fL .i1..:,:,,:,,.,,,,,:::::,:::::,::::::::::::::
0,...  ..:1,,t1  .,.:f:C  80.iC0tL000f:;::::,ti:;:,if;1,:,,,:,:,,,,,:,,,,:::;::::::::::::;;::;:
.:,:fi  .L,   1:fi:t11,; t;,1.i;.iGGf;;,;;:;;;:;;;:1:f;;;;;:;,:;.::::::::::::::;;;:;;;;:;:;;t.
.tiLL,.88:80,,:ff;:G8081L0C,Li;0800G,;::;;;;i;::;;i;i:;;;;;;i;;;;:;;;:;;::;;::;:;;;::;;;;1
t11:1:t:G :0C8L,.f,.;1.,,8Lt8G8t8t08f;i;;:;;;;;;i;;;;,;;;:;;,;;;;:;;:;:::;::;:;;i;;;;;:;:,   C;
;0it08G8L8t8fi1,,.tG; .188L0C88Cf8808G;ii;;;;;;;::::;;:;;;;;;;:;;;;;;;;;;;;:;:;:;;::C;18     .:
18; tfC80888:L;880088.:8G08f08t 80880C0i;;;; ,;;i; 1:;:;:i;;;;;i;;;i;;i;ii;iii;i;;f;0.     .,L0
;i,::1Cft088C0800G808801C8G08  ..    0C0ii;;i;;::;,1;.i;;;:iiii;:iii:;;;;i;;;;;i;00.  .   .,180
1L:1:.:Ct,,C888088088f8G88i       ..  188:ii1i,;ii;;;;;ii,:;;i;;.;;:;;i;:;;ii;G.0L   . 1fL00000
88it;8;LCLti,1fiLC808888,   .   C      ,.8G:1:iii;;;:i;;;ii  fii;;;;;i:;i;ii.LfL  :,.i80888888.
G8088C88C;fL0f1L80f0 .  .  ...ff.  .   .i.   CffftfLCttf1 ;  ; CLfffLfLL;t.,t  .  0f08088f.,f ,
:8888G088Gi,fGi .     . ;1..1.,:;.i :,   ,,:..,0000f;C,.G00;0000CC0..,Lf  ::.;.GG;0: :;88;....
....... ,:::....C,.., i.;,:;1L,,  L1:.L.f .i 0 ,t    CG0000CLLfC0G1   .., :0GC: ..,f888G ,,,,GG
;:::::,  .:,,   t,;,,,,,,.G., . .t,:CLttt,G. 00GL0f;00.       ..,.      .  .008888088 ...,00880
;,.. :t,;:,,88ft1 1:t, ,  :,L.,i;,. 8800.t00i. ;80088C08;.....  .. ;81:. 00f000808i .   tCGCLG0
i0::G888,880t.;i;,t.,.t.,L.t;: .:. 88G08.fC1C1iL f8808888G, f88L,:8800;L08888:.....;888888888
888tt180088,;,;,;Ct;.;::i,t,.8fGGGt,. :8C880:tttCffCCC1;iL08888888800000GG0C    :11;:,:iiL88888
88088f0G880:C008888i8,;,,... .,G88i:C,..;888888.1tt;:,............:.:fG1   .,,;8088888888888888
88GC880808080088G0t88:,:188i:i8;;;8080i,,  ;880888GitL111i1tLfCGL;    88088880888G;   .:;::::::
~ Welcome to the Sh3ll ~
"##;
