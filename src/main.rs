use std::io;
use itertools::Itertools;

mod interface;
mod game;

use game::Game;
use crate::game::CellOrientation;

fn main() -> Result<(), io::Error> {
    let mut game = Game::random_valid(11, 11);
    game.powered_cells().iter().for_each(|(x, y)| game.power_cell(*x, *y).ok().expect("Could not power this cell"));
    interface::run(game).expect("Interface crashed");

    Ok(())
}
