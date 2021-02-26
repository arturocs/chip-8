#![allow(non_snake_case)]
pub mod chip8;
use std::error::Error;

use chip8::Chip8;

use ggez::{event, graphics, Context, GameResult};
use ggez::{
    event::{KeyCode, KeyMods},
    graphics::Color,
};

struct State {
    chip8: Chip8,
    speed: u8,
}
impl State {
    fn new(rom_path: &str) -> Result<Self, Box<dyn Error>> {
        let mut chip8 = Chip8::new();
        chip8.load_rom(rom_path)?;
        Ok(Self { chip8, speed: 3 })
    }

    fn key_event(&mut self, keycode: KeyCode, pressed: bool) {
        match keycode {
            KeyCode::Key1 => self.chip8.set_key(0, pressed),
            KeyCode::Key2 => self.chip8.set_key(1, pressed),
            KeyCode::Key3 => self.chip8.set_key(2, pressed),
            KeyCode::Key4 => self.chip8.set_key(3, pressed),
            KeyCode::Q => self.chip8.set_key(4, pressed),
            KeyCode::W => self.chip8.set_key(5, pressed),
            KeyCode::E => self.chip8.set_key(6, pressed),
            KeyCode::R => self.chip8.set_key(7, pressed),
            KeyCode::A => self.chip8.set_key(8, pressed),
            KeyCode::S => self.chip8.set_key(9, pressed),
            KeyCode::D => self.chip8.set_key(10, pressed),
            KeyCode::F => self.chip8.set_key(11, pressed),
            KeyCode::Z => self.chip8.set_key(12, pressed),
            KeyCode::X => self.chip8.set_key(13, pressed),
            KeyCode::C => self.chip8.set_key(14, pressed),
            KeyCode::V => self.chip8.set_key(15, pressed),
            _ => {}
        }
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        for _ in 0..self.speed {
            self.chip8.emulate_cycle();
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);
        let screen = self.chip8.get_gfx();
        for i in 0..screen.len() {
            for j in 0..screen[0].len() {
                if screen[i][j] == 1 {
                    let rectangle = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new_i32(j as i32 * 10, i as i32 * 10, 10, 10),
                        Color::WHITE,
                    )?;
                    graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
                }
            }
        }
        graphics::present(ctx)?;
        // We yield the current thread until the next update
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        self.key_event(keycode, true);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        self.key_event(keycode, false);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (ctx, events_loop) = ggez::ContextBuilder::new("CHIP-8", "Arturo")
        .window_setup(ggez::conf::WindowSetup::default().title("CHIP-8"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(640.0, 320.0))
        .build()?;

    if let Some(game) = std::env::args().nth(1) {
        let state = State::new(&game)?;
        event::run(ctx, events_loop, state)
    } else {
        panic!("Error: you need to pass the game as argument");
    }
}
