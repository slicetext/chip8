#![allow(non_snake_case)]
use std::{cmp, fs::File, io::{self, BufReader, Read}, u16};

use macroquad::prelude::*;


const START_ADDRESS: u16 = 0x200;

const FONTSET_SIZE: u8 = 80;
const FONTSET_START_ADDRESS: usize = 0x50;

const VIDEO_WIDTH: u8 = 64;
const VIDEO_HEIGHT: u8 = 32;

const COLOR_BG: Color = DARKGREEN;
const COLOR_MAIN: Color = GREEN;

struct Chip8 {
    registers: [u8; 16],
    memory:    [u8; 4096],
    index:     u16,
    pc:        u16,
    stack:     [u16; 16],
    sp:        usize,
    delay_timer:u8,
    sound_timer:u8,
    keypad:    [u8; 16],
    video:     [u32; 64 * 32],
    opcode:    u16,
    // RNG

}
impl Chip8 {
    fn new() -> Self {
        let reg: [u8; 16] = [0;16];
        let mem: [u8; 4096] = [0; 4096];
        let stck: [u16; 16] = [0; 16];
        let kpad: [u8; 16] = [0;16];
        let vdeo: [u32; 64 * 32] = [0;64*32];
        let mut c = Chip8 {
            registers: reg,
            memory: mem,
            index: 0,
            pc: 0,
            stack: stck,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: kpad,
            video: vdeo,
            opcode: 0,

        };
        c.init();
        return c;
    }
    fn init(&mut self) {
        // Create font
        let fontset: [usize; FONTSET_SIZE as usize] =
        [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];
        // Load font into memory
        for i in 0usize..FONTSET_SIZE as usize {
            self.memory[FONTSET_START_ADDRESS + i] = fontset[i] as u8;
        }
    }
    fn load_rom(&mut self, filename: &str) -> io::Result<()> {
        // Read file to buffer
        let f = File::open(filename)?;
        let mut reader = BufReader::new(f);
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        // Load ROM into memory from buffer
        for i in 0..buffer.len() {
            self.memory[i + START_ADDRESS as usize]=buffer[i];
        }
        self.pc=START_ADDRESS;
        return Ok(());
    }
    // Instructions
    // Clear display
    fn OP_00E0(&mut self) {
        for i in 0..self.video.len() {
            self.video[i]=0;
        }
    }
    // Return from subroutine
    fn OP_00EE(&mut self) {
        self.sp-=1;
        self.pc=self.stack[self.sp];
    }
    // Jump to location nnn
    fn OP_1nnn(&mut self) {
        self.pc=self.opcode & 0x0FFF;
    }
    // Call subroutine at nnn
    fn OP_2nnn(&mut self) {
        self.stack[self.sp] = self.pc;
        self.sp+=1;
        self.pc=self.opcode & 0x0FFF;
    }
    // Skip next instruction if Vx == kk
    fn OP_3xkk(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let byte: u16 = self.opcode & 0x00FF;
        if self.registers[Vx as usize] as u16 == byte  {
            self.pc+=2;
        }
    }
    // Skip next instruction if Vx != kk
    fn OP_4xkk(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let byte: u16 = self.opcode & 0x00FF;
        if self.registers[Vx as usize] as u16 != byte  {
            self.pc+=2;
        }
    }
    // Skip next instruction if Vx == Vy
    fn OP_5xy0(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        if self.registers[Vx as usize] == self.registers[Vy as usize]  {
            self.pc+=2;
        }
    }
    // Set Vx = kk
    fn OP_6xkk(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let byte: u16 = self.opcode & 0x00FF;
        self.registers[Vx as usize] = byte as u8;
    }
    // Set Vx += kk
    fn OP_7xkk(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let byte: u16 = self.opcode & 0x00FF;
        self.registers[Vx as usize] =self.registers[Vx as usize].wrapping_add(byte as u8);
    }
    // Set Vx = Vy
    fn OP_8xy0(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        self.registers[Vx as usize] = self.registers[Vy as usize];
    }
    // Set Vx = Vx OR Vy
    fn OP_8xy1(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        self.registers[Vx as usize] |= self.registers[Vy as usize];
    }
    // Set Vx = Vx AND Vy
    fn OP_8xy2(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        self.registers[Vx as usize] &= self.registers[Vy as usize];
    }
    // Set Vx = Vx XOR Vy
    fn OP_8xy3(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        self.registers[Vx as usize] ^= self.registers[Vy as usize];
    }
    // Set Vx += Vy. set VF = carry
    fn OP_8xy4(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        let sum: u16 = self.registers[Vx as usize].wrapping_add(self.registers[Vy as usize]) as u16;

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[Vx as usize] = sum as u8 & 0xFF;
    }
    // Set Vx -= Vy, set VF = NOT borrow
    fn OP_8xy5(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;

        if self.registers[Vx as usize] > self.registers[Vy as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[Vx as usize] = self.registers[Vx as usize].wrapping_sub(self.registers[Vy as usize]);
    }
    // Set Vx >>= 1
    fn OP_8xy6(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;

        // Least Significant Bit stored in VF
        self.registers[0xF] = self.registers[Vx as usize] & 0x1;
        
        self.registers[Vx as usize] >>= 1;
    }
    // Set Vx = Vy - Vx, set VF = NOT borrow
    fn OP_8xy7(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;

        if self.registers[Vy as usize] > self.registers[Vx as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[Vx as usize] = self.registers[Vy as usize].wrapping_sub(self.registers[Vx as usize]);
    }
    // Set Vx <<= 1
    fn OP_8xyE(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;

        // Save Most Significant Bit in VF
        self.registers[0xF] = (self.registers[Vx as usize] & 0x80) >> 7;

        self.registers[Vx as usize] <<= 1;
    }
    // Skip next instruction if Vx != Vy
    fn OP_9xy0(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;

        if self.registers[Vx as usize] != self.registers[Vy as usize] {
            self.pc+=2;
        }
    }
    // Set I = nnn
    fn OP_Annn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.index = address;
    }
    // Jump to nnn + V0
    fn OP_Bnnn(&mut self) {
        let address: u16 = self.opcode & 0x0FFF;
        self.pc = self.registers[0] as u16 + address;
    }
    // Set Vx = random byte AND kk
    fn OP_Cxkk(&mut self) {
        // TODO: rng with u8, not u32
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let byte: u16 = self.opcode & 0x00FF;

        self.registers[Vx as usize] = (rand::rand() as u16 & byte) as u8;
    }
    // Display sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn OP_Dxyn(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let Vy: u16 = (self.opcode & 0x00F0) >> 4;
        let height: u16 = self.opcode & 0x000F;

        // Wrap screen boundaries
        let xPos = self.registers[Vx as usize] % VIDEO_WIDTH;
        let yPos = self.registers[Vy as usize] % VIDEO_HEIGHT;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[(self.index as u16 + row) as usize];
            for col in 0..8 {
                let spritePixel: u8 = sprite_byte & (0x80 >> col);
                let index = ((yPos as u16 + row as u16) * VIDEO_WIDTH as u16 + (xPos as u16 + col as u16)) as usize;
                let mut screenPixel: &mut u32=&mut 0;
                if index<=2048{
                    screenPixel = &mut self.video[index];
                }
                if spritePixel!=0x0 {
                    if *screenPixel == 0xFFFFFFFF {
                        self.registers[0xF] = 1;
                    }
                    *screenPixel ^= 0xFFFFFFFF;
                }
            }
        }
    }
    // Skip next instruction if key with value of Vx is pressed
    fn OP_Ex9E(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let key: u8 = self.registers[Vx as usize];

        if self.keypad[key as usize]!=0x0 {
            self.pc+=2;
        }
    }
    // Skip next instruction if key with the value of Vx is not pressed
    fn OP_ExA1(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let key: u8 = self.registers[Vx as usize];

        if self.keypad[key as usize]==0x0 {
            self.pc+=2;
        }
    }
    // Set Vx = delay timer value
    fn OP_Fx07(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        self.registers[Vx as usize] = self.delay_timer;
    }
    // Wait for a key press, store the value of the key in Vx
    fn OP_Fx0A(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let mut is_pressed: bool =false;
        for i in 0..self.keypad.len() {
            if self.keypad[i]!=0x0 {
                self.registers[Vx as usize] = i as u8;
                is_pressed=true;
            }
        }
        if !is_pressed {
            self.pc-=2;
        }
    }
    // Set delay timer = Vx
    fn OP_Fx15(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        self.delay_timer=self.registers[Vx as usize];
    }
    // Set sound timer = Vx
    fn OP_Fx18(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        self.sound_timer=self.registers[Vx as usize];
    }
    // Set I += Vx
    fn OP_Fx1E(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        self.index+=self.registers[Vx as usize] as u16;
    }
    // Set I = location of sprite for digit Vx
    fn OP_Fx29(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let digit: u8 = self.registers[Vx as usize];

        self.index = FONTSET_START_ADDRESS as u16 + (5 * digit) as u16;
    }
    // Store BCD (Binary Coded Decimal) representation of Vx in memory locations I, I+1, and I+2
    fn OP_Fx33(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;
        let mut value: u8 = self.registers[Vx as usize];

        // Ones place
        self.memory[(self.index+2) as usize] = value % 10;
        value /= 10;

        // Tens place
        self.memory[(self.index+1) as usize] = value % 10;
        value /= 10;

        // Hundreds place
        self.memory[(self.index) as usize] = value % 10;
    }
    // Store registers V0 through Vx in memory starting at location I
    fn OP_Fx55(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;

        for i in 0..Vx {
            self.memory[(self.index+i) as usize] = self.registers[i as usize];
        }
    }
    // Read registers V0 through Vx from memory starting at location I
    fn OP_Fx65(&mut self) {
        let Vx: u16 = (self.opcode & 0x0F00) >> 8;

        for i in 0..Vx {
            self.registers[i as usize] = self.memory[(self.index+i) as usize];
        }
    }
    fn OP_NULL(&mut self, instruction: &u16) {
        println!("Invalid instruction: {:#06x} ({})",instruction,instruction);
    }
    // Call Instruction's function
    fn do_instruction(&mut self) {
        let instruction = self.opcode;
        match (instruction & 0xF000) >> 12 {
            0 => {
                match instruction & 0x000F {
                    0 => self.OP_00E0(),
                    14=> self.OP_00EE(),
                    _ => self.OP_NULL(&instruction),
                }
            },
            1 => self.OP_1nnn(),
            2 => self.OP_2nnn(),
            3 => self.OP_3xkk(),
            4 => self.OP_4xkk(),
            5 => self.OP_5xy0(),
            6 => self.OP_6xkk(),
            7 => self.OP_7xkk(),
            8 => {
                match instruction & 0x000F {
                    0 => self.OP_8xy0(),
                    1 => self.OP_8xy1(),
                    2 => self.OP_8xy2(),
                    3 => self.OP_8xy3(),
                    4 => self.OP_8xy4(),
                    5 => self.OP_8xy5(),
                    6 => self.OP_8xy6(),
                    7 => self.OP_8xy7(),
                    14=> self.OP_8xyE(),
                    _ => self.OP_NULL(&instruction),
                }
            },
            9 => self.OP_9xy0(),
            10=> self.OP_Annn(),
            11=> self.OP_Bnnn(),
            12=> self.OP_Cxkk(),
            13=> self.OP_Dxyn(),
            14=> {
                match instruction & 0x00FF {
                    158 => self.OP_Ex9E(),
                    161 => self.OP_ExA1(),
                    _   => self.OP_NULL(&instruction),
                }
            },
            15=> {
                match instruction & 0x00FF {
                    7   => self.OP_Fx07(),
                    10  => self.OP_Fx0A(),
                    21  => self.OP_Fx15(),
                    24  => self.OP_Fx18(),
                    30  => self.OP_Fx1E(),
                    41  => self.OP_Fx29(),
                    51  => self.OP_Fx33(),
                    85  => self.OP_Fx55(),
                    101 => self.OP_Fx65(),
                    _   => self.OP_NULL(&instruction),
                }
            },
            _ => self.OP_NULL(&instruction),
        }
    }
    fn cycle(&mut self) {
        // self.opcode = ((self.memory[self.pc as usize] << 8) | self.memory[(self.pc+1) as usize]) as u16;
        self.opcode = ((self.memory[self.pc as usize] as u16) << 8) | self.memory[(self.pc+1) as usize] as u16;

        self.pc += 2;

        self.do_instruction();

        if self.delay_timer > 0 {
            self.delay_timer-=1;
        }
        if self.sound_timer > 0 {
            self.sound_timer-=1;
        }
    }
}
fn do_kbd_input(chip8: &Chip8) {
    if is_key_down(KeyCode::) {
    }
}
#[macroquad::main("Chip8")]
async fn main() {
    let mut chip8 = Chip8::new();
    chip8.load_rom("/home/owner/Projects/emulator/roms/tetris.ch8").expect("Failed to load file");
    loop{
        do_kbd_input(&chip8);
        chip8.cycle();
        clear_background(COLOR_BG);
        for i in 0..VIDEO_WIDTH {
            for j in 0..VIDEO_HEIGHT {
                if chip8.video[(i as usize+(j as usize*VIDEO_WIDTH as usize)) as usize] != 0x0 {
                    let screen_size=cmp::min(screen_width() as u32, screen_height() as u32);
                    let pixel_size=70;
                    draw_rectangle(i as f32*(screen_size/64)as f32, j as f32*(screen_size/64)as f32, (screen_size/pixel_size)as f32, (screen_size/pixel_size)as f32, COLOR_MAIN);
                }
            }
        }
        next_frame().await
    }
}
