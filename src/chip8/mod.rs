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

pub mod cpu;
pub mod mem;
mod opcodes;

use cpu::Cpu;
use mem::{Memory,Rom};

pub(crate) fn new_chip8(rom: [u8; mem::ROM_SIZE]) -> Cpu {
    Cpu::new(Box::new(Memory::new(Box::new(Rom::new(rom)))))
}

#[cfg(test)]
mod tests {
    use super::cpu::Cpu;
    use super::mem::{AddressSpace,Memory,RESERVED_MEMORY_SIZE};

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
        let mut mm = Memory::new(Box::new(TestRom(TEST_ROM)));

        assert_eq!(0x61, mm.read_byte(RESERVED_MEMORY_SIZE as u16));
        assert_eq!(0x6101, mm.read_word(RESERVED_MEMORY_SIZE as u16));

        mm.write_byte(0xF, RESERVED_MEMORY_SIZE as u16);
        assert_eq!(0xF, mm.read_byte(RESERVED_MEMORY_SIZE as u16));
    }

    #[test]
    #[should_panic]
    fn test_write_to_reserved_memory() {
        let mut mm = Memory::new(Box::new(DummyRom));
        mm.write_byte(0xF, 0x0);
    }

    #[test]
    fn test_run_rom() {
        let mut cpu = Cpu::new(Box::new(Memory::new(Box::new(TestRom(TEST_ROM)))));

        assert_eq!(0x200, cpu.pc);
        assert_eq!(0x61, cpu.mem.read_byte(cpu.pc));

        cpu.fetch_decode_execute();
        assert_eq!(0x202, cpu.pc);
        assert_eq!(0x1, cpu.v_reg[0x1]);

        for i in 0..=253 {
            cpu.fetch_decode_execute();
            assert_eq!(0x204, cpu.pc);
            assert_eq!(0x2 + (i as u8), cpu.v_reg[0x1]);

            cpu.fetch_decode_execute();
            assert_eq!(0x206, cpu.pc);

            cpu.fetch_decode_execute();
            assert_eq!(cpu.pc, 0x202);
        }

        cpu.fetch_decode_execute();
        assert_eq!(0x204, cpu.pc);
        assert_eq!(0x0, cpu.v_reg[0x1]);

        cpu.fetch_decode_execute();
        assert_eq!(0x208, cpu.pc);

        cpu.fetch_decode_execute();
        assert_eq!(0x20A, cpu.pc);
        assert_eq!(0x1, cpu.v_reg[0x1]);

        cpu.fetch_decode_execute();
        assert_eq!(0x20C, cpu.pc);
        assert_eq!(0xFF, cpu.v_reg[0x2]);

        cpu.fetch_decode_execute();
        assert_eq!(0x20E, cpu.pc);
        assert_eq!(0x0, cpu.v_reg[0x1]);
        assert_eq!(0xFF, cpu.v_reg[0x2]);
        assert_eq!(0x1, cpu.v_reg[0xF]);
    }
}
