use stack_vec::StackVec;
use console::{kprint, kprintln, CONSOLE};
use pi::timer;
use std::path;
use fs;
use fat32::traits;
use shell::Command;
use std::io;

pub trait ShellCmd<'a,'b> {
    fn execute( fs: & fs::FileSystem, fs_path: & mut path::PathBuf, arg0: &'a str, args: &[ &'a str ] );
}

///echo command
pub struct CmdEcho {}
impl<'a, 'b> ShellCmd<'a,'b> for CmdEcho {
    fn execute( fs: & fs::FileSystem, fs_path: & mut path::PathBuf, arg0: &'a str, args: &[ &'a str ] ) {
        let n = args.len();
        for (k,v) in args.iter().enumerate() {
            if k == n - 2 {
                kprint!( "{}", v );
            } else {
                kprint!( "{} ", v );
            }
        }
        kprintln!();
    }
}

///pwd command
pub struct CmdPwd {}
impl<'a, 'b> ShellCmd<'a,'b> for CmdPwd {
    fn execute( fs: & fs::FileSystem, fs_path: & mut path::PathBuf, arg0: &'a str, args: &[ &'a str ] ) {
        match fs_path.to_str() {
            Some(x) => {
                kprintln!( "{}", x );
            },
            _ => {},
        }
    }
}

///ls command
pub struct CmdLs {}
impl<'a, 'b> ShellCmd<'a,'b> for CmdLs {
    fn execute( fs: & fs::FileSystem, fs_path: & mut path::PathBuf, arg0: &'a str, args: &[ &'a str ] ) {

        use fat32::traits::{ Entry, Dir, Metadata };

        let mut peek_iter = args.iter().peekable();
        let show_hidden_files = match peek_iter.peek() {
            Some(x) if *x == &"-a" => { true },
            _ => { false },
        };

        if show_hidden_files {
            peek_iter.next(); //advance
        }

        let input_path = if let Some(x) = peek_iter.next() {
            path::PathBuf::from(x)
        } else {
            path::PathBuf::new()
        };

        match get_entry_from_path( fs, & fs_path, & input_path ) {
            Ok( ( entry, abs_path ) ) => {
                if let Some(y) = entry.as_dir() {
                    let mut entries: Vec<_> = y.entries()
                        .expect("entries interator")
                        .collect();
                    for e in entries.iter() {
                        let entry_name = e.name();
                        if show_hidden_files {
                            kprintln!( "{}", entry_name );
                        } else if !e.metadata().hidden() {
                            match entry_name {
                                "." | ".." => {}, //skip
                                other => {
                                    kprintln!( "{}", entry_name );
                                }
                            }
                        }
                    }
                } else {
                    kprintln!( "Err: path is not a directory" );
                }
            },
            Err( e ) => {
                kprintln!( "Err: {}", e );
            }
        }
    }
}

///cd command
pub struct CmdCd {}
impl<'a, 'b> ShellCmd<'a,'b> for CmdCd {
    fn execute( fs: & fs::FileSystem, fs_path: & mut path::PathBuf, arg0: &'a str, args: &[ &'a str ] ) {

        use fat32::traits::{ Entry, Dir };

        if let Some(x) = args.iter().take(1).next() {
            let input_path = path::PathBuf::from(x);
            let path = match get_entry_from_path( fs, & fs_path, & input_path ) {
                Ok( ( entry, abs_path ) ) => {
                    if let Some(y) = entry.as_dir() {
                        Some( abs_path )
                    } else {
                        kprintln!( "Err: path is not a directory" );
                        None
                    }
                },
                Err( e ) => {
                    kprintln!( "Err: {}", e );
                    None
                }
            };
            if let Some(x) = path {
                (*fs_path) = x.clone();
            }
        } else {
            kprintln!( "Err: expected format: cd <path>" );
        }
    }
}

///cat command
pub struct CmdCat {}
impl<'a, 'b> ShellCmd<'a,'b> for CmdCat {
    fn execute( fs: & fs::FileSystem, fs_path: & mut path::PathBuf, arg0: &'a str, args: &[ &'a str ] ) {

        use fat32::traits::{ Entry, Dir };
        use std::io::Read;
        use std::str;
        
        let mut read = 0;
        
        for x in args.iter() {
            let input_path = path::PathBuf::from(x);
            let path = match get_entry_from_path( fs, & fs_path, & input_path ) {
                Ok( ( entry, abs_path ) ) => {
                    if let Some( mut y) = entry.into_file() {
                        let mut temp = vec![ 0u8; y.size ];
                        match y.read( & mut temp[..] ){
                            Ok(x) => {
                                read += x;
                                match str::from_utf8( temp.as_slice() ) {
                                    Ok(d) => {
                                        kprint!( "{}", d );
                                    },
                                    Err(_) =>{
                                        //just print out raw bytes otherwise
                                        for i in temp.iter() {
                                            kprint!( "{:X}", i );
                                        }
                                    }
                                }
                            },
                            Err( e ) => {
                                kprintln!( "Err: {}", e );
                                return
                            },
                        }
                    } else {
                        kprintln!( "Err: path is not a directory" );
                        return
                    }
                },
                Err( e ) => {
                    kprintln!( "Err: {}", e );
                    return
                }
            };
        }
        kprintln!();
    }
}

///helper method to obtain a entry based on current path and input path
fn get_entry_from_path<'b>( fs: & fs::FileSystem, current_path: & path::PathBuf,
                            input_path: & path::PathBuf ) ->
    io::Result< ( <&'b fs::FileSystem as traits::FileSystem >::Entry, path::PathBuf ) > {
        
        use fat32::traits::{ FileSystem, Entry, Dir };

        let abs_path = if input_path.is_absolute() {
            input_path.clone()
        } else {
            let mut temp = current_path.clone();
            
            //simplify path if possible:
            // current path: /some/path/here
            // input relative path: ../../a
            // reduces to: /a
            for (e,v) in input_path.iter().enumerate() {
                let s = v.to_str().unwrap();
                if s == "." {
                    //skip
                } else if s == ".." {
                    if !temp.pop() {
                        temp.push("/"); //insert / if empty
                    }
                } else {
                    temp.push(s);
                }
            }
            temp
        };

        let abs_path_copy = abs_path.clone();
        
        match fs.open( abs_path ) {
            Ok(x) => { Ok( ( x, abs_path_copy ) ) },
            Err(e) => { Err( e ) }
        }
}
