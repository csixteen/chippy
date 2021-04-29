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

extern crate sdl2;

use std::mem;
use std::thread;
use std::time::Duration;

use crate::chip8::{
    Chip8,
    CHIP8_HEIGHT,
    CHIP8_WIDTH
};

use sdl2::audio::{AudioCallback, AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;

const DISPLAY_SCALE: usize = 10;
const SLEEP: u64 = 1;

pub struct Emulator {
    chip8: Chip8,
}

impl Emulator {
    pub fn new(rom: Vec<u8>) -> Self {
        Emulator {
            chip8: Chip8::new(rom)
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        let sdl_context = sdl2::init()?;

        let mut canvas = self.init_canvas(&sdl_context)?;
        let audio = self.init_audio(&sdl_context)?;
        let mut event_pump = sdl_context.event_pump()?;

        'running: loop {
            self.chip8.fetch_decode_execute();

            if self.chip8.draw {
                self.draw(&mut canvas);
                self.chip8.draw = false;
            }

            if self.chip8.beep {
                audio.resume();
            } else {
                audio.pause();
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }

            self.update_pressed_keys(&event_pump);

            thread::sleep(Duration::from_millis(SLEEP));
        }

        Ok(())
    }

    fn init_canvas(&self, ctx: &sdl2::Sdl) -> Result<Canvas<sdl2::video::Window>, String> {
        let video_subsystem = ctx.video()?;

        let window = video_subsystem
            .window(
                "Chippy - CHIP-8 Interpreter",
                (DISPLAY_SCALE * CHIP8_WIDTH) as u32,
                (DISPLAY_SCALE * CHIP8_HEIGHT) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Ok(canvas)
    }

    fn init_audio(&self, ctx: &sdl2::Sdl) -> Result<AudioDevice<SquareWave>, String> {
        let audio_subsystem = ctx.audio()?;

        let desired_spec = AudioSpecDesired {
            freq: Some(44_100),
            channels: Some(1), // mono
            samples: None,     // default sample size
        };

        audio_subsystem.open_playback(None, &desired_spec, |spec| {
            // Show obtained AudioSpec
            println!("{:?}", spec);

            // initialize the audio callback
            SquareWave {
                phase_inc: 240.0 / spec.freq as f32,
                phase: 0.0,
                volume: 1.25,
            }
        })
    }

    fn draw(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        for row in 0..CHIP8_HEIGHT {
            for col in 0..CHIP8_WIDTH {
                if self.chip8.pixel_at(row, col) == 0 {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
                } else {
                    canvas.set_draw_color(Color::RGB(255, 255, 0));
                }
                canvas.fill_rect(
                    Rect::new(
                        (col*DISPLAY_SCALE) as i32,
                        (row*DISPLAY_SCALE) as i32,
                        DISPLAY_SCALE as u32,
                        DISPLAY_SCALE as u32
                    )
                )
                .expect("could not fill rect");
            }
        }

        canvas.present();
    }

    fn update_pressed_keys(&mut self, events: &sdl2::EventPump) {
        mem::take(&mut self.chip8.keypad);
        let keys: Vec<Keycode> = events
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
                self.chip8.keypad[i] = true;
            }
        }
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 { self.volume } else { -self.volume };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
