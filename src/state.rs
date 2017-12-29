pub type ID = usize;

pub type Point = (f32, f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Docking, Docked, Undocked, Undocking
}

#[derive(Debug)]
pub struct Player {
    pub id: ID,
    pub ships: Vec<ID>,
}

#[derive(Debug)]
pub struct Ship {
    pub id: ID,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub rad: f32,
    pub status: Status,
    pub planet: Option<ID>,
    pub progress: i32,
}

#[derive(Debug)]
pub struct Planet {
    pub id: ID,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub rad: f32,
    pub spots: i32,
    pub spawn: i32,
    pub owner: Option<ID>,
    pub ships: Vec<usize>,
}
