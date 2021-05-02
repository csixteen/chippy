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

pub(super) const RESERVED_MEMORY_SIZE: usize = 512;
pub const ROM_SIZE: usize = 3584;

pub(super) trait AddressSpace {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, value: u8, addr: u16);

    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) << 8 |
            (self.read_byte(addr+1) as u16)
    }
}

#[derive(Default)]
pub(super) struct Memory {
    reserved: ReservedMemory,
    rom: Rom
}

impl Memory {
    pub fn new(rom: [u8; ROM_SIZE]) -> Self {
        Memory {
            reserved: ReservedMemory::new(),
            rom: Rom(rom)
        }
    }
}

impl AddressSpace for Memory {
    fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0x0..=0x1FF => self.reserved.read_byte(addr),
            _           => self.rom.read_byte(addr - RESERVED_MEMORY_SIZE as u16),
        }
    }

    fn write_byte(&mut self, value: u8, addr: u16) {
        match addr {
            0x0..=0x1FF => self.reserved.write_byte(value, addr),
            _           => self.rom.write_byte(value, addr - RESERVED_MEMORY_SIZE as u16),
        }
    }
}

struct ReservedMemory([u8; RESERVED_MEMORY_SIZE]);

impl Default for ReservedMemory {
    fn default() -> Self { ReservedMemory([0_u8; RESERVED_MEMORY_SIZE]) }
}

impl ReservedMemory {
    fn new() -> Self {
        let mut rs = ReservedMemory([0_u8; RESERVED_MEMORY_SIZE]);
        (0..80).for_each(|i| rs.0[i] = FONT_DATA[i]);

        rs
    }
}

impl AddressSpace for ReservedMemory {
    fn read_byte(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn write_byte(&mut self, _value: u8, _addr: u16) {
        panic!("read-only memory")
    }
}

struct Rom([u8; ROM_SIZE]);

impl Default for Rom {
    fn default() -> Self { Rom([0_u8; ROM_SIZE]) }
}

impl AddressSpace for Rom {
    fn read_byte(&self, addr: u16) -> u8 {
        self.0[addr as usize]
    }

    fn write_byte(&mut self, value: u8, addr: u16) {
        self.0[addr as usize] = value;
    }
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
