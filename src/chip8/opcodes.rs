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

use super::cpu::{
    CHIP8_HEIGHT,
    CHIP8_WIDTH,
    Cpu,
    ProgramCounter
};

const SPRITE_SIZE: usize = 5;  // size in bytes

impl Cpu {
    // 00E0 - CLS
    // Clear the display.
    pub(super) fn execute_CLS(&mut self) -> ProgramCounter {
        mem::take(&mut self.display);
        self.draw = true;
        ProgramCounter::Next
    }

    // 00EE - RET
    // Return from a subroutine.
    pub(super) fn execute_RET(&mut self) -> ProgramCounter {
        self.sp -= 1;
        let addr = self.stack[self.sp];
        ProgramCounter::Address(addr)
    }

    // 1nnn - JP addr
    // Jump to location nnn.
    pub(super) fn execute_JP_addr(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Address(nnn)
    }

    // 2nnn - CALL addr
    // Call subroutine at nnn.
    pub(super) fn execute_CALL_addr(&mut self, nnn: usize) -> ProgramCounter {
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        ProgramCounter::Address(nnn)
    }

    // 3xkk - SE Vx, byte
    // Skip next instruction if Vx = kk.
    pub(super) fn execute_SE_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] == kk)
    }

    // 4xkk - SNE Vx, byte
    // Skip next instruction if Vx != kk.
    pub(super) fn execute_SNE_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] != kk)
    }

    // 5xy0 - SE Vx, Vy
    // Skip next instruction if Vx = Vy.
    pub(super) fn execute_SE_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] == self.v_reg[vy])
    }

    // 6xkk - LD Vx, byte
    // Set Vx = kk.
    pub(super) fn execute_LD_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        self.v_reg[vx] = kk;
        ProgramCounter::Next
    }

    // 7xkk - ADD Vx, byte
    // Set Vx = Vx + kk.
    pub(super) fn execute_ADD_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        let val = self.v_reg[vx] as u16;
        let kk = kk as u16;
        self.v_reg[vx] = (val + kk) as u8;
        ProgramCounter::Next
    }

    // 8xy0 - LD Vx, Vy
    // Set Vx = Vy.
    pub(super) fn execute_LD_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] = self.v_reg[vy];
        ProgramCounter::Next
    }

    // 8xy1 - OR Vx, Vy
    // Set Vx = Vx OR Vy.
    pub(super) fn execute_OR_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] |= self.v_reg[vy];
        ProgramCounter::Next
    }

    // 8xy2 - AND Vx, Vy
    // Set Vx = Vx AND Vy.
    pub(super) fn execute_AND_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] &= self.v_reg[vy];
        ProgramCounter::Next
    }

    // 8xy3 - XOR Vx, Vy
    // Set Vx = Vx XOR Vy.
    pub(super) fn execute_XOR_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[vx] ^= self.v_reg[vy];
        ProgramCounter::Next
    }

    // 8xy4 - ADD Vx, Vy
    // Set Vx = Vx + Vy, set VF = carry.
    pub(super) fn execute_ADD_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        let x = self.v_reg[vx] as u16;
        let y = self.v_reg[vy] as u16;
        let result = x + y;
        self.v_reg[vx] = result as u8;
        self.v_reg[0xF] = if result > 0xFF { 1 } else { 0 };
        ProgramCounter::Next
    }

    // 8xy5 - SUB Vx, Vy
    // Set Vx = Vx - Vy, set VF = NOT borrow.
    pub(super) fn execute_SUB_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[0xF] = if self.v_reg[vx] > self.v_reg[vy] { 1 } else { 0 };
        self.v_reg[vx] = self.v_reg[vx].wrapping_sub(self.v_reg[vy]);
        ProgramCounter::Next
    }

    // 8xy6 - SHR Vx {, Vy}
    // Set Vx = Vx SHR 1.
    pub(super) fn execute_SHR_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.v_reg[0xF] = self.v_reg[vx] & 0x1;
        self.v_reg[vx] >>= 1;
        ProgramCounter::Next
    }

    // 8xy7 - SUBN Vx, Vy
    // Set Vx = Vy - Vx, set VF = NOT borrow.
    pub(super) fn execute_SUBN_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        self.v_reg[0xF] = if self.v_reg[vy] > self.v_reg[vx] { 1 } else { 0 };
        self.v_reg[vx] = self.v_reg[vy].wrapping_sub(self.v_reg[vx]);
        ProgramCounter::Next
    }

    // 8xyE - SHL Vx {, Vy}
    // Set Vx = Vx SHL 1.
    pub(super) fn execute_SHL_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.v_reg[0xF] = (self.v_reg[vx] & 0x80) >> 7;
        self.v_reg[vx] <<= 1;
        ProgramCounter::Next
    }

    // 9xy0 - SNE Vx, Vy
    // Skip next instruction if Vx != Vy.
    pub(super) fn execute_SNE_Vx_Vy(&mut self, vx: usize, vy: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.v_reg[vx] != self.v_reg[vy])
    }

    // Annn - LD I, addr
    // Set I = nnn.
    pub(super) fn execute_LD_I_addr(&mut self, nnn: usize) -> ProgramCounter {
        self.i = nnn;
        ProgramCounter::Next
    }

    // Bnnn - JP V0, addr
    // Jump to location nnn + V0.
    pub(super) fn execute_JP_V0_addr(&mut self, nnn: usize) -> ProgramCounter {
        ProgramCounter::Address(nnn + (self.v_reg[0x0] as usize))
    }

    // Cxkk - RND Vx, byte
    // Set Vx = random byte AND kk.
    pub(super) fn execute_RND_Vx_kk(&mut self, vx: usize, kk: u8) -> ProgramCounter {
        self.v_reg[vx] = kk & rand::random::<u8>();
        ProgramCounter::Next
    }

    // Dxyn - DRW Vx, Vy, nibble
    // Display n-byte sprite starting at memory location I
    // at (Vx, Vy), set VF = collision.
    pub(super) fn execute_DRW_Vx_Vy_n(&mut self, vx: usize, vy: usize, n: usize) -> ProgramCounter {
        self.v_reg[0xF] = 0x0;

        for row in 0..n {
            for col in 0..8 {
                let dx = (col + self.v_reg[vx] as usize) % CHIP8_WIDTH;
                let dy = (row + self.v_reg[vy] as usize) % CHIP8_HEIGHT;
                let color = (self.mem[self.i + row] >> (7 - col)) & 1;
                self.v_reg[0xF] |= color & self.display.0[dy * CHIP8_WIDTH + dx];
                self.display.0[dy * CHIP8_WIDTH + dx] ^= color;
            }
        }

        self.draw = true;

        ProgramCounter::Next
    }

    // Ex9E - SKP Vx
    // Skip next instruction if key with the value of Vx is pressed.
    pub(super) fn execute_SKP_Vx(&mut self, vx: usize) -> ProgramCounter {
        ProgramCounter::skip_if(self.keypad[self.v_reg[vx] as usize])
    }

    // ExA1 - SKNP Vx
    // Skip next instruction if key with the value of Vx is not pressed.
    pub(super) fn execute_SKNP_Vx(&mut self, vx: usize) -> ProgramCounter {
        ProgramCounter::skip_if(!self.keypad[self.v_reg[vx] as usize])
    }

    // Fx07 - LD Vx, DT
    // Set Vx = delay timer value.
    pub(super) fn execute_LD_Vx_DT(&mut self, vx: usize) -> ProgramCounter {
        self.v_reg[vx] = self.delay_t;
        ProgramCounter::Next
    }

    // Fx0A - LD Vx, K
    // Wait for a key press, store the value of the key in Vx.
    pub(super) fn execute_LD_Vx_K(&mut self, vx: usize) -> ProgramCounter {
        if let Some(i) = self.is_key_pressed() {
            self.v_reg[vx] = i as u8;
            ProgramCounter::Next
        } else {
            ProgramCounter::Address(self.pc)
        }
    }

    // Fx15 - LD DT, Vx
    // Set delay timer = Vx.
    pub(super) fn execute_LD_DT_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.delay_t = self.v_reg[vx];
        ProgramCounter::Next
    }

    // Fx18 - LD ST, Vx
    // Set sound timer = Vx.
    pub(super) fn execute_LD_ST_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.sound_t = self.v_reg[vx];
        ProgramCounter::Next
    }

    // Fx1E - ADD I, Vx
    // Set I = I + Vx.
    pub(super) fn execute_ADD_I_Vx(&mut self, vx: usize) -> ProgramCounter {
        let v = self.i + (self.v_reg[vx] as usize);
        self.v_reg[0xF] = (v > 0xF00) as u8;
        self.i = v;
        ProgramCounter::Next
    }

    // Fx29 - LD F, Vx
    // Set I = location of sprite for digit Vx.
    pub(super) fn execute_LD_F_Vx(&mut self, vx: usize) -> ProgramCounter {
        self.i = (self.v_reg[vx] as usize) * SPRITE_SIZE;
        ProgramCounter::Next
    }

    // Fx33 - LD B, Vx
    // Store BCD representation of Vx in memory locations I, I+1, and I+2.
    pub(super) fn execute_LD_B_Vx(&mut self, vx: usize) -> ProgramCounter {
        let value_x = self.v_reg[vx];
        self.mem[self.i] = value_x / 100;
        self.mem[self.i + 1] = (value_x % 100) / 10;
        self.mem[self.i + 2] = value_x % 10;
        ProgramCounter::Next
    }

    // Fx55 - LD [I], Vx
    // Store registers V0 through Vx in memory starting at location I.
    pub(super) fn execute_LD_I_Vx(&mut self, vx: usize) -> ProgramCounter {
        (0..=vx).for_each(|i| {
            self.mem[self.i + i] = self.v_reg[i];
        });
        ProgramCounter::Next
    }

    // Fx65 - LD Vx, [I]
    // Read registers V0 through Vx from memory starting at location I.
    pub(super) fn execute_LD_Vx_I(&mut self, vx: usize) -> ProgramCounter {
        (0..=vx).for_each(|i| {
            self.v_reg[i] = self.mem[self.i + i];
        });
        ProgramCounter::Next
    }
}
