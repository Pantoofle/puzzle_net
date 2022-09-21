use rand::distributions::{Distribution, Standard};
use rand::Rng;
use std::collections::VecDeque;
use rand::seq::IteratorRandom;
use itertools::iproduct;

use strum::IntoEnumIterator;
use strum_macros::{FromRepr, EnumIter};

#[derive(Debug)]
pub enum GameError {
    CellIsLocked,
    InvalidCell,
}

#[derive(Clone, Copy, Debug, FromRepr, EnumIter, PartialEq)]
#[repr(u8)]
pub enum CellVersion {
    Single = 0,
    Angle = 1,
    Line = 2,
    Triple = 3,
}

impl Distribution<CellVersion> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CellVersion {
        CellVersion::from_repr(rng.gen_range(0..=3)).unwrap()
    }
}

#[derive(Clone, Copy, Debug, FromRepr, EnumIter, PartialEq)]
#[repr(u8)]
pub enum CellOrientation {
    North = 0,
    East = 1,
    South = 2,
    West = 3,
}

impl Distribution<CellOrientation> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> CellOrientation {
        CellOrientation::from_repr(rng.gen_range(0..=3)).unwrap()
    }
}


impl CellOrientation{
    pub fn rotate(&self, direction: CellOrientation) -> CellOrientation {
        CellOrientation::from_repr(((*self as u8) + (direction as u8)) % 4).unwrap()
    }

    pub fn shift(&self) -> (isize, isize) {
        match self {
            CellOrientation::North => (0, 1),
            CellOrientation::East => (1, 0),
            CellOrientation::South => (0, -1),
            CellOrientation::West => (-1, 0),
        }
    }

    pub fn reverse(&self) -> CellOrientation{
        self.rotate(CellOrientation::from_repr(2).unwrap())
    }

    pub fn step_from(&self, x: usize, y:usize) -> Option<(usize, usize)>{
        let (sx, sy) = self.shift();
        let (nx, ny) = (x as isize + sx, y as isize + sy);
        if nx >= 0 && ny >= 0{
            Some((nx as usize, ny as usize))
        } else {
            None
        }
    }

}

#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub version: CellVersion,
    pub orientation: CellOrientation,
    pub locked: bool,
    pub powered: bool,
}

impl Distribution<Cell> for Standard{
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Cell {
        Cell {
            version: rng.gen(),
            orientation: rng.gen(),
            locked: false,
            powered: false
        }
    }
}

impl Cell{
    pub fn connects(&self) -> Vec<CellOrientation>{
        match self.version{
            CellVersion::Single => vec![CellOrientation::North],
            CellVersion::Angle => vec![CellOrientation::North, CellOrientation::East],
            CellVersion::Line => vec![CellOrientation::North, CellOrientation::South],
            CellVersion::Triple => vec![CellOrientation::East, CellOrientation::South, CellOrientation::West]
        }.iter().map(|v| v.rotate(self.orientation))
            .collect()
    }

    pub fn matches_constraints(&self, constraints: &Vec<(CellOrientation, bool)>) -> bool{
        let cell_connections = self.connects();
        constraints.iter().all(|(dir, connects)|
            if *connects {
                cell_connections.iter().any(|d| *d == *dir)
            } else {
                cell_connections.iter().all(|d| *d != *dir)
            }
        )
    }

    pub fn all_possible() -> Vec<Cell>{
        iproduct!(CellVersion::iter(), CellOrientation::iter()).map(|(version, orientation)| Cell{
            version,
            orientation,
            locked: false,
            powered: false
        }).collect()
    }

    pub fn random_that_matches_constraints(constraints: Vec<(CellOrientation, bool)>) -> Cell {
        Cell::all_possible()
            .iter()
            .filter(|c| c.matches_constraints(&constraints))
            .choose(&mut rand::thread_rng())
            .expect(format!("No cell configuration can match those constraints : {:?}", constraints).as_str())
            .to_owned()
    }
}

pub struct Game {
    pub width: usize,
    pub height: usize,
    grid: Vec<Cell>,
}

impl Game {
    pub fn new(width: usize, height: usize) -> Game {
        Game {
            width,
            height,
            grid: vec![
                Cell {
                    version: CellVersion::Single,
                    orientation: CellOrientation::North,
                    locked: false,
                    powered: false
                };
                width * height
            ],
        }
    }

    pub fn cells(&self) -> impl Iterator<Item = (usize, usize, &Cell)>{
        iproduct!(0..self.height, 0..self.width).map(|(x, y)| (x, y, self.get_cell(x, y).unwrap()))
    }

    pub fn random_invalid(width: usize, height: usize) -> Game {
        let mut game = Game::new(width, height);
        let mut rng = rand::thread_rng();
        game.grid = (0..(width * height)).map(|_| rng.gen()).collect();
        game
    }


    pub fn random_valid(width:usize, height: usize) -> Game {
        let mut rng = rand::thread_rng();
        // Start with an empty canvas
        let mut game = Game::new(width, height);
        // When a cell is generated, lock it, then, add the neighbors to the explore queue
        let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
        // Start with the center piece
        queue.push_front((width/2, height/2));
        loop{
            while let Some((x, y)) = queue.pop_front(){
                // List the hard constraints from cells that are already locked
                let constraints = game.get_cell_constraints(x, y).into_iter()
                    .chain(
                        // Add the cells that are on the queue as they will be reached from another branch
                        game.get_neighbors(x, y).iter()
                            .filter_map(|(dir, cell)|
                                if !cell.locked && queue.iter().any(|pos| *pos == dir.step_from(x, y).unwrap()){
                                    Some((*dir, false))
                                } else { None }
                            )
                    ).collect();

                // Pick a random cell that matches those constraints
                let cell = Cell::random_that_matches_constraints(constraints);

                // println!("Locked cell at ({}, {}) - {:?} - {:?}", x, y, cell.version, cell.orientation);

                // Set the new cell
                game.set_cell(x, y, cell).unwrap();
                game.set_lock(x, y, true).unwrap();

                // Add the connected cells that are not already locked to the queue
                for (nx, ny) in game.get_non_locked_connections(x, y){
                    queue.push_back((nx, ny));
                }
            }

            // If the queue is empty, either the whole board is locked, ore some areas are unreachable
            // We look for an unlocked cell with a non-triple locked neighbor and pick one at random
            // println!("No more cells in the queue. Checking if we missed some");
            if let Some((x, y, dir)) = game.cells().filter_map(|(x, y, c)|
                if c.locked { None }
                else {
                    //Found an unlocked cell. Does it have a locked neighbor where we could plug it (so, non-triple)?
                    game.get_neighbors(x, y).iter()
                        .find_map(|(dir, nc)|
                            if nc.locked && nc.version != CellVersion::Triple { Some((x, y, *dir)) }
                            else {None})
                }
                // Now pick one of those cells at random
            ).choose(&mut rng) {
                // Found an unlocked cell that is next to a locked cell
                // println!("Manually forcing a way to fill ({}, {}) by changing the cell at its {:?}", x, y, dir);
                let (lx, ly) = dir.step_from(x, y).unwrap(); // Coords of the locked cell
                game.set_lock(lx, ly, false).unwrap();
                // Get its constraints, add the fact that the unlocked cell MUST be reached
                let mut constraints = game.get_cell_constraints(lx, ly);
                constraints.push((dir.reverse(), true));
                // Find a new cell configuration with those constraints
                game.set_cell(lx, ly, Cell::random_that_matches_constraints(constraints)).unwrap();
                // Lock it
                game.set_lock(lx, ly, true).unwrap();
                // Add newly accessible cells to the list
                for (nx, ny) in game.get_non_locked_connections(lx, ly){
                    queue.push_back((nx, ny));
                }

            }
            else {
                // No unlocked cells, break out of the loop, our job is done
                break;
            }
        }
        game
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.width || y >= self.height{
            None
        } else {
            self.grid.get(x + self.width * y)
        }
    }

    pub fn set_cell(&mut self, x:usize, y:usize, cell:Cell) -> Result<(), GameError>{
        if self.get_cell(x, y).is_some(){
            self.grid[x + self.width * y] = cell;
            Ok(())
        } else {
            Err(GameError::InvalidCell)
        }
    }

    fn get_mut_cell(&mut self, x: usize, y: usize) -> Result<&mut Cell, GameError> {
        if let Some(cell) = self.grid.get_mut(x + self.width * y) {
            Ok(cell)
        } else {
            Err(GameError::InvalidCell)
        }
    }

    pub fn get_neighbors(&self, x:usize, y:usize) -> Vec<(CellOrientation, &Cell)>{
        CellOrientation::iter()
            .filter_map(|dir|
                if let Some(cell) = self.get_neighbor_at_direction(x, y, dir) {
                    Some((dir, cell))
                } else {
                    None
                })
            .collect()
    }

    pub fn get_neighbor_at_direction(&self, x: usize, y:usize, direction: CellOrientation) -> Option<&Cell>{
        if let Some((nx, ny)) = direction.step_from(x, y){
            self.get_cell(nx as usize, ny as usize)
        } else {
            None
        }
    }

    pub fn get_non_locked_connections(&self, x:usize, y:usize) -> Vec<(usize, usize)> {
        self.get_cell(x, y).unwrap()
            .connects().iter()
            .filter_map(|dir|
                if let Some(neigh) = self.get_neighbor_at_direction(x, y, *dir) {
                    if !neigh.locked { Some(dir.step_from(x, y).unwrap()) } else { None }
                } else { None }
            ).collect()
    }

    pub fn get_cell_constraints(&self, x: usize, y:usize) -> Vec<(CellOrientation, bool)>{
        CellOrientation::iter().filter_map(|dir|
            // If there is a neighbor in this direction
            if let Some(c) = self.get_neighbor_at_direction(x, y, dir){
                if c.locked {
                    // Check if it must be connected or not-connected
                    Some((dir, c.connects().iter().any(|d| d.reverse() == dir)))
                } else {
                    // If the cell is not locked, we do what we want
                    None
                }
            } else {
                // If the cell does not exist (hence, we reached a wall), we must not connect to it
                Some((dir, false))
            }
        ).collect()
    }

    pub fn set_cell_orientation(
        &mut self,
        x: usize,
        y: usize,
        orientation: CellOrientation,
    ) -> Result<(), GameError> {
        self.get_mut_cell(x, y).and_then(|cell| {
            if cell.locked {
                Err(GameError::CellIsLocked)
            } else {
                Ok(cell.orientation = orientation)
            }
        })
    }

    pub fn set_lock(&mut self, x: usize, y: usize, lock: bool) -> Result<(), GameError> {
        self.get_mut_cell(x, y)
            .and_then(|cell| Ok(cell.locked = lock))
    }



    pub fn power_cell(&mut self, x: usize, y: usize) -> Result<(), GameError> {
        self.get_mut_cell(x, y)
            .and_then(|cell| Ok(cell.powered = true))
    }
}
