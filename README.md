# chippy

CHIP-8 Interpreter written in Rust.

Most of the code for input (keyboard) and output (audio and display) was shamelessly copied from the examples found in [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) repository.

# Usage

You'll definitely need to install the [SDL2.0 Development Libraries](https://www.libsdl.org/). I'd suggest for you to check [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2)'s README, since it covers scenarios that may be relevant to you. Once you have SDL2 installed, you can simply run:

```bash
$ cargo run -- games/<GAME>
```

You should replace `<GAME>` with whatever game tickles your fancy. The games under `games/` are [public domain](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html).

# Testing

This definitely needs more coverage, but I wrote some simple tests just to give me some confidence.

```bash
$ cargo test
```

# Project structure

This is what you'll find in the box:

```
.
├── bin
│   └── chippy
│       └── main.rs
├── chip8
│   ├── cpu.rs
│   ├── mod.rs
│   └── opcodes.rs
├── drivers
│   ├── audio.rs
│   ├── keyboard.rs
│   ├── mod.rs
│   └── video.rs
├── emulator.rs
└── lib.rs
```

There is nothing super fancy. I tried to keep the code as simple as possible. After having read lots of code from other emulators out there (in several languages), this is the structure that made more sense to me. Inside `chip8` directory, you'll find the actual representation and implementation of the (sort of) CPU. Essentially, this is the piece responsible for executing instructions and storing the state in memory (including display memory), stack or registers. On the other hand, the `emulator` is what turns that state into something visible (or audible). It glues together the CHIP-8 interpreter (emulator, whatever you want to call it) and the drivers for Audio, Display and Input (only Keyboard). Those drivers use the SDL2 bindings for Rust to display the actual graphics, produce sound and capture input from the keyboard.

# Screenshots

![alt text](screenshots/Space_Invaders.png)

# References
- [CHIP-8 Games Pack](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)
- [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Matthew Mikolay's CHIP-8 Technical Reference](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference)
- [Collection of CHIP-8 related documentation](https://github.com/trapexit/chip-8_documentation)
- [Awesome CHIP-8](https://chip-8.github.io/links/)
