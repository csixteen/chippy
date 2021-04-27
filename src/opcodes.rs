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

        let vx = self.get_reg_x(opcode);
        let vy = self.get_reg_y(opcode);
        let nnn = opcode & 0xFFF;
        let value = self.get_8bit_value(opcode);

        match opcode {
            // CLS
            0x00E0 => self.clear_display(),
            // RET
            0x00EE => new_pc = self.pop() + 2,
            // JP addr
            0x1000..=0x1FFF => new_pc = nnn,
            // CALL addr
            0x2000..=0x2FFF => {
                self.push(self.pc);
                new_pc = nnn;
            },
            // SE Vx, byte
            0x3000..=0x3FFF => if self.v_reg[vx as usize] == value {
                new_pc += 2;
            },
            // SNE Vx, byte
            0x4000..=0x4FFF => if self.v_reg[vx as usize] != value {
                new_pc += 2;
            },
            // SE Vx, Vy
            0x5000..=0x5FFF if opcode & 0x1 == 0x0 =>
                if self.v_reg[vx as usize] == self.v_reg[vy as usize] {
                    new_pc += 2;
                },
            // LD Vx, byte
            0x6000..=0x6FFF => self.v_reg[vx as usize] = value,
            // ADD Vx, byte
            0x7000..=0x7FFF => {
                self.v_reg[vx as usize] = self.v_reg[vx as usize].wrapping_add(value);
            },
            0x8000..=0x8FFE => {
                let value_x = self.v_reg[vx as usize];
                let value_y = self.v_reg[vy as usize];

                match opcode & 0xF {
                    // LD Vx, Vy
                    0x0 => self.v_reg[vx as usize] = value_y,
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
                        self.v_reg[0xF] = of as u8;
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
            // SNE Vx, Vy
            0x9000..=0x9FFF if opcode & 0x1 == 0x0 =>
                if self.v_reg[vx as usize] != self.v_reg[vy as usize] {
                    new_pc += 2;
                },
            // LD I, addr
            0xA000..=0xAFFF => self.i = nnn,
            // JP V0, addr
            0xB000..=0xBFFF => self.pc = nnn + (self.v_reg[0x0] as u16),
            // RND Vx, byte
            0xC000..=0xCFFF => self.v_reg[vx as usize] = value & rand::random::<u8>(),
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
