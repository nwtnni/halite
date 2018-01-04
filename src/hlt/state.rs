use fnv::FnvHashMap;
use std::f64;
use std::io::stdin;
use hlt::parse::*;
use hlt::strategy::Plan;
use hlt::command::Queue;
use hlt::constants::DOCK_RADIUS;
use hlt::collision::*;

pub type ID = usize;
pub type Point = (f64, f64);
pub type Planets = FnvHashMap<ID, Planet>;
pub type Ships = FnvHashMap<ID, Ship>;

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
    pub x: f64,
    pub y: f64,
    pub hp: i32,
    pub rad: f64,
    pub status: Status,
    pub planet: Option<ID>,
    pub progress: i32,
    pub owner: ID,
}

impl Ship {
    pub fn is_docked(&self) -> bool {
        self.status != Status::Undocked
    }

    pub fn in_docking_range(&self, p: &Planet) -> bool {
        (p.y - self.y).hypot(p.x - self.x) <= self.rad + p.rad + DOCK_RADIUS
    }

    pub fn retreat_from(&self, e: &Ship, d: i32) -> Point {
        let angle = f64::atan2(e.y - self.y, e.x - self.x);
        (self.x - (d as f64)*angle.cos(), self.y - (d as f64)*angle.sin())
    }

    pub fn distance_to<T: ToEntity>(&self, e: &T) -> f64 {
        let (x, y) = e.to_entity().pos();
        (y - self.y).hypot(x - self.x)
    }
}

#[derive(Debug)]
pub struct Planet {
    pub id: ID,
    pub x: f64,
    pub y: f64,
    pub hp: i32,
    pub rad: f64,
    pub spots: i32,
    pub spawn: i32,
    pub owner: Option<ID>,
    pub ships: Vec<usize>,
}

impl Planet {
    pub fn value(&self) -> f64 {
        self.owner.map_or(35.0, |_| 0.0)
        + (self.spots as f64)
    }

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
        self.spots > (self.ships.len() as i32)
    }

    pub fn spots(&self) -> i32 {
        self.spots - (self.ships.len() as i32)
    }

    pub fn docked(&self) -> i32 {
        self.ships.len() as i32
    }
}

pub struct State {
    pub id: ID,
    pub width: f64,
    pub height: f64,
    pub grid: Grid,
    pub players: Vec<Player>,
    pub planets: Planets,
    pub plan: Plan,
    pub ships: Ships,
    pub queue: Queue,
}

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
        let plan = Plan::new();
        let queue = Queue::new();
        let (players, planets, ships, grid) = take(&mut stream);
        State { id, width, height, players, planets,
                ships, plan, queue, grid }
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
        self.grid.owner = self.id;
        self.plan = Plan::new();
    }

    pub fn send_ready(name: &str) {
        println!("{}", name);
    }
}
