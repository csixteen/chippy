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

use std::ops::{Index,IndexMut};

use super::mem::{
    AddressSpace,
    Memory,
    RESERVED_MEMORY_SIZE,
    ROM_SIZE
};

pub(crate) const CHIP8_WIDTH: usize  = 64;
pub(crate) const CHIP8_HEIGHT: usize = 32;
const STACK_SIZE: usize              = 16;

pub(super) enum ProgramCounter {
    Next,
    Skip,
    Address(u16),
}

impl ProgramCounter {
    pub(super) fn skip_if(cond: bool) -> ProgramCounter {
        if cond { ProgramCounter::Skip }
        else { ProgramCounter::Next }
    }
}

pub(crate) struct Display([u8; CHIP8_HEIGHT * CHIP8_WIDTH]);

impl Default for Display {
    fn default() -> Self {
        Display([0_u8; CHIP8_HEIGHT * CHIP8_WIDTH])
    }
}

impl Index<usize> for Display {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for Display {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[derive(Default)]
pub struct Cpu {
    pub(super) mem: Memory,
    pub(super) stack: [u16; STACK_SIZE],

    // Registers - the register VF shouldn't be
    // used by programs, as it is used as a flag
    // by some instructions
    pub(super) v_reg: [u8; 16],
    pub(super) i: u16,

    // Timers
    pub(super) delay_t: u8,  // delay timer
    pub(super) sound_t: u8,  // sound timer

    // Pseudo-registers (not directly accessible to the user)
    pub(super) pc: u16,  // Program Counter
    pub(super) sp: usize,  // Stack-Pointer

    // +---------------+
    // | 1 | 2 | 3 | C |
    // +---+---+---+---+
    // | 4 | 5 | 6 | D |
    // +---+---+---+---+
    // | 7 | 8 | 9 | E |
    // +---+---+---+---+
    // | A | 0 | B | F |
    // +---+---+---+---+
    pub(crate) keypad: [bool; 16],  // 16-key hexadecimal keypad

    pub(crate) display: Display,
    pub(crate) draw: bool,
    pub(crate) beep: bool,
}

impl Cpu {
    pub fn new(rom: [u8; ROM_SIZE]) -> Self {
        Cpu {
            pc: RESERVED_MEMORY_SIZE as u16, // Initialize the ProgramCounter at 0x200
            mem: Memory::new(rom),
            ..Default::default()
        }
    }

    pub fn fetch_decode_execute(&mut self) {
        let opcode = self.mem.read_word(self.pc);

        self.pc = self.execute_instruction(opcode);

        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            self.sound_t -= 1;
        }

        self.beep = self.sound_t > 0;
    }

    fn execute_instruction(&mut self, opcode: u16) -> u16 {
        let parts = (
            ((opcode & 0xF000) >> 12) as usize,
            ((opcode & 0x0F00) >> 8) as usize,
            ((opcode & 0x00F0) >> 4) as usize,
            (opcode & 0x000F) as usize
        );

        let vx = parts.1;
        let vy = parts.2;
        let nnn = opcode & 0xFFF;
        let kk = (opcode & 0xFF) as u8;
        let n = (opcode & 0xF) as usize;

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

    pub(super) fn is_key_pressed(&self) -> Option<usize> {
        self.keypad.iter().position(|&k| k)
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
