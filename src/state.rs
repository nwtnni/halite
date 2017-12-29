use fnv::{FnvBuildHasher};
use std::collections::hash_map::HashMap;
use collision::HashGrid;

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

#[derive(Debug)]
pub struct Map {
    pub players: Vec<Player>,
    pub planets: Vec<Planet>,
    pub ships: HashMap<ID, Ship, FnvBuildHasher>,
    pub grid: HashGrid,
}
