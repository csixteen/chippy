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

pub(crate) trait AddressSpace {
    fn read_byte(&self, addr: u16) -> u8;
    fn write_byte(&mut self, value: u8, addr: u16);

    fn read_word(&self, addr: u16) -> u16 {
        (self.read_byte(addr) as u16) << 8 |
            (self.read_byte(addr+1) as u16)
    }
}

pub(crate) struct Memory {
    reserved: ReservedMemory,
    rom: Box<dyn AddressSpace>,
}

impl Memory {
    pub fn new(rom: Box<dyn AddressSpace>) -> Self {
        Memory {
            reserved: ReservedMemory::new(),
            rom: rom
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

pub(crate) struct Rom([u8; ROM_SIZE]);

impl Default for Rom {
    fn default() -> Self { Rom([0_u8; ROM_SIZE]) }
}

impl Rom {
    pub fn new(rom: [u8; ROM_SIZE]) -> Self {
        Rom(rom)
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyRom;

    impl AddressSpace for DummyRom {
        fn read_byte(&self, _addr: u16) -> u8 { 0 }
        fn write_byte(&mut self, _value: u8, _addr: u16) {}
    }

    struct TestRom([u8; 20]);

    impl AddressSpace for TestRom {
        fn read_byte(&self, addr: u16) -> u8 {
            self.0[addr as usize]
        }

        fn write_byte(&mut self, value: u8, addr: u16) {
            self.0[addr as usize] = value;
        }
    }

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
    fn test_memory_mapper() {
        let mm = Memory::new(Box::new(TestRom(TEST_ROM)));

        assert_eq!(0x61, mm.read_byte(RESERVED_MEMORY_SIZE as u16));
        assert_eq!(0x6101, mm.read_word(RESERVED_MEMORY_SIZE as u16));
    }

    #[test]
    #[should_panic]
    fn test_write_to_reserved_memory() {
        let mut mm = Memory::new(Box::new(DummyRom));
        mm.write_byte(0xF, 0x0);
    }
}
