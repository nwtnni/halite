pub struct State {
    pub id: usize,
    pub width: usize,  
    pub height: usize,
    pub players: Vec<Player>,
    pub halite: Vec<usize>,
}

pub struct Player {
    pub id: usize,
    pub yard: Shipyard, 
    pub drops: Vec<Dropoff>,
    pub ships: Vec<Ship>,
    pub halite: usize,
}

pub struct Ship {
    pub owner: usize,
    pub id: usize,
    pub x: usize,
    pub y: usize,
    pub halite: usize,
}

pub struct Shipyard {
    pub owner: usize,
    pub id: usize,
    pub x: usize,
    pub y: usize,
}

pub struct Dropoff {
    pub owner: usize,
    pub id: usize,
    pub x: usize,
    pub y: usize,
}
