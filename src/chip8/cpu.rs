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

use super::mem::{AddressSpace, RESERVED_MEMORY_SIZE};

pub(crate) const CHIP8_WIDTH: usize  = 64;
pub(crate) const CHIP8_HEIGHT: usize = 32;
const STACK_SIZE: usize              = 16;

/// Indicates how the Program Counter will change after a certain
/// instruction is executed: it may advance to the next instruction,
/// it may skip to the next instruction or it may jump to a specific
/// address.
pub(super) enum ProgramCounter {
    Next,
    Skip,
    Address(u16),
}

impl ProgramCounter {
    /// If the condition is true, then the Program Counter will
    /// skip the next instruction, otherwise it won't.
    pub(super) fn skip_if(cond: bool) -> ProgramCounter {
        if cond { ProgramCounter::Skip }
        else { ProgramCounter::Next }
    }
}

/// Represents the 64x32 monochrome display. Individual pixels are accessed
/// by indexing the Display using a tuple (x, y).
pub(crate) struct Display([u8; CHIP8_HEIGHT * CHIP8_WIDTH]);

impl Default for Display {
    fn default() -> Self {
        Display([0_u8; CHIP8_HEIGHT * CHIP8_WIDTH])
    }
}

impl Index<(usize, usize)> for Display {
    type Output = u8;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.1 * CHIP8_WIDTH + index.0]
    }
}

impl IndexMut<(usize, usize)> for Display {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.1 * CHIP8_WIDTH + index.0]
    }
}

pub struct Cpu {
    pub(super) mem: Box<dyn AddressSpace>,
    /// 16-level stack used to store memory addresses where the interpreter
    /// should return to when a subroutine is complete.
    pub(super) stack: [u16; STACK_SIZE],

    /// 16 8-bit registers from V0 to VF. The register VF shouldn't be
    /// used directly by the applications, as it is used as a flag register
    /// by some instructions.
    pub(super) v_reg: [u8; 16],
    /// 16-bit register used to hold memory addresses.
    pub(super) i: u16,

    /// Delay timer register. The delay timer is active whenever this register
    /// is non-zero. According to the specs, its value should be substracted
    /// by 1 at a rate of 60Hz, until it reaches 0. When this happens, the delay
    /// timer deactivates.
    pub(super) delay_t: u8,
    /// Sound timer register. Its value also decrements at a rate of 60Hz. As long
    /// as its value is greater than zero, the CHIP-8 buzzer should produce a sound.
    pub(super) sound_t: u8,

    /// The Program Counter holds holds the memory address of
    /// the next instruction to be executed.
    pub(super) pc: u16,
    /// Stack Pointer points to the top of the stack.
    pub(super) sp: usize,

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
    pub(crate) fn new(m: Box<dyn AddressSpace>) -> Self {
        Cpu {
            mem: m,
            stack: [0_u16; STACK_SIZE],
            v_reg: [0_u8; 16],
            i: 0,
            delay_t: 0,
            sound_t: 0,
            pc: RESERVED_MEMORY_SIZE as u16, // Initialize the ProgramCounter at 0x200
            sp: 0,
            keypad: [false; 16],
            display: Display::default(),
            draw: false,
            beep: false
        }
    }

    /// Like the name indicates, it fetches the next instruction to be executed
    /// (located in the address stored in the Program Counter), it decodes the
    /// instruction (parse the operands) and executes it. Since each Opcode is
    /// 2 bytes, the instruction is stored in two adjacent memory addresses:
    /// PC and PC + 1.
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
