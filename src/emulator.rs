use crate::chip8::Chip8;

pub struct Emulator {
    chip8: Chip8,
    rom: Vec<u8>,
    running: bool
}

impl Emulator {
    pub fn new(rom: Vec<u8>) -> Self {
        Emulator {
            chip8: Chip8::new(rom.clone()),
            rom: rom,
            running: false
        }
    }
}
