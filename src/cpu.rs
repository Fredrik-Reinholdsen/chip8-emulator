/*
 *    Created     - 2022-06-27 10:12:41
 *    Updated     - 2022-06-27 10:12:41
 *    Author      - Fredrik Reinholdsen
 *    Project     - ###################
 *    Description - ###################
 */
use crate::{Chip8Display, DISPLAY_HEIGHT, DISPLAY_WIDTH};
use rand::Rng;
use std::fs::File;
use std::io::Read;

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
        // Initializes the RAM, as all zeros except for 0x00 t0 0x1FF
        // which are initialized to hold sprites for hex digits 0 to F
        let mut data = [0_u8; 4096];
        // Digit 0
        data[0..5].copy_from_slice(&[0xF0, 0x90, 0x90, 0x90, 0xF0]);
        // Digit 1
        data[5..10].copy_from_slice(&[0x20, 0x60, 0x20, 0x20, 0x70]);
        // Digit 2
        data[10..15].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x80, 0xF0]);
        // Digit 3
        data[15..20].copy_from_slice(&[0xF0, 0x10, 0xF0, 0x10, 0xF0]);
        // Digit 4
        data[15..20].copy_from_slice(&[0x90, 0x90, 0xF0, 0x10, 0x10]);
        // Digit 5
        data[20..25].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x10, 0xF0]);
        // Digit 6
        data[25..30].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x90, 0xF0]);
        // Digit 7
        data[30..35].copy_from_slice(&[0xF0, 0x10, 0x20, 0x40, 0x40]);
        // Digit 8
        data[35..40].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0xF0]);
        // Digit 9
        data[40..45].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x10, 0xF0]);
        // Digit A
        data[45..50].copy_from_slice(&[0xF0, 0x90, 0xF0, 0x90, 0x90]);
        // Digit B
        data[50..55].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]);
        // Digit C
        data[55..60].copy_from_slice(&[0xF0, 0x80, 0x80, 0x80, 0xF0]);
        // Digit D
        data[60..65].copy_from_slice(&[0xE0, 0x90, 0xE0, 0x90, 0xE0]);
        // Digit E
        data[70..75].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0xF0]);
        // Digit F
        data[75..80].copy_from_slice(&[0xF0, 0x80, 0xF0, 0x80, 0x80]);
        Ram { data }
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
    /// Delay timer register
    /// Decremented by 1 each cycle
    dt: u8,
    /// Sound timer register
    /// Decremented by 1 each cycle
    st: u8,
    /// 4 kB (4096 bytes) of RAM
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
    pub cycle: u64,
    /// The clock speed of the device in Hz
    clock_speed: f64,
    /// A vector that contains all currently held keys.
    /// Used for CPU instructions that do different things
    /// depending on if a certain key is pressed
    pub pressed_keys: [bool; 16],
    /// Holds CPU execution while true
    /// used for an instruction that holds CPU execution
    /// until a key is pressed
    hold_flag: bool,
    /// Variable that holds the loaded instruction in each cycle
    inst: u16,
}

#[allow(dead_code)]
impl Cpu {
    pub fn new(clock_speed: f64) -> Self {
        Cpu {
            v: [0x00; 16],
            stack: [0_u16; 16],
            dt: 0x00,
            st: 0x00,
            ram: Ram::new(),
            display: Chip8Display::new(),
            pc: PROGRAM_START,
            sp: 0x00,
            i: 0x0000,
            cycle: 0,
            clock_speed,
            pressed_keys: [false; 16],
            hold_flag: false,
            inst: 0x0000,
        }
    }

    fn nnn(opcode: u16) -> u16 {
        opcode & 0x0FFF
    }

    fn x(opcode: u16) -> u8 {
        (opcode & 0x0F00 >> 8) as u8
    }

    fn y(opcode: u16) -> u8 {
        (opcode & 0x00F0 >> 8) as u8
    }

    fn kk(opcode: u16) -> u8 {
        (opcode & 0x00FF) as u8
    }

    pub fn core_dump(&self) {
        println!("ERROR!\n Core dump:\n\tCycles: {}", self.cycle);
        println!("\tStack Pointer: {}", self.sp);
        self.stack_print();
        self.ram_print();
    }
    //Resets the entire CPU to its initial state
    pub fn reset(&mut self) {
        self.v = [0x00; 16];
        self.stack = [0; 16];
        self.dt = 0x00;
        self.sp = 0x00;
        self.pc = PROGRAM_START;
        self.st = 0x00;
        self.i = 0x00;
        self.cycle = 0;
        self.pressed_keys = [false; 16];
        self.hold_flag = false;
        self.display = Chip8Display::new();
        self.ram = Ram::new();
    }

    //Loads a chip 8 ROM into memory and resets the CPU
    pub fn load_rom(&mut self, path: &str) -> std::io::Result<()> {
        self.reset();
        let mut file = File::open(path)?;
        let mut contents: Vec<u8> = vec![];
        file.read_to_end(&mut contents)?;
        self.ram.data[0x200..0x200 + contents.len()].clone_from_slice(&contents);
        Ok(())
    }

    // If any key is pressed Some with the key value is returned
    // else None is retrurned
    fn get_pressed_key(&self) -> Option<usize> {
        self.pressed_keys.iter().position(|&x| x == true)
    }

    // Returns the value ontop of the stack and
    // decrements the stack pointer
    // PANICs if the stack is empty i.e stack pointer is 0
    fn stack_pop(&mut self) -> u16 {
        if self.sp == 0 {
            panic!("Stack underflow!");
        } else {
            self.sp -= 1;
            let ret_val = self.stack[self.sp as usize];
            ret_val
        }
    }

    // Pushes a value onto the stack
    // PANICs if a push is attempted when the stack is full
    fn stack_push(&mut self, val: u16) {
        if self.sp == 16 {
            panic!("Stack overflow!");
        } else {
            self.stack[self.sp as usize] = val;
            self.sp += 1;
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

    // This function is run at a frequency of
    // 60 Hz. A timer is active as long as the timer
    // value is greater than 0
    // while a timer is active it is decremented by 1
    // at a rate of 60 Hz until it deactivates
    pub fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            self.st -= 1;
        }
    }

    // Main function of the CPU
    // Executes a clock cycles, and executing instructions
    pub fn tick(&mut self) {
        // Decrement timer registers with wrapping
        if !self.hold_flag {
            // Get the two insruction bytes
            let inst_hi = self.ram.data[self.pc as usize];
            self.pc += 1;
            let inst_lo = self.ram.data[self.pc as usize];
            self.pc += 1;
            self.inst = ((inst_hi as u16) << 8) | inst_lo as u16;
            // Match and dispatch instruction function
            match (inst_hi & 0xF0) >> 4 {
                0x0 => {
                    if inst_lo == 0xE0 {
                        self.cls();
                    } else if inst_lo == 0xEE {
                        self.ret();
                    } else {
                        let nnn = self.inst & 0x0FFF;
                        self.sys(nnn);
                    }
                }
                0x1 => {
                    let nnn = self.inst & 0x0FFF;
                    self.jmp(nnn);
                }
                0x2 => {
                    let nnn = self.inst & 0x0FFF;
                    self.call(nnn);
                }
                0x3 => {
                    let kk = inst_lo;
                    let x = inst_hi & 0x0F;
                    self.se(x, kk);
                }
                0x4 => {
                    let kk = inst_lo;
                    let x = inst_hi & 0x0F;
                    self.sne(x, kk);
                }
                0x5 => {
                    let x = inst_hi & 0x0F;
                    let y = (inst_lo & 0xF0) >> 4;
                    self.sexy(x, y);
                }
                0x6 => {
                    let x = inst_hi & 0x0F;
                    let kk = inst_lo;
                    self.ld(x, kk);
                }
                0x7 => {
                    let x = inst_hi & 0x0F;
                    let kk = inst_lo;
                    self.add(x, kk);
                }
                // General purpose register instructions
                // for arithmetic and logical operations
                0x8 => {
                    let x = inst_hi & 0x0F;
                    let y = (inst_lo & 0xF0) >> 4;
                    match inst_lo & 0x0F {
                        0x0 => self.ldxy(x, y),
                        0x1 => self.or(x, y),
                        0x2 => self.and(x, y),
                        0x3 => self.xor(x, y),
                        0x4 => self.adc(x, y),
                        0x5 => self.sub(x, y),
                        0x6 => self.shr(x),
                        0x7 => self.subn(x, y),
                        0xE => self.shl(x),
                        _ => self.ill(),
                    }
                }
                0x9 => {
                    let x = inst_hi & 0x0F;
                    let y = (inst_lo & 0xF0) >> 4;
                    self.snexy(x, y);
                }
                0xA => {
                    let nnn = self.inst & 0x0FFF;
                    self.ldi(nnn);
                }
                0xB => {
                    let nnn = self.inst & 0x0FFF;
                    self.jpv0(nnn);
                }
                0xC => {
                    let x = inst_hi & 0x0F;
                    let kk = inst_lo;
                    self.rnd(x, kk);
                }
                0xD => {
                    let x = inst_hi & 0x0F;
                    let y = (inst_lo & 0xF0) >> 4;
                    let n = inst_lo & 0x0F;
                    self.drw(x, y, n);
                }
                0xE => {
                    let x = inst_hi & 0x0F;
                    match inst_lo {
                        0x9E => self.skp(x),
                        0xA1 => self.sknp(x),
                        _ => self.ill(),
                    }
                }
                0xF => {
                    let x = inst_hi & 0x0F;
                    match inst_lo {
                        0x07 => self.ldvdt(x),
                        0x0A => match self.get_pressed_key() {
                            Some(key) => {
                                self.ldk(x, key as u8);
                            }
                            None => self.hold_flag = true,
                        },
                        0x15 => self.lddt(x),
                        0x18 => self.ldst(x),
                        0x1E => self.addi(x),
                        0x29 => self.ldsi(x),
                        0x33 => self.ldbcd(x),
                        0x55 => self.cpvi(x),
                        0x65 => self.ldiv(x),
                        _ => self.ill(),
                    }
                }
                // Illegal instruction
                _ => self.ill(),
            }
        } else {
            match self.get_pressed_key() {
                Some(key) => {
                    // Fetch the value x from the last instruction
                    // that was loaded before sleep
                    let x = (self.inst & 0x0F00) >> 8;
                    self.ldk(x as u8, key as u8);
                    self.hold_flag = false;
                }
                None => {
                    self.sleep();
                }
            }
            self.sleep();
        }
        // Update sound timers if every 1/60 seconds
        let cycles_per_60hz = ((1.0 / 60.0) / (1.0 / self.clock_speed)).round() as u64;
        if self.cycle % cycles_per_60hz == 0 {
            self.update_timers();
        }
        self.cycle += 1;
    }

    // No operation. CPU idles
    fn sleep(&mut self) {
        self.cycle += 1;
    }

    //Illegal operation
    fn ill(&mut self) {
        self.core_dump();
        panic!(
            "Illegal instruction {:#06X} provided! Dumping core!",
            self.inst
        );
    }

    // Implement CPU instructions
    fn sys(&mut self, addr: u16) {
        self.pc = addr;
    }

    // Clears the display
    fn cls(&mut self) {
        self.display.clear();
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
        }
    }

    // Skips the next instruction if Vx != kk
    fn sne(&mut self, vx: u8, byte: u8) {
        if self.v[vx as usize] != byte {
            self.pc += 2;
        }
    }

    // Skips the nex instruction if Vx == Vy
    fn sexy(&mut self, vx: u8, vy: u8) {
        if self.v[vx as usize] == self.v[vy as usize] {
            self.pc += 2;
        }
    }

    // Puts value into register Vx
    fn ld(&mut self, vx: u8, byte: u8) {
        self.v[vx as usize] = byte;
    }

    // Adds kk to the register Vx
    fn add(&mut self, vx: u8, byte: u8) {
        self.v[vx as usize] = self.v[vx as usize].overflowing_add(byte).0;
    }

    // Sets Vx = Vy
    fn ldxy(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] = self.v[vy as usize];
    }

    // Sets Vx = Vx OR Vy
    fn or(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] |= self.v[vy as usize];
    }

    // Sets Vx = Vx AND Vy
    fn and(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] &= self.v[vy as usize];
    }

    // Sets Vx = Vx XOR Vy
    fn xor(&mut self, vx: u8, vy: u8) {
        self.v[vx as usize] ^= self.v[vy as usize];
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
    }

    // The least significant bit of Vx is stored in Vf
    // and Vx is then right-shifted by 1 (divided by 2)
    fn shr(&mut self, vx: u8) {
        self.v[0xF] = self.v[vx as usize] & 0x01;
        self.v[vx as usize] = self.v[vx as usize] >> 1;
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
    }

    // The significant bit of Vx is stored in Vf
    // and Vx is then left-shifted by 1 (multiplied by 2)
    fn shl(&mut self, vx: u8) {
        self.v[0xF] = (self.v[vx as usize] & 0x80) >> 7;
        self.v[vx as usize] = self.v[vx as usize] << 1;
    }

    // Skips the next instruction if Vx != Vy
    fn snexy(&mut self, vx: u8, vy: u8) {
        if self.v[vx as usize] != self.v[vy as usize] {
            self.pc += 2;
        }
    }

    // Loads the 16-bit address value into the I register
    fn ldi(&mut self, addrs: u16) {
        self.i = addrs;
    }

    // Sets PC to the provided address + V0
    fn jpv0(&mut self, addrs: u16) {
        self.pc = addrs + self.v[0x0] as u16;
    }

    // Set Vx to a random byte AND:ed with the provided byte kk
    fn rnd(&mut self, vx: u8, byte: u8) {
        let mut rng = rand::thread_rng();
        self.v[vx as usize] = rng.gen::<u8>() & byte;
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
        for i in (0..n as usize).into_iter() {
            let byte = self.ram.data[self.i as usize + i];
            // Wrap y-cordinate if sprite goes off screen
            let y = if self.v[vy as usize] as usize + i >= DISPLAY_HEIGHT {
                self.v[vy as usize] as usize + i - DISPLAY_HEIGHT
            } else {
                self.v[vy as usize] as usize + i
            };
            for (j, bit) in byte_to_bools(byte).iter().enumerate() {
                // Wrap x-coordinate if it goes off screen
                let x = if self.v[vx as usize] as usize + j >= DISPLAY_WIDTH {
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
    }

    // Skips the next instruction if the specified key is currently held
    fn skp(&mut self, key: u8) {
        if self.pressed_keys[key as usize] {
            self.pc += 2;
        }
    }

    // Skips the next instruction if a certain key is not pressed
    fn sknp(&mut self, key: u8) {
        if !self.pressed_keys[key as usize] {
            self.pc += 2;
        }
    }

    // Loads the value of the delay timer into register Vx
    fn ldvdt(&mut self, vx: u8) {
        self.v[vx as usize] = self.dt;
    }

    // Holds execution until a key is pressed,
    // then stores the value of the key in Vx
    // Holding of CPU execution is handled in the tick function
    fn ldk(&mut self, vx: u8, key: u8) {
        self.v[vx as usize] = key;
    }

    // Loads the value of Vx into the the delay timer
    fn lddt(&mut self, vx: u8) {
        self.dt = self.v[vx as usize];
    }

    // Loads the value of Vx into the sound timer register
    fn ldst(&mut self, vx: u8) {
        self.st = self.v[vx as usize];
    }

    // Adds the contents of register Vx to the 16-bit I register
    fn addi(&mut self, vx: u8) {
        self.i = self.i.overflowing_add(self.v[vx as usize] as u16).0;
    }

    // Loads the RAM location of the digit stored in Vx into
    // the I register. Panics if the digit value is larger than 15
    fn ldsi(&mut self, vx: u8) {
        if self.v[vx as usize] <= 0xF {
            self.i = 5 * self.v[vx as usize] as u16;
        } else {
            panic!("Tried to load sprite of an invalid digit!");
        }
    }

    // Stores the BCD representation of the value in Vx, in I
    // (hudreds in I, tens in I+1, and ones in I+2)
    fn ldbcd(&mut self, vx: u8) {
        let idx = self.i as usize;
        self.ram.data[idx] = (self.v[vx as usize] as f32 / 100.0).floor() as u8;
        self.ram.data[idx] = ((self.v[vx as usize] % 100) as f32 / 10.0).floor() as u8;
        self.ram.data[idx] = self.v[vx as usize] % 10;
    }

    // Copies register V0 through Vx into RAM, starting at
    // the address strored in I
    fn cpvi(&mut self, vx: u8) {
        for i in 0..vx as usize + 1 {
            self.ram.data[self.i as usize + i] = self.v[i];
        }
    }

    // Copies values from RAM into registers V0 through Vx
    fn ldiv(&mut self, vx: u8) {
        for j in 0..vx as usize + 1 {
            self.v[j] = self.ram.data[self.i as usize + j];
        }
    }
}

fn disassemble(input: String) -> Result<String, String> {
    Err("Not yet implemented".to_string())
}
