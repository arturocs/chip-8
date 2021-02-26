#![allow(non_snake_case)]
use std::fs;
use std::{error::Error, usize};

use rand::{thread_rng, Rng};
struct State {
    opcode: u16,
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    gfx: [u8; 64 * 32],
    delay_timer: u8,
    sound_timer: u8,
    stack: [u16; 16],
    sp: u16,
    key: [u8; 16],
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
            opcode: 0,
            memory,
            v: [0; 16],
            i: 0,
            pc: 512,
            gfx: [0; 64 * 32],
            delay_timer: 0,
            sound_timer: 0,
            stack: [0; 16],
            sp: 0,
            key: [0; 16],
            draw_flag: true,
        }
    }

    fn load_rom(&mut self, rom_path: &str) -> Result<(), Box<dyn Error>> {
        let rom = fs::read(rom_path)?;
        self.memory[512..512 + rom.len()].copy_from_slice(&rom);
        Ok(())
    }

    fn x0(&mut self) {
        match self.opcode {
            0x00E0 => {
                self.gfx.fill(0);
                self.draw_flag = true
            }
            0x00EE => {
                //self.sp -= 1;
                self.sp = self.sp.saturating_sub(1);
                self.pc = self.stack[self.sp as usize];
            }
            _ => panic!("Unknown Opcode {:#X}", self.opcode),
        }
        self.pc += 2;
    }

    fn x1(&mut self) {
        self.pc = self.opcode & 0x0FFF;
    }

    fn x2(&mut self) {
        self.stack[self.sp as usize] = self.pc;
        self.pc += 1;
        self.pc = self.opcode & 0x0FFF;
    }

    fn x3(&mut self) {
        if self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16 == (self.opcode & 0x00FF) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn x4(&mut self) {
        if self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16 != (self.opcode & 0x00FF) {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn x5(&mut self) {
        if self.v[((self.opcode & 0x0F00) >> 8) as usize]
            == self.v[((self.opcode & 0x00F0) >> 4) as usize]
        {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn x6(&mut self) {
        self.v[((self.opcode & 0x0F00) >> 8) as usize] = (self.opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    fn x7(&mut self) {
        self.v[((self.opcode & 0x0F00) >> 8) as usize] += (self.opcode & 0x00FF) as u8;
        self.pc += 2;
    }

    fn x8(&mut self) {
        match self.opcode & 0x000F {
            0x0000 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] =
                    self.v[((self.opcode & 0x00F0) >> 4) as usize];
            }
            0x0001 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] |=
                    self.v[((self.opcode & 0x00F0) >> 4) as usize];
            }
            0x0002 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] &=
                    self.v[((self.opcode & 0x00F0) >> 4) as usize];
            }
            0x0003 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] ^=
                    self.v[((self.opcode & 0x00F0) >> 4) as usize];
            }
            0x0004 => {
                let (result, overflow) = self.v[((self.opcode & 0x0F00) >> 8) as usize]
                    .overflowing_add(self.v[((self.opcode & 0x00F0) >> 4) as usize]);
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = result;
                self.v[0xF] = overflow as u8;
            }

            0x0005 => {
                let (result, borrow) = self.v[((self.opcode & 0x0F00) >> 8) as usize]
                    .overflowing_sub(self.v[((self.opcode & 0x00F0) >> 4) as usize]);
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = result;
                self.v[0xF] = borrow as u8;
            }
            0x0006 => {
                self.v[0xF] = self.v[((self.opcode & 0x0F00) >> 8) as usize] & 0x1;
                self.v[((self.opcode & 0x0F00) >> 8) as usize] >>= 1;
            }
            0x0007 => {
                let (result, borrow) = self.v[((self.opcode & 0x00F0) >> 4) as usize]
                    .overflowing_sub(self.v[((self.opcode & 0x0F00) >> 8) as usize]);
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = result;
                self.v[0xF] = borrow as u8;
            }

            0x000E => {
                self.v[0xF] = self.v[((self.opcode & 0x0F00) >> 8) as usize] >> 7;
                self.v[((self.opcode & 0x0F00) >> 8) as usize] <<= 1;
            }

            _ => panic!("Unknown Opcode {:#X}", self.opcode),
        }
        self.pc += 2;
    }

    fn x9(&mut self) {
        if self.v[((self.opcode & 0x0F00) >> 8) as usize]
            != self.v[((self.opcode & 0x00F0) >> 4) as usize]
        {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    fn xA(&mut self) {
        self.i = self.opcode & 0x0FFF;
        self.pc += 2;
    }

    fn xB(&mut self) {
        self.pc = (self.opcode & 0x0FFF) + self.v[0] as u16;
    }

    fn xC(&mut self) {
        self.v[((self.opcode & 0x0F00) >> 8) as usize] =
            thread_rng().gen::<u8>() & ((self.opcode & 0x00FF) as u8);
        self.pc += 2;
    }

    fn xD(&mut self) {
        let x = self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16;
        let y = self.v[((self.opcode & 0x00F0) >> 4) as usize] as u16;
        let height = self.opcode & 0x000F;

        self.v[0xF] = 0;
        for yline in 0..height {
            let pixel = self.memory[(self.i + yline) as usize];
            for xline in 0..8 {
                //{
                if (pixel & (0x80 >> xline)) != 0 {
                    if self.gfx[(x + xline + ((y + yline) * 64)) as usize] == 1 {
                        self.v[0xF] = 1;
                    }
                    self.gfx[(x + xline + ((y + yline) * 64)) as usize] ^= 1;
                }
            }
        }

        self.draw_flag = true;
        self.pc += 2;
    }

    fn xE(&mut self) {
        match self.opcode & 0xF0FF {
            0xE09E => {
                if self.key[self.v[((self.opcode & 0x0F00) >> 8) as usize] as usize] != 0 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0xE0A1 => {
                if self.key[self.v[((self.opcode & 0x0F00) >> 8) as usize] as usize] == 0 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            _ => panic!("Unknown Opcode {:#X}", self.opcode),
        }
    }

    fn xF(&mut self) {
        match self.opcode & 0xF0FF {
            0xF007 => {
                self.v[((self.opcode & 0x0F00) >> 8) as usize] = self.delay_timer;
                self.pc += 2;
            }
            0xF00A => {
                let mut key_press = false;

                for (i, &k) in self.key.iter().enumerate() {
                    if k != 0 {
                        self.v[((self.opcode & 0x0F00) >> 8) as usize] = i as u8;
                        key_press = true;
                    }
                }

                // If we didn't received a keypress, skip this cycle and try again.
                if !key_press {
                    return;
                }

                self.pc += 2;
            }
            0xF015 => {
                self.delay_timer = self.v[((self.opcode & 0x0F00) >> 8) as usize];
                self.pc += 2;
            }
            0xF018 => {
                self.sound_timer = self.v[((self.opcode & 0x0F00) >> 8) as usize];
                self.pc += 2;
            }
            0xF01E => {
                if self.i + self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16 > 0xFFF {
                    // VF is set to 1 when range overflow (I+VX>0xFFF), and 0 when there isn't.
                    self.v[0xF] = 1;
                } else {
                    self.v[0xF] = 0;
                }
                self.i += self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16;
                self.pc += 2;
            }
            0xF029 => {
                self.i = self.v[((self.opcode & 0x0F00) >> 8) as usize] as u16 * 0x5;
                self.pc += 2;
            }
            0xF033 => {
                let i = self.i as usize;
                self.memory[i] = self.v[((self.opcode & 0x0F00) >> 8) as usize] / 100;
                self.memory[i + 1] = (self.v[((self.opcode & 0x0F00) >> 8) as usize] / 10) % 10;
                self.memory[i + 2] = (self.v[((self.opcode & 0x0F00) >> 8) as usize] % 100) % 10;
                self.pc += 2;
            }
            0xF055 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let i = self.i as usize;
                self.memory[i..=i + x].copy_from_slice(&self.v[0..=x]);
                //self.i += x as u16;
                self.pc += 2;
            }
            0xF065 => {
                let x = ((self.opcode & 0x0F00) >> 8) as usize;
                let i = self.i as usize;
                self.v[0..=x].copy_from_slice(&self.memory[i..=i + x]);

                //self.i += x as u16;
                self.pc += 2;
            }
            _ => panic!("Unknown Opcode {:#X}", self.opcode),
        }
    }

    fn emulate_cycle(&mut self) {
        let pc = self.pc as usize;
        self.opcode = (self.memory[pc] as u16) << 8 | self.memory[pc + 1] as u16;
        match self.opcode >> 12 {
            0x0 => Self::x0(self),
            0x1 => Self::x1(self),
            0x2 => Self::x2(self),
            0x3 => Self::x3(self),
            0x4 => Self::x4(self),
            0x5 => Self::x5(self),
            0x6 => Self::x6(self),
            0x7 => Self::x7(self),
            0x8 => Self::x8(self),
            0x9 => Self::x9(self),
            0xA => Self::xA(self),
            0xB => Self::xB(self),
            0xC => Self::xC(self),
            0xD => Self::xD(self),
            0xE => Self::xE(self),
            0xF => Self::xF(self),
            _ => panic!("Unknown Opcode {:#X}", self.opcode),
        };

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
    fn get_gfx(&self) -> &[u8] {
        &self.gfx[..]
    }
}

fn main() {
    let mut c8 = State::new();
    c8.load_rom("./tetris.c8").unwrap();
    loop {
        c8.emulate_cycle();
        for i in 0..32 {
            for j in 0..64 {
                if c8.get_gfx()[i * 64 + j] == 0 {
                    print!(" ");
                } else {
                    print!("*");
                }
            }
            println!("");
        }
    }
}
