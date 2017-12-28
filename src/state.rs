use fnv::{FnvBuildHasher};
use std::collections::hash_map::HashMap;

pub type ID = usize;

pub enum Status {
    Docking, Docked, Undocked, Undocking  
}

pub trait Entity {
    fn pos(&self) -> (f32, f32);
    fn rad(&self) -> f32;
    fn hp(&self) -> i32;
}

pub struct Player {
    pub id: ID,
    pub ships: Vec<ID>,
}

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

impl Entity for Ship {
    fn hp(&self) -> i32 { self.hp }
    fn pos(&self) -> (f32, f32) { (self.x, self.y) }
    fn rad(&self) -> f32 { self.rad }
}

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

impl Entity for Planet {
    fn hp(&self) -> i32 { self.hp }
    fn pos(&self) -> (f32, f32) { (self.x, self.y) }
    fn rad(&self) -> f32 { self.rad }
}

pub struct Map {
    pub players: Vec<Player>,
    pub planets: Vec<Planet>,
    pub ships: HashMap<ID, Ship, FnvBuildHasher>,
}
