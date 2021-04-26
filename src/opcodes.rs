use crate::chip8::Chip8;

impl Chip8 {
    pub fn execute_instruction(&mut self, opcode: u16) -> u16 {
        let mut new_pc = self.pc + 2;

        match opcode {
            0x00E0 => self.clear_display(),
            0x6000..=0x6FFF => {
                let dst: u8 = ((opcode >> 8) & 0xF) as u8;
                let value: u8 = (opcode & 0xF) as u8;
                self.v_reg[dst as usize] = value;
            },
            _ => (),
        }

        new_pc
    }
}
