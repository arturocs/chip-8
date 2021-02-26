#![allow(non_snake_case)]
pub mod chip8;
pub mod interface;
use std::error::Error;

use ggez::event;
use interface::State;

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
