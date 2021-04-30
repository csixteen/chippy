use std::mem;

use sdl2::EventPump;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct KeyboardDriver {
    event_pump: EventPump
}

impl KeyboardDriver {
    pub fn new(ctx: &sdl2::Sdl) -> Self {
        KeyboardDriver {
            event_pump: ctx.event_pump().unwrap()
        }
    }

    pub fn read(&mut self, keypad: &mut [bool; 16]) -> Result<(), &str> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => return Err("Finito"),
                _ => {}
            }
        }

        mem::take(keypad);

        let keys: Vec<Keycode> = self.event_pump
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        for key in keys {
            let index = match key {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = index {
                keypad[i] = true;
            }
        }

        Ok(())
    }
}
