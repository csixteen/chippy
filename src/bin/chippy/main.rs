// MIT License
//
// Copyright (c) 2021 Pedro Rodrigues
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

extern crate clap;

use std::io::prelude::*;
use std::fs::File;

use clap::{Arg, App};

use chippy::emulator::{Emulator, ROM_SIZE};

fn main() -> Result<(), String> {
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
    let mut f = File::open(file_name).map_err(|e| e.to_string())?;
    let mut buffer = [0_u8; ROM_SIZE];
    f.read(&mut buffer).map_err(|e| e.to_string())?;

    Emulator::run(buffer)
}
