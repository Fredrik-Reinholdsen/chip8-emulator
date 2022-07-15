/*
 *    Created     - 2022-06-27 10:12:41
 *    Updated     - 2022-06-27 10:12:41
 *    Author      - Fredrik Reinholdsen
 *    Project     - ###################
 *    Description - ###################
 */
use crate::{Chip8Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use rand::Rng;

const PROGRAM_START: u16 = 0x200;
const ETI_START: u16 = 0x600;

// Converts a byte into an array of bits as bools
// Ex: 0xAA -> [true, false, true, false, true, false, true, false]
fn byte_to_bools(b: u8) -> [bool; 8] {
    let mut output = [false; 8];
    for i in 0..8 {
        output[i] = (b >> (7 - i)) & 0x01 != 0;
    }
    output
}

// Struct for the 4kB of RAM that the CPU has
struct Ram {
    data: [u8; 4096],
}

impl Ram {
    pub fn new() -> Self {
        Ram { data: [0; 4096] }
    }
}

// Implement display trait for nice display of the
impl Ram {
    // Prints the current state of the RAM
    // Used for debug
    fn print(&self) {
        println!("RAM:");
        self.data.chunks_exact(8).for_each(|c| {
            println!(
                "\t{:#04X}, {:#04X}, {:#04X}, {:#04X}, {:#04X}, {:#04X}, {:#04X}, {:#04X}",
                c[0], c[1], c[2], c[3], c[4], c[5], c[6], c[7]
            )
        });
    }
}

pub struct Cpu {
    /// General purpose, 8-bit registers
    v: [u8; 16],
    /// 16x16-bit stack
    stack: [u16; 16],
    /// Special timer registers that are decremented
    /// each processor cycle
    timer1: u8,
    timer2: u8,
    /// 4 kB of RAM
    ram: Ram,
    /// Display connected to the CPU
    pub display: Chip8Display,
    /// Program counter register
    /// 16-bit register, effectivley a pointer to the
    /// memory address of the next operand
    pc: u16,
    /// Stack pointer
    sp: u8,
    /// Special 16-bit register
    i: u16,
    /// Keeps track if which cycle the CPU is on
    cycle: u64,
    /// A vector that contains all currently held keys.
    /// Used for CPU instructions that do different things
    /// depending on if a certain key is pressed
    pub pressed_keys: [bool; 16],
}

#[allow(dead_code)]
impl Cpu {
    pub fn new() -> Self {
        Cpu {
            v: [0x00; 16],
            stack: [0_u16; 16],
            timer1: 0x00,
            timer2: 0x00,
            ram: Ram::new(),
            display: Chip8Display::new(),
            pc: 0x00,
            sp: 0x00,
            i: 0x0000,
            cycle: 0,
            pressed_keys: [false; 16],
        }
    }

    // Returns the value ontop of the stack and
    // decrements the stack pointer
    // PANICs if the stack is empty i.e stack pointer is 0
    fn stack_pop(&mut self) -> u16 {
        if self.sp == 0 {
            panic!("Stack underflow!");
        } else {
            let ret_val = self.stack[self.sp as usize];
            self.sp -= 1;
            ret_val
        }
    }

    // Pushes a value onto the stack
    // PANICs if a push is attempted when the stack is full
    fn stack_push(&mut self, val: u16) {
        if self.sp == 16 {
            panic!("Stack overflow!");
        } else {
            self.sp += 1;
            self.stack[self.sp as usize] = val;
        }
    }

    // Prints the current state of the stack
    // Used for debug purposes only
    pub fn stack_print(&self) {
        println!("Stack:");
        println!("\tStack Pointer: {:#04X}", self.sp);
        println!("\tData:");
        for i in 0..4 {
            println!(
                "\t\t{:#06X}, {:#06X}, {:#06X}, {:#06X}",
                self.stack[i * 4],
                self.stack[i * 4 + 1],
                self.stack[i * 4 + 2],
                self.stack[i * 4 + 3],
            );
        }
    }


    // Prints the current state of the RAM
    // USED FOR DEBUG ONLY
    pub fn ram_print(&self) {
        self.ram.print();
    }

    // Main function of the CPU
    // Executes a clock cycles, and executing instructions
    pub fn tick(&mut self) {
        // Decrement timer registers with wrapping
        self.timer1 = self.timer1.overflowing_sub(1).0;
        self.timer2 = self.timer2.overflowing_sub(1).0;
        self.cycle += 1;
    }
    // Implement CPU instructions

    fn sys(&mut self, _addr: u16) {
        panic!("Not implemented on modern Chip-8 interpreters");
    }

    // Clears the display
    fn cls(&mut self) {
        self.display.clear();
        self.pc += 1;
    }

    // Sets the program counter to the address value ontop of
    // the stack, and then decrement the stack pointer
    fn ret(&mut self) {
        self.pc = self.stack_pop();
    }

    // Jump instruction
    // Sets the program counter equal to the provided address
    fn jmp(&mut self, addrs: u16) {
        self.pc = addrs;
    }

    fn call(&mut self, addrs: u16) {
        self.stack_push(self.pc);
        self.pc = addrs;
    }

    // Skips the next instruction if Vx == kk
    fn se(&mut self, vx: u8, byte: u8) {
        if self.v[vx as usize] == byte {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }

    // Skips the next instruction if Vx != kk
    fn sne(&mut self, vx: u8, byte: u8) {
        if self.v[vx as usize] != byte {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }

    // Skips the nex instruction if Vx == Vy
    fn sexy(&mut self, vx: u8, vy: u8) {
        if self.v[vx as usize] == self.v[vy as usize] {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }

    // Puts value into register Vx
    fn ld(&mut self, vx: u8, byte: u8) {
        self.v[vx as usize] = byte;
        self.pc += 1;
    }

    // Adds kk to the register Vx
    fn add(&mut self, vx: u8, byte: u8) {
        self.v[vx as usize] = self.v[vx as usize].overflowing_add(byte).0;
        self.pc += 1;
    }

    // Sets Vx = Vy
    fn ldxy(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] = self.v[vy as usize];
        self.pc += 1;
    }

    // Sets Vx = Vx OR Vy
    fn or(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] |= self.v[vy as usize];
        self.pc += 1;
    }

    // Sets Vx = Vx AND Vy
    fn and(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] &= self.v[vy as usize];
        self.pc += 1;
    }

    // Sets Vx = Vx XOR Vy
    fn xor(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] ^= self.v[vy as usize];
        self.pc += 1;
    }

    // Adds Vy to Vx. If overflow occurs Vf is set to 1
    // to indicate carry
    fn adc(&mut self, vx: u8, vy: u8) {
        let (ret, carry) = self.v[vx as usize].overflowing_add(self.v[vy as usize]);
        self.v[vx as usize] = ret;
        if carry {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.pc += 1;
    }

    // Subtracts Vy to Vx. If overflow occurs Vf is set to 1
    // to indicate carry
    fn sub(&mut self, vx: u8, vy: u8) {
        let (ret, carry) = self.v[vx as usize].overflowing_sub(self.v[vy as usize]);
        self.v[vx as usize] = ret;
        if carry {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.pc += 1;
    }

    // The least significant bit of Vx is stored in Vf
    // and Vx is then right-shifted by 1 (divided by 2)
    fn shr(&mut self, vx: u8) {
        self.v[0xF] = self.v[vx as usize] & 0x01;
        self.v[vx as usize] = self.v[vx as usize] >> 1;
        self.pc += 1;
    }

    // Subtracts Vx from Vy. If overflow occurs Vf is set to 1
    // to indicate carry. Result is stored in Vx
    fn subn(&mut self, vx: u8, vy: u8) {
        let (ret, carry) = self.v[vy as usize].overflowing_sub(self.v[vx as usize]);
        self.v[vx as usize] = ret;
        if carry {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.pc += 1;
    }

    // The significant bit of Vx is stored in Vf
    // and Vx is then left-shifted by 1 (multiplied by 2)
    fn shl(&mut self, vx: u8) {
        self.v[0xF] = self.v[vx as usize] & 0x80;
        self.v[vx as usize] = self.v[vx as usize] << 1;
        self.pc += 1;
    }

    // Skips the next instruction if Vx != Vy
    fn snexy(&mut self, vx: u8, vy: u8) {
        if self.v[vx as usize] != self.v[vy as usize] {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }

    // Loads the 16-bit address value into the I register
    fn ldi(&mut self, addrs: u16) {
        self.i = addrs;
        self.pc += 1;
    }

    // Sets PC to the provided address + V0
    fn jpv0(&mut self, addrs: u16) {
        self.pc = addrs + self.v[0x0] as u16;
    }

    // Set Vx to a random byte AND:ed with the provided byte kk
    fn rnd(&mut self, vx: u8, byte: u8) {
        let mut rng = rand::thread_rng();
        self.v[vx as usize] = rng.gen::<u8>() & byte;
        self.pc += 1;
    }

    // Reads n and n-byte sprite from memory starting from the
    // address stored in register I, and XORing it to the screen
    // starting from coordinates (Vx, Vy).
    // Sprites that crosses the edge screen will be wrapped to the over side
    fn drw(&mut self, vx: u8, vy: u8, n: u8) {
        if n > 15 {
            panic!("Invalid operation, maximum sprite size is 15!");
        }
        // Flag used to indicate if any pixels on
        // the screen are overwritten
        let mut flag: bool = false;
        for (i, byte) in (0..n).into_iter().enumerate() {
            // Wrap y-cordinate if sprite goes off screen
            let y = if self.v[vy as usize] as usize + i > DISPLAY_HEIGHT {
                self.v[vy as usize] as usize + i - DISPLAY_HEIGHT
            } else {
                self.v[vy as usize] as usize + i
            };
            for (j, bit) in byte_to_bools(byte).iter().enumerate() {
                // Wrap x-coordinate if it goes off screen
                let x = if self.v[vx as usize] as usize + j > DISPLAY_WIDTH {
                    self.v[vx as usize] as usize + j - DISPLAY_WIDTH
                } else {
                    self.v[vx as usize] as usize + j
                };
                // Set the flag to true if XORing true and true
                if self.display.screen[y][x] && *bit {
                    flag = true;
                }
                self.display.screen[y][x] ^= bit;
            }
        }
        // If pixel is overwritten, set the Vf register to 1, else 0
        if flag {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.pc += 1;
    }

    // Skips the next instruction if the specified key is currently held
    fn skp(&mut self, key: u8) {
        if self.pressed_keys[key as usize] {
            self.pc += 2;
        } else {
            self.pc += 1;
        }
    }
}
