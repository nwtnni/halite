use fnv::FnvHashMap;
use std::f64;
use std::cmp::*;
use std::io::stdin;
use hlt::parse::*;
use hlt::command::Queue;
use hlt::constants::*;
use hlt::collision::*;
use hlt::scout::*;

pub type ID = usize;
pub type Point = (f64, f64);
pub type Planets = FnvHashMap<ID, Planet>;
pub type Ships = FnvHashMap<ID, Ship>;
pub type Docked = FnvHashMap<ID, ID>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Docking, Docked, Undocked, Undocking
}

#[derive(Debug)]
pub struct Player {
    pub id: ID,
    pub ships: Vec<ID>,
}

#[derive(Debug, Clone)]
pub struct Ship {
    pub id: ID,
    pub xp: f64,
    pub yp: f64,
    pub x: f64,
    pub y: f64,
    pub hp: i32,
    pub status: Status,
    pub planet: Option<ID>,
    pub progress: i32,
    pub owner: ID,
}

#[derive(Debug, Clone)]
pub struct Planet {
    pub id: ID,
    pub x: f64,
    pub y: f64,
    pub hp: i32,
    pub rad: f64,
    pub spots: usize,
    pub spawn: i32,
    pub owner: Option<ID>,
    pub ships: Vec<usize>,
}

pub struct State {
    pub id: ID,
    pub width: f64,
    pub height: f64,
    pub grid: Grid,
    pub players: Vec<Player>,
    pub planets: Planets,
    pub scout: Scout,
    pub ships: Ships,
    pub queue: Queue,
    pub docked: Docked,
}

impl Ship {
    pub fn is_docked(&self) -> bool {
        self.status != Status::Undocked
    }

    pub fn in_docking_range(&self, p: &Planet) -> bool {
        (p.y - self.y).hypot(p.x - self.x) <= SHIP_RADIUS + p.rad + DOCK_RADIUS
    }

    pub fn distance_to<T: ToEntity>(&self, e: &T) -> f64 {
        let (x, y) = e.to_entity().pos();
        (y - self.y).hypot(x - self.x)
    }

    pub fn is_facing(&self, s: &Ship) -> bool {
        let theta = f64::atan2(s.yp - self.yp, s.xp - self.xp);
        let phi = f64::atan2(self.y - self.yp, self.x - self.xp);
        (theta - phi).abs() < EPSILON
    }
}

impl PartialEq for Ship {
    fn eq(&self, other: &Ship) -> bool { self.id == other.id }
}

impl Eq for Ship {}

impl Planet {
    pub fn is_owned(&self, id: ID) -> bool {
        match self.owner {
            Some(body_once_told_me) => id == body_once_told_me,
            None => false,
        }
    }

    pub fn is_enemy(&self, id: ID) -> bool {
        match self.owner {
            Some(other) => id != other,
            None => false,
        }
    }

    pub fn has_spots(&self) -> bool {
        self.spots > self.ships.len()
    }

    pub fn spots(&self) -> usize {
        self.spots as usize - self.ships.len()
    }

    pub fn docked(&self) -> i32 {
        self.ships.len() as i32
    }

    pub fn will_spawn(&self, n: i32) -> i32 {
        (self.spawn + (self.docked() * PRODUCTIVITY * n)) / 12
    }
}

impl PartialEq for Planet {
    fn eq(&self, other: &Planet) -> bool { self.id == other.id }
}

impl Eq for Planet {}

impl State {
    pub fn new() -> Self {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        let id = usize::take(&mut stream);
        let width = f64::take(&mut stream);
        let height = f64::take(&mut stream);
        let queue = Queue::new();
        let docked = FnvHashMap::default();
        let (players, planets, ships, grid) = take(&mut stream);
        let scout = Scout::new(id, &ships, &planets);
        State { id, width, height, players, planets, scout,
                ships, queue, grid, docked}
    }

    pub fn update(&mut self) {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        let (players, planets, mut ships, grid) = take(&mut stream);
        self.players = players;
        self.planets = planets;
        self.grid = grid;
        self.grid.owner = self.id;
        self.grid.width = self.width;
        self.grid.height = self.height;
        for ship in &mut ships.values_mut() {
            if let Some(previous) = self.ships.get(&ship.id) {
                ship.xp = previous.x;
                ship.yp = previous.y;
            }
        }
        self.ships = ships;
        self.scout = Scout::new(self.id, &self.ships, &self.planets);
    }

    pub fn send_ready(name: &str) {
        println!("{}", name);
    }
}
