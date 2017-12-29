use fnv::{FnvBuildHasher, FnvHashMap};
use std::collections::hash_map::HashMap;
use state::*;
use constants::{X_GRID_SCALE, Y_GRID_SCALE};

#[derive(Debug, Copy, Clone)]
pub struct Location {
    pub id: ID,
    pub x: f32,
    pub y: f32,
    pub rad: f32,
}

#[derive(Debug, Copy, Clone)]
pub enum Entity {
    Ship(Location),
    Planet(Location),
    Point(Location),
}

impl Entity {
    pub fn id(&self) -> ID {
        use self::Entity::*;
        match *self { Ship(ref l) | Planet(ref l) | Point(ref l) => l.id, }
    }

    pub fn pos(&self) -> (f32, f32) {
        use self::Entity::*;
        match *self { Ship(ref l) | Planet(ref l) | Point(ref l) => (l.x, l.y) }
    }

    pub fn rad(&self) -> f32 {
        use self::Entity::*;
        match *self { Ship(ref l) | Planet(ref l) | Point(ref l) => l.rad, }
    }

    pub fn sq_distance_to(&self, other: &Self) -> f32 {
        let (x1, y1) = self.pos();
        let (x2, y2) = other.pos();
        (x2 - x1)*(x2 - x1) + (y2 - y1)*(y2 - y1)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let rad1 = self.rad(); 
        let rad2 = other.rad();
        self.sq_distance_to(other) < rad1 + rad2
    }
}

pub trait ToEntity {
    fn to_entity(&self) -> Entity;
}

impl ToEntity for Ship {
    fn to_entity(&self) -> Entity {
        Entity::Ship(Location {id: self.id, x: self.x, y: self.y, rad: self.rad})
    }
}

impl ToEntity for Planet {
    fn to_entity(&self) -> Entity {
        Entity::Planet(Location {id: self.id, x: self.x, y: self.y, rad: self.rad})
    }
}

impl ToEntity for Location {
    fn to_entity(&self) -> Entity { Entity::Point(*self) }
}

type Cell = (i32, i32);

const AROUND: [Cell; 9] = [
    (-1, -1), (-1, 0), (-1, 1), (0, -1),
    (0,0), (0, 1), (1, -1), (1, 0), (1, 1)
];

#[derive(Debug)]
pub struct HashGrid {
    scale: (f32, f32),
    grid: HashMap<Cell, Vec<Entity>, FnvBuildHasher>,
}

impl HashGrid {
    pub fn new() -> Self {
        HashGrid {
            scale: (X_GRID_SCALE, Y_GRID_SCALE),
            grid: FnvHashMap::default(),
        }
    }

    fn to_cell(&self, (x, y): Point) -> Cell {
        let (xs, ys) = self.scale;
        ((x / xs) as i32, (y / ys) as i32)
    }

    pub fn insert<T: ToEntity>(&mut self, e: &T) {
        let entity = e.to_entity();
        let cell = self.to_cell(entity.pos());
        self.grid.entry(cell)
            .and_modify(|e| e.push(entity))
            .or_insert(vec!(entity));
    }

    pub fn collides<T: ToEntity>(&self, e: &T) -> bool {
        let entity = e.to_entity();
        let (x, y) = self.to_cell(entity.pos());
        let mut near = Vec::new();

        for &(xo, yo) in &AROUND {
            match self.grid.get(&(x + xo, y + yo)) {
                Some(bucket) => near.extend(bucket.iter()),
                None => continue,
            }
        }

        near.iter().any(|other| entity.intersects(other))
    }
}
