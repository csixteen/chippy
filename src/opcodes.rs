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
    pub(crate) fn execute_instruction(&mut self, opcode: u16) -> u16 {
        let mut new_pc = self.pc + 2;

        match opcode {
            // CLS
            0x00E0 => self.clear_display(),
            // RET
            0x00EE => new_pc = self.pop() + 2,
            // JP addr
            0x1000..=0x1FFF => {
                let addr: u16 = opcode & 0xFFF;
                new_pc = addr;
            },
            // CALL addr
            0x2000..=0x2FFF => {
                self.push(self.pc);
                new_pc = opcode & 0x0FFF;
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
                let vx = self.get_reg_x(opcode);
                let value = self.get_8bit_value(opcode);
                self.v_reg[vx as usize] = value;
            },
            // ADD Vx, byte
            0x7000..=0x7FFF => {
                let vx = self.get_reg_x(opcode);
                let value = self.get_8bit_value(opcode);
                self.v_reg[vx as usize] = self.v_reg[vx as usize].wrapping_add(value);
            },
            0x8000..=0x8FFE => {
                let vx = self.get_reg_x(opcode);
                let vy = self.get_reg_y(opcode);
                let value_x = self.v_reg[vx as usize];
                let value_y = self.v_reg[vy as usize];

                match opcode & 0xF {
                    // LD Vx, Vy
                    0x0 => self.v_reg[vx as usize] = self.v_reg[vy as usize],
                    // OR Vx, Vy
                    0x1 => self.v_reg[vx as usize] = value_x | value_y,
                    // AND Vx, Vy
                    0x2 => self.v_reg[vx as usize] = value_x & value_y,
                    // XOR Vx, Vy
                    0x3 => self.v_reg[vx as usize] = value_x ^ value_y,
                    // ADD Vx, Vy
                    0x4 => {
                        let (v, of) = value_x.overflowing_add(value_y);
                        self.v_reg[vx as usize] = v;
                        if of { self.v_reg[0xF] = 1; }
                    },
                    // SUB Vx, Vy
                    0x5 => {
                        let borrow = value_x < value_y;
                        self.v_reg[vx as usize] = value_x.wrapping_sub(value_y);
                        self.v_reg[0xF] = !borrow as u8;
                    },
                    // SHR Vx {, Vy}
                    0x6 => {
                        self.v_reg[0xF] = value_x & 0x1;
                        self.v_reg[vx as usize] = value_x >> 1;
                    },
                    // SUBN Vx, Vy
                    0x7 => {
                        let borrow = value_y < value_x;
                        self.v_reg[vx as usize] = value_y.wrapping_sub(value_x);
                        self.v_reg[0xF] = !borrow as u8;
                    },
                    // SHL Vx {, Vy}
                    0x8 => {
                        self.v_reg[0xF] = value_x & 0x80;
                        self.v_reg[vx as usize] = value_x << 1;
                    },
                    _ => (),
                }
            },
            _ => (),
        }

        new_pc
    }

    fn get_reg_x(&self, opcode: u16) -> u8 {
        ((opcode >> 8) & 0xF) as u8
    }

    fn get_reg_y(&self, opcode: u16) -> u8 {
        ((opcode >> 4) & 0xF) as u8
    }

    fn get_8bit_value(&self, opcode: u16) -> u8 {
        (opcode & 0xFF) as u8
    }

    fn push(&mut self, v: u16) {
        self.stack[self.sp as usize] = v;
        self.sp -= 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp += 1;
        self.stack[self.sp as usize]
    }
}
