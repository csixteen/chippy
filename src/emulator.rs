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

use crate::chip8::cpu::Cpu;
use crate::drivers::audio::AudioDriver;
use crate::drivers::keyboard::KeyboardDriver;
use crate::drivers::video::VideoDriver;

const SLEEP: u64 = 1;

pub struct Emulator;

impl Emulator {
    pub fn run(rom: Vec<u8>) -> Result<(), String> {
        let sdl_context = sdl2::init()?;

        let mut chip8 = Cpu::new(rom);
        let mut keyboard = KeyboardDriver::new(&sdl_context);
        let mut video = VideoDriver::new(&sdl_context);
        let audio = AudioDriver::new(&sdl_context);

        loop {
            chip8.fetch_decode_execute();

            if chip8.draw {
                video.draw(&chip8.display);
                chip8.draw = false;
            }

            if chip8.beep {
                audio.start_beeping();
            } else {
                audio.stop_beeping();
            }

            if let Err(_) = keyboard.read(&mut chip8.keypad) {
                break;
            }

            thread::sleep(Duration::from_millis(SLEEP));
        }

        Ok(())
    }
}
