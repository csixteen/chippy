use crate::chip8::Chip8;

impl Chip8 {
    pub fn execute_instruction(&mut self, opcode: u16) {
        match opcode {
            0x00E0 => self.clear_display(),
            _ => (),
        }
    }
}
