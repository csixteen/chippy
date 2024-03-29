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

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::chip8::cpu::{CHIP8_HEIGHT,CHIP8_WIDTH,Display};

const DISPLAY_SCALE: usize = 10;

pub(crate) struct VideoDriver {
    canvas: Canvas<Window>,
}

impl VideoDriver {
    pub fn new(ctx: &sdl2::Sdl) -> Self {
        let video_subsystem = ctx.video().unwrap();
        let window = video_subsystem
            .window(
                "Chippy - CHIP-8 Interpreter",
                (DISPLAY_SCALE * CHIP8_WIDTH) as u32,
                (DISPLAY_SCALE * CHIP8_HEIGHT) as u32,
            )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        VideoDriver { canvas }
    }

    pub fn draw(&mut self, data: &Display) {
        for y in 0..CHIP8_HEIGHT {
            for x in 0..CHIP8_WIDTH {
                self.canvas.set_draw_color(
                    VideoDriver::color(data[(x, y)])
                );

                self.canvas.fill_rect(
                    Rect::new(
                        (x * DISPLAY_SCALE) as i32,
                        (y * DISPLAY_SCALE) as i32,
                        DISPLAY_SCALE as u32,
                        DISPLAY_SCALE as u32
                    )
                )
                .expect("could not fill rect");
            }
        }

        self.canvas.present();
    }

    fn color(v: u8) -> Color {
        if v == 0 { Color::RGB(0, 0, 0) }
        else { Color::RGB(255, 255, 0) }
    }
}
