use crate::chip8::Chip8;

impl Chip8 {
    fn get_reg_x(&self, opcode: u16) -> u8 {
        ((opcode >> 8) & 0xF) as u8
    }

    fn get_reg_y(&self, opcode: u16) -> u8 {
        ((opcode >> 4) & 0xF) as u8
    }

    fn get_8bit_value(&self, opcode: u16) -> u8 {
        (opcode & 0xF) as u8
    }

    pub fn execute_instruction(&mut self, opcode: u16) -> u16 {
        let mut new_pc = self.pc + 2;

        match opcode {
            // CLS
            0x00E0 => self.clear_display(),
            // JP addr
            0x1000..=0x1FFF => {
                let addr: u16 = opcode & 0xFFF;
                new_pc = addr;
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
