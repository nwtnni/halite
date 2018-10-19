#[derive(Debug, Clone)]
pub struct State {
    pub id: usize,
    pub width: usize,  
    pub height: usize,
    pub round: usize,
    pub scores: Vec<usize>,
    pub drops: Vec<Dropoff>,
    pub ships: Vec<Ship>,
    pub yards: Vec<Shipyard>, 
    pub halite: Vec<usize>,
}

impl State {
    pub fn halite(&self) -> usize {
        self.scores[self.id]
    }

    pub fn allies(&self) -> impl Iterator<Item = &Ship> {
        self.ships.iter().filter(move |ship| ship.owner == self.id)
    }

    pub fn enemies(&self) -> impl Iterator<Item = &Ship> {
        self.ships.iter().filter(move |ship| ship.owner != self.id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ship {
    pub owner: usize,
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub halite: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Shipyard {
    pub owner: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Dropoff {
    pub owner: usize,
    pub x: usize,
    pub y: usize,
}
