#![allow(non_snake_case)]

use chip8::Chip8;
pub mod chip8;
fn main() {
    let mut c8 = Chip8::new();
    c8.load_rom("./teris.c8").unwrap();
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
