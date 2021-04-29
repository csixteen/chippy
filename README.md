# chippy

CHIP-8 Interpreter written in Rust (**THIS IS STILL WORK IN PROGRESS**).

Most of the code for input (keyboard) and output (audio and display) was shamelessly copied from the examples found in [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) repository. The code is still not pretty, especially in `emulator.rs`, but like Kent Beck says: "Make it work. Make it right. Make it fast.". Right now, it sort of works.

# Usage

You'll definitely need to install the [SDL2.0 Development Libraries](https://www.libsdl.org/). I'd suggest for you to check [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2)'s README, since it covers scenarios that may be relevant to you. Once you have SDL2 installed, you can simply run:

```bash
$ cargo run -- games/<GAME>
```

You should replace `<GAME>` with whatever game tickles your fancy.

# Testing

This definitely needs more coverage, but I wrote some simple tests just to give me some confidence.

```bash
$ cargo test
```

# References
- [CHIP-8 Games Pack](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)
- [Cowgod's Chip-8 Technical Reference v1.0](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
- [Matthew Mikolay's CHIP-8 Technical Reference](https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference)
- [Collection of CHIP-8 related documentation](https://github.com/trapexit/chip-8_documentation)
- [Awesome CHIP-8](https://chip-8.github.io/links/)
