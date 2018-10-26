pub type PID = u8;
pub type ID = u16;
pub type Dist = i8;
pub type Time = i16;
pub type Halite = i32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(pub Dist, pub Dist);

#[derive(Debug, Clone)]
pub struct State {
    pub id: PID,
    pub width: Dist,  
    pub height: Dist,
    pub round: Time,
    pub scores: Vec<Halite>,
    pub drops: Vec<Dropoff>,
    pub ships: Vec<Ship>,
    pub yards: Vec<Shipyard>, 
    pub halite: Vec<Halite>,
}

impl State {
    pub fn halite(&self) -> Halite {
        self.scores[self.id as usize]
    }

    pub fn allies(&self) -> impl Iterator<Item = &Ship> {
        self.ships.iter().filter(move |ship| ship.owner == self.id)
    }

    pub fn enemies(&self) -> impl Iterator<Item = &Ship> {
        self.ships.iter().filter(move |ship| ship.owner != self.id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ship {
    pub owner: u8,
    pub id: u16,
    pub x: i8,
    pub y: i8,
    pub halite: Halite,
}

impl <'a> Into<Pos> for &'a Ship {
    fn into(self) -> Pos {
        Pos(self.x, self.y)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Shipyard {
    pub owner: u8,
    pub x: i8,
    pub y: i8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dropoff {
    pub owner: u8,
    pub x: i8,
    pub y: i8,
}
