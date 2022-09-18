
# CHIP-8 Emulator
This projects implements an emulator of the CHIP-8 virtual-machine/interpreter from 1970, written in Rust. CHIP-8 is essentially an interpreted programming language, designed mainly for games. Programs run on a CHIP-8 virtual machine. The display and GUI elements are written using the *ggez* and *egui* crates. Included in the repository is a number of classic CHIP-8 game ROMs, including Pong, Breakout, Tetris, etc.That can be run using the emulator. *NOTE*: Just like the 1970s original, the emulator is a bit flickery. To keep the emulation bare-bones, and true to the original, I did not fix this.

![A GIF of some Breakout gamplay](/Breakout.gif "GIF")

## Build Instruction
Building the project requires Rust and Cargo, most easily installed using [rustup](https://rustup.rs).
To build and run the project simply run:
```bash
cargo build --release
```
Or to build and run directly, run:
```bash
cargo run --release
```

## Functionality
All of the 35 original CHIP-8 op-codes/instructions are implemented in the emulator. The original CHIP-8 display and keyboard are emulated. The original CHIP-8 is designed to work with a keyboard of 16 keys, one for each hex digit, from 0 to F. These keys are mapped to regular keyboard keys as indicated below.

```
Original               Emulator
1 2 3 C                1 2 3 4
4 5 6 D                Q W E R
7 8 9 E                A S D F
A 0 B F                Z X C V
```

CHIP-8 originally came with a 64x32, monochromatic display. This display is emulated using the *ggez* Rust crate. The emulator also incorporates an in-game menu, that can be activated using the _Enter_ key. The menu allows you to start/stop game execution, and alter the clock speed of the emulation, thus altering the game speed.
