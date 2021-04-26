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

use crate::chip8::Chip8;

impl Chip8 {
    fn get_reg_x(&self, opcode: u16) -> u8 {
        ((opcode >> 8) & 0xF) as u8
    }

    fn get_reg_y(&self, opcode: u16) -> u8 {
        ((opcode >> 4) & 0xF) as u8
    }

    fn get_8bit_value(&self, opcode: u16) -> u8 {
        (opcode & 0xFF) as u8
    }

    pub(crate) fn execute_instruction(&mut self, opcode: u16) -> u16 {
        let mut new_pc = self.pc + 2;

        match opcode {
            // CLS
            0x00E0 => self.clear_display(),
            // JP addr
            0x1000..=0x1FFF => {
                let addr: u16 = opcode & 0xFFF;
                new_pc = addr;
            },
            // SE Vx, byte
            0x3000..=0x3FFF => {
                let vx = self.get_reg_x(opcode);
                let value = self.get_8bit_value(opcode);
                if self.v_reg[vx as usize] == value {
                    new_pc += 2;
                }
            },
            // LD Vx, byte
            0x6000..=0x6FFF => {
                let dst = self.get_reg_x(opcode);
                let value = self.get_8bit_value(opcode);
                self.v_reg[dst as usize] = value;
            },
            // ADD Vx, byte
            0x7000..=0x7FFF => {
                let dst = self.get_reg_x(opcode);
                let value = self.get_8bit_value(opcode);
                self.v_reg[dst as usize] = self.v_reg[dst as usize].wrapping_add(value);
            },
            _ => (),
        }

        new_pc
    }
}
