use fnv::FnvHashMap;
use std::f32::consts::SQRT_2;
use hlt::state::*;
use hlt::constants::{GRID_SCALE, GRID_SCALE_2, SHIP_RADIUS, FUDGE};

type Cell = (i32, i32);

#[derive(Debug, Copy, Clone)]
pub enum Entity {
    Ship((f32, f32, f32, ID)),
    Planet((f32, f32, f32, ID)),
    Beacon((f32, f32)),
    Obstacle((f32, f32, f32)),
}

impl Entity {
    pub fn pos(&self) -> (f32, f32) {
        use hlt::collision::Entity::*;
        match *self {
            Ship((x, y, _, _))
            | Planet((x, y, _, _))
            | Beacon((x, y))
            | Obstacle((x, y, _)) => (x, y)
        }
    }

    pub fn rad(&self) -> f32 {
        use hlt::collision::Entity::*;
        match *self {
            Ship((_, _, r, _)) 
            | Planet((_, _, r, _))
            | Obstacle((_, _, r)) => r,
            Beacon(_) => SHIP_RADIUS
        }
    }

    pub fn key(&self) -> ID {
        use hlt::collision::Entity::*;
        match *self {
            Ship((_, _, _, id)) => id,
            Planet((_, _, _, id)) => id + 10000,
            Beacon(_) | Obstacle(_) => 100000,
        }
    }

    pub fn to_ship(&self) -> Option<(f32, f32, ID)> {
        match *self {
            Entity::Ship((x, y, _, id)) => Some((x, y, id)),
            _ => None,
        }
    }

    pub fn to_planet(&self) -> Option<(f32, f32, ID)> {
        match *self {
            Entity::Planet((x, y, _, id)) => Some((x, y, id)),
            _ => None,
        }
    }

    pub fn to_beacon(&self) -> Option<(f32, f32, ID)> {
        match *self {
            Entity::Beacon((x, y)) => Some((x, y, 100000)),
            _ => None,
        }
    }

    // From https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm
    pub fn intersects_line(&self, (x1, y1): Point, (x2, y2): Point) -> bool {
        let (x, y) = self.pos();
        let r = self.rad() + SHIP_RADIUS*FUDGE;
        let (dx, dy) = (x2 - x1, y2 - y1);
        let a = dx*dx + dy*dy;
        let b = 2.0 * ((x1 - x)*dx + (y1 - y)*dy);
        let c = (x1 - x)*(x1 - x) + (y1 - y)*(y1 - y) - r*r;
        let d = b*b - 4.0*a*c;
        if d < 0.0 { return false }

        let d = d.sqrt();
        let (t1, t2) = ((-b - d)/(2.0*a), (-b + d)/(2.0*a));
        (t1 >= 0.0 && t1 <= 1.0) || (t2 >= 0.0 && t2 <= 1.0)
    }
}

pub trait ToEntity {
    fn to_entity(&self) -> Entity;
}

impl<'a> ToEntity for &'a Ship {
    fn to_entity(&self) -> Entity {
        Entity::Ship((self.x, self.y, self.rad, self.id))
    }
}

impl<'a> ToEntity for &'a Planet {
    fn to_entity(&self) -> Entity {
        Entity::Planet((self.x, self.y, self.rad, self.id))
    }
}

impl ToEntity for Entity {
    fn to_entity(&self) -> Entity { *self }
}

#[derive(Debug, Default)]
pub struct Grid {
    pub id: ID,
    place: FnvHashMap<ID, Vec<Cell>>,
    grid: FnvHashMap<Cell, Vec<Entity>>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            id: 0,
            place: FnvHashMap::default(),
            grid: FnvHashMap::default(),
        }
    }

    fn to_cell(x: f32, y: f32) -> Cell {
        ((x / GRID_SCALE) as i32, (y / GRID_SCALE) as i32)
    }

    fn to_cells((x, y): Point, r: f32) -> Vec<Cell> {
        let mut cells = Vec::new();
        let (mut x1, mut y1) = (x - SQRT_2*r, y - SQRT_2*r);
        let (x2, y2) = (x + SQRT_2*r, y + SQRT_2*r);
        while x1 < x2 {
            let mark = y1;
            while y1 < y2 {
                // From https://stackoverflow.com/questions/401847/circle-rectangle-collision-detection-intersection
                let cx = (x - (x1 + GRID_SCALE_2)).abs();
                let cy = (y - (y1 + GRID_SCALE_2)).abs();
                if cx > GRID_SCALE_2 + r || cy > GRID_SCALE_2 + r {
                } else if cx <= GRID_SCALE_2 || cy <= GRID_SCALE_2
                || (cx - GRID_SCALE_2).hypot(cy - GRID_SCALE_2) <= r {
                    cells.push(Self::to_cell(x1, y1));
                }
                y1 += GRID_SCALE;
            }
            y1 = mark;
            x1 += GRID_SCALE;
        }
        cells.push(Self::to_cell(x, y));
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    pub fn insert<T: ToEntity>(&mut self, e: &T) {
        let entity = e.to_entity();
        let cells = Self::to_cells(entity.pos(), entity.rad());
        for &cell in &cells {
            self.grid.entry(cell).or_insert(vec!(entity));
            self.grid.get_mut(&cell).unwrap().push(entity);
        }
        self.place.insert(entity.key(), cells);
    }

    pub fn remove<T: ToEntity>(&mut self, e: &T) {
        let entity = e.to_entity();
        let key = entity.key();
        let cells = self.place.remove(&key).expect("Illegal remove");
        for cell in cells {
            self.grid.entry(cell).or_insert(Vec::new());
            self.grid.get_mut(&cell).unwrap().retain(|other| other.key() != key);
        }
    }

    fn near<'a, T: ToEntity>(&'a self, e: &T, r: f32) -> Vec<&'a Entity> {
        let entity = e.to_entity();
        let key = entity.key();
        let cells = Self::to_cells(entity.pos(), r);
        let mut nearby = cells.iter()
            .filter_map(|cell| self.grid.get(cell))
            .flat_map(|ref bucket| bucket.iter())
            .filter(|&other| other.key() != key)
            .collect::<Vec<_>>();
        nearby.sort_unstable_by_key(|&entity| entity.key());
        nearby.dedup_by_key(|&mut entity| entity.key());
        nearby
    }

    fn near_entity<F, T: ToEntity>(&self, e: &T, r: f32, f: F) -> Vec<(f32, f32, ID)>
        where F: Fn(&Entity)-> Option<(f32, f32, ID)> {
        let (x1, y1) = e.to_entity().pos();
        let mut nearby = self.near(e, r)
            .into_iter()
            .filter_map(|entity| f(entity))
            .collect::<Vec<_>>();
        nearby.sort_unstable_by_key(|&(x2, y2, _)| (y2 - y1).hypot(x2 - x1) as i32);
        nearby
    }

    pub fn near_enemies<'a, T: ToEntity>(&self, e: &T, r: f32, ships: &'a Ships)
        -> Vec<&'a Ship> {
        self.near_entity(e, r, |entity| entity.to_ship())
            .into_iter()
            .filter_map(|(_, _, id)| { ships.get(&id).map_or(None, |ship| {
                if ship.owner != self.id { Some(ship) } else { None }
            })}).collect()
    }

    pub fn near_allies<'a, T: ToEntity>(&self, e: &T, r: f32, ships: &'a Ships)
        -> Vec<&'a Ship> {
        self.near_entity(e, r, |entity| entity.to_ship())
            .into_iter()
            .filter_map(|(_, _, id)| { ships.get(&id).map_or(None, |ship| {
                if ship.owner == self.id { Some(ship) } else { None }
            })}).collect()
    }

    pub fn near_planets<'a, T: ToEntity>(&self, e: &T, r: f32, planets: &'a Planets)
        -> Vec<&'a Planet> {
        self.near_entity(e, r, |entity| entity.to_planet())
            .into_iter()
            .filter_map(|(_, _, id)| {
                planets.get(&id).map_or(None, |planet| { Some(planet) })
            }).collect()
    }

    pub fn near_beacons<'a, T: ToEntity>(&self, e: &T, r: f32)
        -> Vec<(f32, f32)> {
        self.near_entity(e, r, |entity| entity.to_beacon()) 
            .into_iter()
            .map(|(x, y, _)| (x, y))
            .collect()
    }

    pub fn collides_toward<T: ToEntity>(&self, e: &T, (x2, y2): Point) -> bool {
        let entity = e.to_entity();
        let key = entity.key();
        let (x1, y1) = entity.pos();
        let r = (y2 - y1).hypot(x2 - x1);
        self.near(e, r)
            .into_iter()
            .any(|&other| {
                other.key() != key && other.intersects_line((x1, y1), (x2, y2))
            })
    }
}

mod tests {
    #![cfg(test)]
    use hlt::collision::*;

    #[test]
    fn test_insert() {
        let mut grid = Grid::new();
        grid.insert(&Location {x: 12.0, y: 12.0, rad: 12.0, id:0});
    }

    #[test]
    fn test_circle() {
        let circle = Entity::Point(Location { x: 0.0, y: 0.0, rad: 5.0, id:0});
        assert!(circle.intersects_line((-5.0, 5.0), (5.0, 5.0)));
        assert!(circle.intersects_line((-5.0, 5.0), (0.0, 5.0)));
        assert!(circle.intersects_line((0.0, 5.0), (5.0, 5.0)));
    }

    #[test]
    fn test_offset_circle() {
        let circle = Entity::Point(Location { x: 5.0, y: 5.0, rad: 5.0, id:0});
        assert!(circle.intersects_line((-10.0, 1.0), (10.0, 10.0)));
        assert!(circle.intersects_line((0.0, 0.0), (0.0, 10.0)));
        assert!(circle.intersects_line((0.0, 0.0), (10.0, 0.0)));
    }

    #[test]
    fn test_no_collision() {
        let circle = Entity::Point(Location { x: 5.0, y: 5.0, rad: 0.0, id:0});
        assert!(!circle.intersects_line((0.0, 0.0), (1.0, 1.0)));
    }
}
