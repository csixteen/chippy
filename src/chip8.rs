const DISPLAY_WIDTH: usize  = 64;
const DISPLAY_HEIGHT: usize = 32;

struct Chip8 {
    mem: [u8; 4096],
    stack: [u16; 16], // 16-level Stack

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

    display: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
}

impl Default for Chip8 {
    fn default() -> Chip8 {
        Chip8 {
            mem: [0_u8; 4096],
            display: [0_u8; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            ..Default::default()
        }
    }
}

impl Chip8 {
    pub fn new() -> Self {
        Default::default()
    }
}
