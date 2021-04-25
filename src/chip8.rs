const DISPLAY_WIDTH: usize  = 64;
const DISPLAY_HEIGHT: usize = 32;

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
        Chip8 {
            mem: vec![0_u8; 4096],
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

        for i in 0..16 { self.v_reg[i] = 0; }
        for i in 0..16 { self.keypad[i] = 0; }
        for i in 0..16 { self.stack[i] = 0; }
    }

    pub fn clear_display(&mut self) {
        for i in 0..DISPLAY_WIDTH*DISPLAY_HEIGHT {
            self.display[i] = 0;
        }
    }

    // -------------------------------------------------
    // Fetch, Decode and Execute
    pub fn fetch_decode_and_execute(&mut self) {
        let opcode = self.fetch_opcode();
        self.pc += 2;

        self.execute_instruction(opcode);
    }

    fn fetch_opcode(&self) -> u16 {
        (self.mem[self.pc as usize] as u16) << 8 |
            (self.mem[self.pc as usize + 1] as u16)
    }

    // -------------------------------------------------
    // Stack operations
    fn push(&mut self, v: u16) {
        self.stack[self.sp as usize] = v;
        self.sp -= 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp += 1;
        self.stack[self.sp as usize]
    }
}
