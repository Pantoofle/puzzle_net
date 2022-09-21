use std::io;

mod interface;
mod game;

use game::Game;
use crate::game::CellOrientation;

fn main() -> Result<(), io::Error> {
    let mut game = Game::random_valid(51, 51);
    interface::run(game).expect("Interface crashed");

    Ok(())
}
