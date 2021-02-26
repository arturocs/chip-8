#![allow(non_snake_case)]
use std::fs;
use std::{error::Error, usize};

use rand::{thread_rng, Rng};
struct State {
    opcode: (u8, u8, u8, u8),
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    gfx: [[u8; 64]; 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    keys: [u8; 16],
    draw_flag: bool,
}

impl State {
    fn new() -> Self {
        let mut memory = [0u8; 4096];
        let chip8_fontset = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20, 0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80,
            0xF0, 0xF0, 0x10, 0xF0, 0x10, 0xF0, 0x90, 0x90, 0xF0, 0x10, 0x10, 0xF0, 0x80, 0xF0,
            0x10, 0xF0, 0xF0, 0x80, 0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40, 0xF0, 0x90,
            0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0, 0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
            0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80, 0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0,
            0xF0, 0x80, 0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
        ];
        memory[..80].copy_from_slice(&chip8_fontset);
        Self {
            opcode: (0, 0, 0, 0),
            memory,
            v: [0; 16],
            i: 0,
            pc: 512,
            gfx: [[0; 64]; 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            keys: [0; 16],
            draw_flag: true,
        }
    }

    fn load_rom(&mut self, rom_path: &str) -> Result<(), Box<dyn Error>> {
        let rom = fs::read(rom_path)?;
        let pc = self.pc as usize;
        self.memory[pc..pc + rom.len()].copy_from_slice(&rom);
        Ok(())
    }

    fn NNN(&self) -> u16 {
        (self.opcode.1 as u16) << 8 | self.memory[self.pc as usize + 1] as u16
    }
    fn NN(&self) -> u8 {
        self.memory[self.pc as usize + 1]
    }

    // 0x00E0: Clears the screen
    fn x00E0(&mut self) {
        self.gfx.fill([0; 64]);
        self.draw_flag = true
    }

    // 0x00EE: Returns from subroutine
    fn x00EE(&mut self) {
        self.sp = self.sp.saturating_sub(1);
        self.pc = self.stack[self.sp as usize];
    }

    // 0x1NNN: Jumps to address NNN
    fn x1NNN(&mut self) {
        self.pc = Self::NNN(self) - 2;
    }

    // 0x2NNN: Calls subroutine at NNN.
    fn x2NNN(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        self.pc = Self::NNN(self) - 2;
    }

    // 0x3XNN: Skips the next instruction if VX equals NN
    fn x3XNN(&mut self) {
        let (_, x, ..) = self.opcode;
        if self.v[x as usize] == Self::NN(&self) {
            self.pc += 2;
        }
    }

    // 0x4XNN: Skips the next instruction if VX doesn't equal NN
    fn x4XNN(&mut self) {
        let (_, x, ..) = self.opcode;
        if self.v[x as usize] != Self::NN(&self) {
            self.pc += 2;
        }
    }
    // 0x5XY0: Skips the next instruction if VX equals VY.
    fn x5XY0(&mut self) {
        let (_, x, y, _) = self.opcode;
        if x == y {
            self.pc += 2;
        }
    }

    // 0x6XNN: Sets VX to NN.
    fn x6XNN(&mut self) {
        let (_, x, ..) = self.opcode;
        self.v[x as usize] = Self::NN(&self);
    }

    // 0x7XNN: Adds NN to VX.
    fn x7XNN(&mut self) {
        let (_, x, ..) = self.opcode;
        self.v[x as usize] += Self::NN(&self);
    }

    // 0x8XY0: Sets VX to the value of VY
    fn x8XY0(&mut self) {
        let (_, x, y, _) = self.opcode;
        self.v[x as usize] = self.v[y as usize];
    }

    // 0x8XY1: Sets VX to "VX OR VY"
    fn x8XY1(&mut self) {
        let (_, x, y, _) = self.opcode;
        self.v[x as usize] |= self.v[y as usize];
    }

    // 0x8XY2: Sets VX to "VX AND VY"
    fn x8XY2(&mut self) {
        let (_, x, y, _) = self.opcode;
        self.v[x as usize] &= self.v[y as usize];
    }

    // 0x8XY3: Sets VX to "VX XOR VY"
    fn x8XY3(&mut self) {
        let (_, x, y, _) = self.opcode;
        self.v[x as usize] ^= self.v[y as usize];
    }

    // 0x8XY4: Adds VY to VX. VF is set to 1 when there's a carry, and to 0 when there isn't
    fn x8XY4(&mut self) {
        let (_, x, y, _) = self.opcode;
        let (result, overflow) = self.v[x as usize].overflowing_add(self.v[y as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = overflow as u8;
    }

    // 0x8XY5: VY is subtracted from VX. VF is set to 0 when there's a borrow, and 1 when there isn't
    fn x8XY5(&mut self) {
        let (_, x, y, _) = self.opcode;
        let (result, borrow) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = borrow as u8;
    }

    // 0x8XY6: Shifts VX right by one. VF is set to the value of the least significant bit of VX before the shift
    fn x8XY6(&mut self) {
        let (_, x, ..) = self.opcode;
        self.v[0xF] = self.v[x as usize] & 0x1;
        self.v[x as usize] >>= 1;
    }

    // 0x8XY7: Sets VX to VY minus VX. VF is set to 0 when there's a borrow, and 1 when there isn't
    fn x8XY7(&mut self) {
        let (_, x, y, _) = self.opcode;
        let (result, borrow) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
        self.v[x as usize] = result;
        self.v[0xF] = borrow as u8;
    }

    // 0x8XYE: Shifts VX left by one. VF is set to the value of the most significant bit of VX before the shift
    fn x8XYE(&mut self) {
        let (_, x, ..) = self.opcode;
        self.v[0xF] = self.v[x as usize] >> 7;
        self.v[x as usize] <<= 1;
    }

    // 0x9XY0: Skips the next instruction if VX doesn't equal VY
    fn x9XY0(&mut self) {
        let (_, x, y, _) = self.opcode;
        if self.v[x as usize] != self.v[y as usize] {
            self.pc += 2;
        }
    }

    // ANNN: Sets I to the address NNN
    fn xANNN(&mut self) {
        self.i = Self::NNN(self);
    }

    // BNNN: Jumps to the address NNN plus V0
    fn xBNNN(&mut self) {
        self.i = Self::NNN(self) + self.v[0] as u16 - 2;
    }

    // CXNN: Sets VX to a random number and NN
    fn xCXNN(&mut self) {
        let (_, x, ..) = self.opcode;
        self.v[x as usize] = thread_rng().gen::<u8>() & Self::NN(self);
    }

    // DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels and a height of N pixels.
    fn xDXYN(&mut self) {
        let (_, x, y, n) = self.opcode;
        let x = self.v[x as usize] as usize;
        let y = self.v[y as usize] as usize;
        let n = n as usize;
        self.v[0xF] = 0;

        for yline in 0..n {
            let pixel = self.memory[self.i as usize + yline];
            for xline in 0..8 {
                if (pixel & (0x80 >> xline)) != 0 {
                    if self.gfx[y + yline][x + xline] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.gfx[y + yline][x + xline] ^= 1;
                }
            }
        }
        self.draw_flag = true;
    }
    // EX9E: Skips the next instruction if the key stored in VX is pressed
    fn xEX9E(&mut self) {
        let (_, x, ..) = self.opcode;
        if self.keys[self.v[x as usize] as usize] != 0 {
            self.pc += 2;
        }
    }
    // EXA1: Skips the next instruction if the key stored in VX isn't pressed
    fn xEXA1(&mut self) {
        let (_, x, ..) = self.opcode;
        if self.keys[self.v[x as usize] as usize] == 0 {
            self.pc += 2;
        }
    }
    // FX07: Sets VX to the value of the delay timer
    fn xFX07(&mut self) {
        let (_, x, ..) = self.opcode;
        self.v[x as usize] = self.delay_timer;
    }

    // FX0A: A key press is awaited, and then stored in VX
    fn xFX0A(&mut self) {
        let (_, x, ..) = self.opcode;
        if let Some((i, _)) = self.keys.iter().enumerate().find(|(_, &k)| k != 0) {
            self.v[x as usize] = i as u8
        } else {
            // If we didn't received a keypress, skip this cycle and try again.
            self.pc -= 2;
        }
    }

    // FX15: Sets the delay timer to VX
    fn xFX15(&mut self) {
        let (_, x, ..) = self.opcode;
        self.delay_timer = self.v[x as usize];
    }

    // FX18: Sets the sound timer to VX
    fn xFX18(&mut self) {
        let (_, x, ..) = self.opcode;
        self.sound_timer = self.v[x as usize];
    }

    // FX1E: Adds VX to I
    fn xFX1E(&mut self) {
        let (_, x, ..) = self.opcode;
        // Most CHIP-8 interpreters' FX1E instructions do not affect VF, with one exception: The CHIP-8 interpreter for the Commodore
        // Amiga sets VF to 1 when there is a range overflow (I+VX>0xFFF), and to 0 when there isn't. The only known game that
        // depends on this behavior is Spacefight 2091! while at least one game, Animal Race, depends on VF not being affected.
        // if self.i + self.v[x as usize] as u16 > 0xFFF {
        //    self.v[0xF] = 1;
        // } else {
        //     self.v[0xF] = 0;
        // }
        self.i += self.v[x as usize] as u16;
    }

    // FX29: Sets I to the location of the sprite for the character in VX. Characters 0-F (in hexadecimal) are represented by a 4x5 font
    fn xFX29(&mut self) {
        let (_, x, ..) = self.opcode;
        self.i = self.v[x as usize] as u16 * 0x5;
    }

    // FX33: Stores the Binary-coded decimal representation of VX at the addresses I, I plus 1, and I plus 2
    fn xFX33(&mut self) {
        let (_, x, ..) = self.opcode;
        let i = self.i as usize;
        self.memory[i] = self.v[x as usize] / 100;
        self.memory[i + 1] = (self.v[x as usize] / 10) % 10;
        self.memory[i + 2] = (self.v[x as usize] % 100) % 10;
    }

    // FX55: Stores V0 to VX in memory starting at address I
    fn xFX55(&mut self) {
        let (_, x, ..) = self.opcode;
        let i = self.i as usize;
        self.memory[i..=i + x as usize].copy_from_slice(&self.v[0..=x as usize]);
        //self.i += x as u16;
    }
    // FX65: Fills V0 to VX with values from memory starting at address I
    fn xFX65(&mut self) {
        let (_, x, ..) = self.opcode;
        let i = self.i as usize;
        self.v[0..=x as usize].copy_from_slice(&self.memory[i..=i + x as usize]);
        //self.i += x as u16;
    }

    fn emulate_cycle(&mut self) {
        let pc = self.pc as usize;
        self.opcode = (
            (self.memory[pc] & 0xF0) >> 4,
            self.memory[pc] & 0x0F,
            (self.memory[pc + 1] & 0xF0) >> 4,
            self.memory[pc + 1] & 0x0F,
        );
        let opcode_fn = match self.opcode {
            (0, 0, 0xE, 0) => Self::x00E0,
            (0, 0, 0xE, 0xE) => Self::x00EE,
            (1, ..) => Self::x1NNN,
            (2, ..) => Self::x2NNN,
            (3, ..) => Self::x3XNN,
            (4, ..) => Self::x4XNN,
            (5, ..) => Self::x5XY0,
            (6, ..) => Self::x6XNN,
            (7, ..) => Self::x7XNN,
            (8, .., 0) => Self::x8XY0,
            (8, .., 1) => Self::x8XY1,
            (8, .., 2) => Self::x8XY2,
            (8, .., 3) => Self::x8XY3,
            (8, .., 4) => Self::x8XY4,
            (8, .., 5) => Self::x8XY5,
            (8, .., 6) => Self::x8XY6,
            (8, .., 7) => Self::x8XY7,
            (8, .., 0xE) => Self::x8XYE,
            (9, ..) => Self::x9XY0,
            (0xA, ..) => Self::xANNN,
            (0xB, ..) => Self::xBNNN,
            (0xC, ..) => Self::xCXNN,
            (0xD, ..) => Self::xDXYN,
            (0xE, _, 9, 0xE) => Self::xEX9E,
            (0xE, _, 0xA, 1) => Self::xEXA1,
            (0xF, _, 0, 7) => Self::xFX07,
            (0xF, _, 0, 0xA) => Self::xFX0A,
            (0xF, _, 1, 5) => Self::xFX15,
            (0xF, _, 1, 8) => Self::xFX18,
            (0xF, _, 1, 0xE) => Self::xFX1E,
            (0xF, _, 2, 9) => Self::xFX29,
            (0xF, _, 3, 3) => Self::xFX33,
            (0xF, _, 5, 5) => Self::xFX55,
            (0xF, _, 6, 5) => Self::xFX65,
            _ => panic!(
                "Unknown Opcode ({:#X},{:#X},{:#X},{:#X})",
                self.opcode.0, self.opcode.1, self.opcode.2, self.opcode.3
            ),
        };

        opcode_fn(self);
        self.pc += 2;

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                println!("BEEP!");
            }
            self.sound_timer -= 1;
        }
    }
    fn get_gfx(&self) -> &[[u8; 64]] {
        &self.gfx[..]
    }
}

fn main() {
    let mut c8 = State::new();
    c8.load_rom("./tetris.c8").unwrap();
    loop {
        c8.emulate_cycle();
        let gfx = c8.get_gfx();
        for i in 0..gfx.len() {
            for j in 0..gfx[0].len() {
                if gfx[i][j] == 0 {
                    print!(" ");
                } else {
                    print!("*");
                }
            }
            println!();
        }
    }
}
