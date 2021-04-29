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

#![allow(non_snake_case)]

use std::mem;

use crate::debug::DebugLog;

pub(crate) const DISPLAY_WIDTH: usize   = 64;
pub(crate) const DISPLAY_HEIGHT: usize  = 32;
const STACK_SIZE: usize     = 16;
const ROM_OFFSET: usize     = 0x200;
const SPRITE_SIZE: usize    = 5;  // size in bytes
const DEBUG_LOG_SIZE: usize = 32;

#[derive(Default)]
pub struct Chip8 {
    mem: Vec<u8>,
    stack: [usize; STACK_SIZE],

    // Registers - the register VF shouldn't be
    // used by programs, as it is used as a flag
    // by some instructions
    v_reg: [u8; 16],  // V0..VF
    i: usize, // used to store memory addresses. Only the lowest 12 bits are used

    // Timers
    delay_t: u8,  // delay timer
    sound_t: u8,  // sound timer

    // Pseudo-registers (not directly accessible to the user)
    pc: usize,  // Program Counter
    sp: usize,   // Stack-Pointer

    // +---------------+
    // | 1 | 2 | 3 | C |
    // +---+---+---+---+
    // | 4 | 5 | 6 | D |
    // +---+---+---+---+
    // | 7 | 8 | 9 | E |
    // +---+---+---+---+
    // | A | 0 | B | F |
    // +---+---+---+---+
    keypad: [u8; 16],  // 16-key hexadecimal keypad

    display: Vec<u8>,
    pub(crate) draw: bool,

    dbg_log: DebugLog,
}

// Preloaded sprite data representing a font of sixteen
// hexadecimal digits.
const FONT_DATA: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

enum ProgramCounter {
    Next,
    Skip,
    Address(usize),
}

impl ProgramCounter {
    fn skip_if(cond: bool) -> ProgramCounter {
        if cond { ProgramCounter::Skip }
        else { ProgramCounter::Next }
    }
}

impl Chip8 {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mem = vec![0_u8; 4096];

        (0..80).for_each(|i| { mem[i] = FONT_DATA[i]; });
        (0..rom.len()).for_each(|i| { mem[i + ROM_OFFSET] = rom[i]; });

        Chip8 {
            mem: mem,
            display: vec![0_u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            dbg_log: DebugLog::new(DEBUG_LOG_SIZE),
            ..Default::default()
        }
    }

    pub fn initialize(&mut self) {
        self.i = 0;
        self.sp = 0;
        self.pc = ROM_OFFSET;
        self.delay_t = 0;
        self.sound_t = 0;

        mem::take(&mut self.v_reg);
        mem::take(&mut self.keypad);
        mem::take(&mut self.stack);

        self.clear_display();
    }

    pub fn clear_display(&mut self) {
        for i in 0..self.display.len() {
            self.display[i] = 0;
        }
        self.draw = true;
    }

    pub fn pixel_at(&self, row: usize, col: usize) -> u8 {
        self.display[row * DISPLAY_WIDTH + col]
    }

    // -------------------------------------------------
    // Fetch, Decode and Execute
    pub fn fetch_decode_execute(&mut self) {
        let opcode = self.fetch_opcode();

        self.pc = self.execute_instruction(opcode);

        //println!("{}", self.dbg_log.last_entry().unwrap());

        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            self.sound_t -= 1;
        }
    }

    fn fetch_opcode(&self) -> u16 {
        (self.mem[self.pc] as u16) << 8 |
            (self.mem[self.pc + 1] as u16)
    }

    fn execute_instruction(&mut self, opcode: u16) -> usize {
        let parts = (
            ((opcode & 0xF000) >> 12) as usize,
            ((opcode & 0x0F00) >> 8) as usize,
            ((opcode & 0x00F0) >> 4) as usize,
            (opcode & 0x000F) as usize
        );

        let vx = parts.1;
        let vy = parts.2;
        let nnn = (opcode & 0xFFF) as usize;
        let kk = (opcode & 0xFF) as u8;
        let n = (opcode & 0xF) as usize;

        let log_entry: String;

        let new_pc = match parts {
            (0x0, 0x0, 0xE, 0x0) => self.execute_CLS(),
            (0x0, 0x0, 0xE, 0xE) => self.execute_RET(),
            (0x1, _, _, _)       => self.execute_JP_addr(nnn),
            (0x2, _, _, _)       => self.execute_CALL_addr(nnn),
            (0x3, _, _, _)       => self.execute_SE_Vx_kk(vx, kk),
            (0x4, _, _, _)       => self.execute_SNE_Vx_kk(vx, kk),
            (0x5, _, _, 0x0)     => self.execute_SE_Vx_Vy(vx, vy),
            (0x6, _, _, _)       => self.execute_LD_Vx_kk(vx, kk),
            (0x7, _, _, _)       => self.execute_ADD_Vx_kk(vx, kk),
            (0x8, _, _, 0x0)     => self.execute_LD_Vx_Vy(vx, vy),
            (0x8, _, _, 0x1)     => self.execute_OR_Vx_Vy(vx, vy),
            (0x8, _, _, 0x2)     => self.execute_AND_Vx_Vy(vx, vy),
            (0x8, _, _, 0x3)     => self.execute_XOR_Vx_Vy(vx, vy),
            (0x8, _, _, 0x4)     => self.execute_ADD_Vx_Vy(vx, vy),
            (0x8, _, _, 0x5)     => self.execute_SUB_Vx_Vy(vx, vy),
            (0x8, _, _, 0x6)     => self.execute_SHR_Vx(vx),
            (0x8, _, _, 0x7)     => self.execute_SUBN_Vx_Vy(vx, vy),
            (0x8, _, _, 0xE)     => self.execute_SHL_Vx(vx),
            (0x9, _, _, 0x0)     => self.execute_SNE_Vx_Vy(vx, vy),
            (0xA, _, _, _)       => self.execute_LD_I_addr(nnn),
            (0xB, _, _, _)       => self.execute_JP_V0_addr(nnn),
            (0xC, _, _, _)       => self.execute_RND_Vx_kk(vx, kk),
            (0xD, _, _, _)       => self.execute_DRW_Vx_Vy_n(vx, vy, n),
            (0xE, _, 0x9, 0xE)   => self.execute_SKP_Vx(vx),
            (0xE, _, 0xA, 0x1)   => self.execute_SKNP_Vx(vx),
            (0xF, _, 0x0, 0x7)   => self.execute_LD_Vx_DT(vx),
            (0xF, _, 0x0, 0xA)   => self.execute_LD_Vx_K(vx),
            (0xF, _, 0x1, 0x5)   => self.execute_LD_DT_Vx(vx),
            (0xF, _, 0x1, 0x8)   => self.execute_LD_ST_Vx(vx),
            (0xF, _, 0x1, 0xE)   => self.execute_ADD_I_Vx(vx),
            (0xF, _, 0x2, 0x9)   => self.execute_LD_F_Vx(vx),
            (0xF, _, 0x3, 0x3)   => self.execute_LD_B_Vx(vx),
            (0xF, _, 0x5, 0x5)   => self.execute_LD_I_Vx(vx),
            (0xF, _, 0x6, 0x5)   => self.execute_LD_Vx_I(vx),
            _                    => ProgramCounter::Next,
        };

        match new_pc {
            ProgramCounter::Next => self.pc + 2,
            ProgramCounter::Skip => self.pc + 4,
            ProgramCounter::Address(addr) => addr,
        }
    }

    fn execute_CLS(&mut self) -> ProgramCounter {
        self.clear_display();
        ProgramCounter::Next
    }

    fn execute_RET(&mut self) -> ProgramCounter {
        self.sp -= 1;
        let addr = self.stack[self.sp];
        ProgramCounter::Address(addr)
    }

    fn execute_JP_addr(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Address(nnn)
    }

    fn execute_CALL_addr(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        ProgramCounter::Address(nnn)
    }

    fn execute_SE_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] == kk)
    }

    fn execute_SNE_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] != kk)
    }

    fn execute_SE_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] == self.v_reg[vy])
    }

    fn execute_LD_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        self.v_reg[vx] = kk;
        ProgramCounter::Next
    }

    fn execute_ADD_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        let val = self.v_reg[vx] as u16;
        let kk = kk as u16;
        self.v_reg[vx] = (val + kk) as u8;
        ProgramCounter::Next
    }

    fn execute_LD_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] = self.v_reg[vy];
        ProgramCounter::Next
    }

    fn execute_OR_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] |= self.v_reg[vy];
        ProgramCounter::Next
    }

    fn execute_AND_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] &= self.v_reg[vy];
        ProgramCounter::Next
    }

    fn execute_XOR_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] ^= self.v_reg[vy];
        ProgramCounter::Next
    }

    fn execute_ADD_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        let (v, of) = self.v_reg[vx].overflowing_add(self.v_reg[vy]);
        self.v_reg[vx] = v;
        self.v_reg[0xF] = of as u8;
        ProgramCounter::Next
    }

    fn execute_SUB_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        let (v, of) = self.v_reg[vx].overflowing_sub(self.v_reg[vy]);
        self.v_reg[vx] = v;
        self.v_reg[0xF] = !of as u8;
        ProgramCounter::Next
    }

    fn execute_SHR_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.v_reg[0xF] = self.v_reg[vx] & 0x1;
        self.v_reg[vx] >>= 1;
        ProgramCounter::Next
    }

    fn execute_SUBN_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        let (v, of) = self.v_reg[vy].overflowing_sub(self.v_reg[vx]);
        self.v_reg[vx] = v;
        self.v_reg[0xF] = !of as u8;
        ProgramCounter::Next
    }

    fn execute_SHL_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.v_reg[0xF] = self.v_reg[vx] & 0x80;
        self.v_reg[vx] <<= 1;
        ProgramCounter::Next
    }

    fn execute_SNE_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] != self.v_reg[vy])
    }

    fn execute_LD_I_addr(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    fn execute_JP_V0_addr(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Address(nnn + (self.v_reg[0x0] as usize))
    }

    fn execute_RND_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        self.v_reg[vx] = kk & rand::random::<u8>();
        ProgramCounter::Next
    }

    fn execute_DRW_Vx_Vy_n(&mut self, vx: usize, vy: usize, n: usize) -> ProgramCounter {
        let mut collision = false;

        for row in 0..n {
            let dy = (row + vy) % DISPLAY_HEIGHT;
            for col in 0..8 {
                let dx = (col + vx) % DISPLAY_WIDTH;
                let color = (self.mem[self.i + row] >> (7 - col)) & 1;
                collision |= (color & self.display[dy * DISPLAY_WIDTH + dx]) == 1;
                self.display[dy * DISPLAY_WIDTH + dx] ^= color;
            }
        }

        self.v_reg[0xF] = collision as u8;
        self.draw = true;

        ProgramCounter::Next
    }

    fn execute_SKP_Vx(&mut self, vx: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.keypad[self.v_reg[vx] as usize] == 1)
    }

    fn execute_SKNP_Vx(&mut self, vx: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.keypad[self.v_reg[vx] as usize] == 0)
    }

    fn execute_LD_Vx_DT(&mut self, vx: usize) -> ProgramCounter {
        self.v_reg[vx] = self.delay_t;
        ProgramCounter::Next
    }

    fn execute_LD_Vx_K(&mut self, vx: usize) -> ProgramCounter {
        if let Some(i) = self.key_pressed() {
            self.v_reg[vx] = i as u8;
            ProgramCounter::Next
        } else {
            ProgramCounter::Address(self.pc)
        }
    }

    fn execute_LD_DT_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.delay_t = self.v_reg[vx];
        ProgramCounter::Next
    }

    fn execute_LD_ST_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.sound_t = self.v_reg[vx];
        ProgramCounter::Next
    }

    fn execute_ADD_I_Vx(&mut self, vx: usize) -> ProgramCounter {
        let v = self.i + self.v_reg[vx] as usize;
        self.v_reg[0xF] = (v > 0xFFF) as u8;
        self.i = v;
        ProgramCounter::Next
    }

    fn execute_LD_F_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.i = (self.v_reg[vx] as usize) * SPRITE_SIZE;
        ProgramCounter::Next
    }

    fn execute_LD_B_Vx(&mut self, vx: usize) -> ProgramCounter {
        let value_x = self.v_reg[vx];
        self.mem[self.i] = value_x / 100;
        self.mem[self.i + 1] = (value_x % 100) / 10;
        self.mem[self.i + 2] = value_x % 10;
        ProgramCounter::Next
    }

    fn execute_LD_I_Vx(&mut self, vx: usize) -> ProgramCounter {
        (0..=vx).for_each(|x| {
            self.mem[self.i + x] = self.v_reg[x];
        });
        ProgramCounter::Next
    }

    fn execute_LD_Vx_I(&mut self, vx: usize) -> ProgramCounter {
        (0..=vx).for_each(|i| {
            self.v_reg[i] = self.mem[self.i + i];
        });
        ProgramCounter::Next
    }

    fn key_pressed(&self) -> Option<usize> {
        self.keypad.iter().position(|&k| k == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ROM: [u8; 20] = [
        0x61, 0x01,  // Sets V1 to 0x1
        0x71, 0x01,  // V1 = V1 + 0x1
        0x31, 0x00,  // Skips next instruction if V1 == 0x0
        0x12, 0x02,  // PC = 0x0202
        0x61, 0x01,  // Sets V1 to 0x1
        0x62, 0xFF,  // Sets V2 to 0xFF
        0x81, 0x24,  // Sets V1 to V1 + V2. VF should be set to 0x1
        0xB2, 0x12,  // PC = V0 + 0x212
        0xC2, 0x30,  // V2 = rand byte AND 0x30 (should be skipped because of last instruction)
        0xFF, 0x1E,  // I = I + VF (should be 0x1)
    ];

    #[test]
    fn test_new_chip8() {
        let mut c = Chip8::new(TEST_ROM.to_vec());

        c.initialize();

        assert_eq!(0x200, c.pc);
        assert_eq!(0x61, c.mem[c.pc as usize]);

        c.fetch_decode_execute();
        assert_eq!(0x202, c.pc);
        assert_eq!(0x1, c.v_reg[0x1]);

        for i in 0..=253 {
            c.fetch_decode_execute();
            assert_eq!(0x204, c.pc);
            assert_eq!(0x2 + (i as u8), c.v_reg[0x1]);

            c.fetch_decode_execute();
            assert_eq!(0x206, c.pc);

            c.fetch_decode_execute();
            assert_eq!(c.pc, 0x202);
        }

        c.fetch_decode_execute();
        assert_eq!(0x204, c.pc);
        assert_eq!(0x0, c.v_reg[0x1]);

        c.fetch_decode_execute();
        assert_eq!(0x208, c.pc);

        c.fetch_decode_execute();
        assert_eq!(0x20A, c.pc);
        assert_eq!(0x1, c.v_reg[0x1]);

        c.fetch_decode_execute();
        assert_eq!(0x20C, c.pc);
        assert_eq!(0xFF, c.v_reg[0x2]);

        c.fetch_decode_execute();
        assert_eq!(0x20E, c.pc);
        assert_eq!(0x0, c.v_reg[0x1]);
        assert_eq!(0xFF, c.v_reg[0x2]);
        assert_eq!(0x1, c.v_reg[0xF]);
    }
}
