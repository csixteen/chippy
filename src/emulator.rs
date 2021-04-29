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

use std::thread;
use std::time::Duration;

use crate::chip8::{
    Chip8,
    DISPLAY_HEIGHT,
    DISPLAY_WIDTH
};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;

const DISPLAY_SCALE: usize = 10;
const SLEEP: u64 = 2;

pub struct Emulator {
    chip8: Chip8,
}

impl Emulator {
    pub fn new(rom: Vec<u8>) -> Self {
        Emulator {
            chip8: Chip8::new(rom.clone())
        }
    }

    fn init_canvas(&self, ctx: &sdl2::Sdl) -> Result<Canvas<sdl2::video::Window>, String> {
        let video_subsystem = ctx.video()?;

        let window = video_subsystem
            .window(
                "Chippy - CHIP-8 Interpreter",
                (DISPLAY_SCALE * DISPLAY_WIDTH) as u32,
                (DISPLAY_SCALE * DISPLAY_HEIGHT) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())?;

        window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.chip8.initialize();

        let sdl_context = sdl2::init()?;

        let mut canvas = self.init_canvas(&sdl_context)?;

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let mut event_pump = sdl_context.event_pump()?;

        'running: loop {
            self.chip8.fetch_decode_execute();

            if self.chip8.draw {
                self.draw(&mut canvas);
                self.chip8.draw = false;
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

            thread::sleep(Duration::from_millis(SLEEP));
        }

        Ok(())
    }

    fn draw(&mut self, canvas: &mut Canvas<sdl2::video::Window>) {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for row in 0..DISPLAY_HEIGHT {
            for col in 0..DISPLAY_WIDTH {
                if self.chip8.pixel_at(row, col) != 0 {
                    canvas.set_draw_color(Color::RGB(255, 255, 0));
                } else {
                    canvas.set_draw_color(Color::RGB(0, 0, 0));
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
}
