extern crate clap;

use std::io;
use std::io::prelude::*;
use std::fs::File;

use clap::{Arg, App};

fn main() -> io::Result<()> {
    let matches = App::new("CHIP-8 interpreter written in Rust.")
                        .version("1.0.0")
                        .author("Pedro Rodrigues <csixteen@protonmail.com>")
                        .arg(Arg::with_name("file_name")
                             .value_name("FILE")
                             .help("CHIP-8 program source file.")
                             .takes_value(true)
                             .required(true))
                        .get_matches();

    let file_name = matches.value_of("file_name").unwrap();
    let mut f = File::open(file_name)?;
    let mut buffer = Vec::new();

    f.read_to_end(&mut buffer)?;

    Ok(())
}
