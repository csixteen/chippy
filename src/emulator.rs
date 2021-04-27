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

use crate::chip8::{
    Chip8,
    DISPLAY_HEIGHT,
    DISPLAY_WIDTH
};

use sdl2::pixels::Color;

const DISPLAY_SCALE: usize = 10;

pub struct Emulator {
    chip8: Chip8,
    running: bool
}

impl Emulator {
    pub fn new(rom: Vec<u8>) -> Self {
        Emulator {
            chip8: Chip8::new(rom.clone()),
            running: false
        }
    }

    pub fn run(&mut self) -> Result<(), String> {
        self.chip8.initialize();

        let sdl_context = sdl2::init()?;
        let video_subsystem = sdl_context.video()?;

        let window = video_subsystem
            .window(
                "Chippy - CHIP-8 Interpreter",
                (DISPLAY_SCALE * DISPLAY_WIDTH) as u32,
                (DISPLAY_SCALE * DISPLAY_HEIGHT) as u32,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let mut canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()
            .map_err(|e| e.to_string())?;

        println!("Using SDL_Renderer \"{}\"", canvas.info().name);
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        self.running = true;

        //loop {
            // self.chip8.fetch_decode_execute();
        //}

        Ok(())
    }
}
