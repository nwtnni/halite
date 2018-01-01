use fnv::FnvHashMap;
use std::io::stdin;
use hlt::parse::*;
use hlt::constants::{SHIP_SPEED, DOCK_RADIUS};
use hlt::collision::*;

pub type ID = usize;
pub type Point = (f32, f32);
pub type Planets = FnvHashMap<ID, Planet>;
pub type Ships = FnvHashMap<ID, Ship>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Docking, Docked, Undocked, Undocking
}

impl Status {
    pub fn value(&self) -> f32 {
        match *self {
            Status::Docking => SHIP_SPEED * 16.0,
            Status::Docked => SHIP_SPEED * 9.0,
            Status::Undocking => SHIP_SPEED * 4.0,
            Status::Undocked => 0.0,
        }
    }
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
    pub owner: ID,
}

impl Ship {
    pub fn value(&self) -> f32 {
        self.status.value()
    }

    pub fn is_docked(&self) -> bool {
        self.status == Status::Docked
        || self.status == Status::Docking
    }

    pub fn in_docking_range(&self, p: &Planet) -> bool {
        within((self.x, self.y), self.rad, (p.x, p.y), p.rad, DOCK_RADIUS)
    }
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

impl Planet {
    pub fn value(&self) -> f32 {
        SHIP_SPEED * (self.spots as f32)
    }

    pub fn is_enemy(&self, id: ID) -> bool {
        match self.owner {
            Some(other) => id != other,
            None => false,
        }
    }

    pub fn has_spots(&self) -> bool {
        self.spots > (self.ships.len() as i32) 
    }
}

#[derive(Debug)]
pub struct Game {
    pub id: ID,
    pub width: f32,
    pub height: f32,
    pub players: Vec<Player>,
    pub planets: Planets,
    pub ships: Ships,
    pub grid: Grid,
}

impl Game {
    pub fn new() -> Self {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        let id = usize::take(&mut stream);
        let width = f32::take(&mut stream);
        let height = f32::take(&mut stream);
        let (players, planets, ships, grid) = take(&mut stream);
        Game { id, width, height, players, planets, ships, grid }
    }

    pub fn update(&mut self) {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        let (players, planets, ships, grid) = take(&mut stream);
        self.players = players;
        self.planets = planets;
        self.ships = ships;
        self.grid = grid;
    }

    pub fn send_ready(name: &str) {
        println!("{}", name);
    }
}
