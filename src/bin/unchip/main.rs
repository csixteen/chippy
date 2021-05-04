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

use std::env;
use std::io::prelude::*;
use std::fs::File;

const ROM_SIZE: usize = 3584;

fn decode(opcode: u16) {
    let parts = (
        ((opcode & 0xF000) >> 12) as u8,
        ((opcode & 0x0F00) >> 8) as usize,
        ((opcode & 0x00F0) >> 4) as usize,
        (opcode & 0x000F) as u8
    );

    let vx = parts.1;
    let vy = parts.2;
    let nnn = opcode & 0xFFF;
    let kk = (opcode & 0xFF) as u8;
    let n = (opcode & 0xF) as usize;

    let mnemonic = match parts {
        (0x0, 0x0, 0xE, 0x0) => format!("CLS"),
        (0x0, 0x0, 0xE, 0xE) => format!("RET"),
        (0x0, _, _, _)       => format!("SYS {:#05X?} (unsupported)", nnn),
        (0x1, _, _, _)       => format!("JP {:#05X?}", nnn),
        (0x2, _, _, _)       => format!("CALL {:#05X?}", nnn),
        (0x3, _, _, _)       => format!("SE V{}, {:#04X?}", vx, kk),
        (0x4, _, _, _)       => format!("SNE V{}, {:#04X?}", vx, kk),
        (0x5, _, _, 0x0)     => format!("SE V{}, V{}", vx, vy), 
        (0x6, _, _, _)       => format!("LD V{}, {:#04X?}", vx, kk),
        (0x7, _, _, _)       => format!("ADD V{}, {:#04X?}", vx, kk),
        (0x8, _, _, 0x0)     => format!("LD V{}, V{}", vx, vy),
        (0x8, _, _, 0x1)     => format!("OR V{}, V{}", vx, vy),
        (0x8, _, _, 0x2)     => format!("AND V{}, V{}", vx, vy),
        (0x8, _, _, 0x3)     => format!("XOR V{}, V{}", vx, vy),
        (0x8, _, _, 0x4)     => format!("ADD V{}, V{}", vx, vy),
        (0x8, _, _, 0x5)     => format!("SUB V{}, V{}", vx, vy),
        (0x8, _, _, 0x6)     => format!("SHR V{}", vx),
        (0x8, _, _, 0x7)     => format!("SUBN V{}, V{}", vx, vy),
        (0x8, _, _, 0xE)     => format!("SHL V{}", vx),
        (0x9, _, _, 0x0)     => format!("SNE V{}, V{}", vx, vy),
        (0xA, _, _, _)       => format!("LD I, {:#05X?}", nnn),
        (0xB, _, _, _)       => format!("JP V0, {:#05X?}", nnn),
        (0xC, _, _, _)       => format!("RND V{}, {:#04X?}", vx, kk),
        (0xD, _, _, _)       => format!("DRW V{}, V{}, {:#03X?}", vx, vy, n),
        (0xE, _, 0x9, 0xE)   => format!("SKP V{}", vx),
        (0xE, _, 0xA, 0x1)   => format!("SKNP V{}", vx),
        (0xF, _, 0x0, 0x7)   => format!("LD V{}, DT", vx),
        (0xF, _, 0x0, 0xA)   => format!("LD V{}, K", vx),
        (0xF, _, 0x1, 0x5)   => format!("LD DT, V{}", vx),
        (0xF, _, 0x1, 0x8)   => format!("LD ST, V{}", vx),
        (0xF, _, 0x1, 0xE)   => format!("ADD I, V{}", vx),
        (0xF, _, 0x2, 0x9)   => format!("LD F, V{}", vx),
        (0xF, _, 0x3, 0x3)   => format!("LD B, V{}", vx),
        (0xF, _, 0x5, 0x5)   => format!("LD [I], V{}", vx),
        (0xF, _, 0x6, 0x5)   => format!("LD V{}, [I]", vx),
        _ => format!("Unsupported"),
    };

    println!("({:#06X?}) {}", opcode, mnemonic);
}

fn unchip(rom: [u8; ROM_SIZE], size: usize) {
    for i in (0..size).step_by(2) {
        let opcode: u16 = (rom[i] as u16) << 8 | (rom[i+1] as u16);
        decode(opcode);
    }
}

fn main() -> Result<(), String> {
    if let Some(file_name) = env::args().skip(1).next() {
        let mut f = File::open(file_name).map_err(|e| e.to_string())?;
        let mut buffer = [0_u8; ROM_SIZE];
        let size = f.read(&mut buffer).map_err(|e| e.to_string())?;

        if size % 2 != 0 {
            Err("File possibly corrupted.".to_string())
        } else {
            unchip(buffer, size);

            Ok(())
        }
    } else {
        Err("Usage: unchip <chip8 file>".to_string())
    }
}
