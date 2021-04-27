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

use std::mem;

const DISPLAY_WIDTH: usize  = 64;
const DISPLAY_HEIGHT: usize = 32;
const ROM_OFFSET: usize     = 0x200;

#[derive(Default)]
pub struct Chip8 {
    mem: Vec<u8>,
    stack: Vec<u16>, // 16-level Stack

    // Registers - the register VF shouldn't be
    // used by programs, as it is used as a flag
    // by some instructions
    v_reg: [u8; 16],  // V0..VF
    i: u16, // used to store memory addresses. Only the lowest 12 bits are used

    // Timers
    delay_t: u8,  // delay timer
    sound_t: u8,  // sound timer

    // Pseudo-registers (not directly accessible to the user)
    pc: u16,  // Program Counter
    sp: u8,   // Stack-Pointer

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
    draw: bool,
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

impl Chip8 {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut mem = vec![0_u8; 4096];

        (0..80).for_each(|i| { mem[i] = FONT_DATA[i]; });
        (0..rom.len()).for_each(|i| { mem[i + ROM_OFFSET] = rom[i]; });

        Chip8 {
            mem: mem,
            stack: vec![0_u16; 16],
            display: vec![0_u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            ..Default::default()
        }
    }

    pub fn initialize(&mut self) {
        self.i = 0;
        self.sp = 0xF;
        self.pc = 0x200;
        self.delay_t = 0;
        self.sound_t = 0;

        mem::take(&mut self.v_reg);
        mem::take(&mut self.keypad);
        mem::take(&mut self.stack);

        self.clear_display();
    }

    pub fn clear_display(&mut self) {
        mem::take(&mut self.display);
    }

    // -------------------------------------------------
    // Fetch, Decode and Execute
    pub fn fetch_decode_execute(&mut self) {
        let opcode = self.fetch_opcode();

        self.pc = self.execute_instruction(opcode);

        if self.delay_t > 0 {
            self.delay_t -= 1;
        }

        if self.sound_t > 0 {
            self.sound_t -= 1;
        }
    }

    fn fetch_opcode(&self) -> u16 {
        (self.mem[self.pc as usize] as u16) << 8 |
            (self.mem[self.pc as usize + 1] as u16)
    }

    fn execute_instruction(&mut self, opcode: u16) -> u16 {
        let mut new_pc = self.pc + 2;

        let vx = self.get_reg_x(opcode);
        let vy = self.get_reg_y(opcode);
        let nnn = opcode & 0xFFF;
        let value = self.get_8bit_value(opcode);
        let nibble = self.get_4bit_value(opcode);

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
            // DRW Vx, Vy, nibble
            0xD000..=0xDFFF => {
                self.v_reg[0xF] = self.draw(vx, vy, nibble) as u8;
                self.draw = true;
            },
            // SKP Vx
            0xE09E..=0xEF9E if value == 0x9E =>
                if self.keypad[self.v_reg[vx as usize] as usize] == 1 {
                    new_pc += 2;
                },
            // SKNP Vx
            0xE0A1..=0xEFA1 if value == 0xA1 =>
                if self.keypad[self.v_reg[vx as usize] as usize] == 0 {
                    new_pc += 2;
                },
            // LD Vx, DT
            0xF007..=0xFF07 if value == 0x07 => self.v_reg[vx as usize] = self.delay_t,
            // LD Vx, K
            0xF00A..=0xFF0A if value == 0x0A =>
                if let Some(i) = self.key_pressed() {
                    self.v_reg[vx as usize] = i as u8;
                } else {
                    new_pc -= 2;
                },
            // LD DT, Vx
            0xF015..=0xFF15 if value == 0x15 => self.delay_t = self.v_reg[vx as usize],
            _ => (),
        }

        new_pc
    }

    // ---------------------------------------------------
    // Operands helpers

    fn get_reg_x(&self, opcode: u16) -> u8 {
        ((opcode >> 8) & 0xF) as u8
    }

    fn get_reg_y(&self, opcode: u16) -> u8 {
        ((opcode >> 4) & 0xF) as u8
    }

    fn get_8bit_value(&self, opcode: u16) -> u8 {
        (opcode & 0xFF) as u8
    }

    fn get_4bit_value(&self, opcode: u16) -> u8 {
        (opcode & 0xF) as u8
    }

    // -----------------------------------------------------
    // Stack manipulation helpers

    fn push(&mut self, v: u16) {
        self.stack[self.sp as usize] = v;
        self.sp -= 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp += 1;
        self.stack[self.sp as usize]
    }

    // -----------------------------------------------------
    // Display helpers
    fn draw(&mut self, vx: u8, vy: u8, n: u8) -> bool {
        let mut collision = false;

        for row in 0..(n as usize) {
            for col in 0..8 {
                let dy = row + (vy as usize) % DISPLAY_HEIGHT;
                let dx = col + (vx as usize) % DISPLAY_WIDTH;
                let old = self.display[dy * DISPLAY_HEIGHT + dx];
                let new = (self.mem[self.i as usize + row] >> col) & 0x1;
                self.display[dy * DISPLAY_HEIGHT + dx] ^= new;
                if (old & new) == 1 {
                    collision = true;
                }
            }
        }

        collision
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
