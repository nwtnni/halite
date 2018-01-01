use fnv::FnvHashMap;
use hlt::state::*;
use hlt::constants::{X_GRID_SCALE, Y_GRID_SCALE, SHIP_RADIUS, FUDGE};

#[derive(Debug, Copy, Clone)]
pub struct Location {
    pub x: f32,
    pub y: f32,
    pub rad: f32,
    pub id: ID,
}

#[derive(Debug, Copy, Clone)]
pub enum Entity {
    Ship(Location),
    Planet(Location),
    Point(Location),
}

pub fn within(a: Point, ar: f32, b: Point, br: f32, r: f32) -> bool {
    let (x1, y1) = a;
    let (x2, y2) = b;
    let d = (y2 - y1).hypot(x2 - x1);
    d <= ar + br + r
}

impl Entity {
    pub fn pos(&self) -> (f32, f32) {
        use hlt::collision::Entity::*;
        match *self { Ship(ref l) | Planet(ref l) | Point(ref l) => (l.x, l.y) }
    }

    pub fn rad(&self) -> f32 {
        use hlt::collision::Entity::*;
        match *self { Ship(ref l) | Planet(ref l) | Point(ref l) => l.rad, }
    }
    
    pub fn id(&self) -> ID {
        use hlt::collision::Entity::*;
        match *self { Ship(ref l) | Planet(ref l) | Point(ref l) => l.id, }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        within(self.pos(), self.rad(), other.pos(), other.rad(), 0.0)
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
        (t1 >= 0.0 && t1 <= 1.0) || (t2 >= 0.0 && t2 <= 1.0) || (t1 <= 0.0 && t2 >= 1.0)
    }
}

pub trait ToEntity {
    fn to_entity(&self) -> Entity;
}

impl ToEntity for Ship {
    fn to_entity(&self) -> Entity {
        Entity::Ship(Location {x: self.x, y: self.y, rad: self.rad, id: self.id})
    }
}

impl ToEntity for Planet {
    fn to_entity(&self) -> Entity {
        Entity::Planet(Location {x: self.x, y: self.y, rad: self.rad, id: self.id})
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

#[derive(Debug, Default)]
pub struct Grid {
    scale: (f32, f32),
    grid: FnvHashMap<Cell, Vec<Entity>>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
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
        self.grid.entry(cell).or_insert(vec!(entity));
        self.grid.get_mut(&cell).unwrap().push(entity);
    }

    pub fn remove(&mut self, e: Point) {
        let cell = self.to_cell(e);
        self.grid.entry(cell).or_insert(Vec::new());
        self.grid.get_mut(&cell).unwrap().retain(|other| other.pos() != e);
    }

    pub fn near<T: ToEntity>(&self, e: &T) -> Vec<Entity> {
        let entity = e.to_entity();
        let (x, y) = self.to_cell(entity.pos());
        let mut near = Vec::new();
        for &(xo, yo) in &AROUND {
            match self.grid.get(&(x + xo, y + yo)) {
                Some(bucket) => near.extend(bucket.iter()),
                None => continue,
            }
        }
        near
    }

    pub fn near_enemies<T: ToEntity>(&self, e: &T, player: ID, ships: &Ships) -> i32 {
        self.near(e)
            .iter()
            .filter(|&&other| match other {
                Entity::Ship(Location {x: _, y: _, rad: _, id}) => {
                    ships.get(&id).map_or(false, |ship| ship.owner != player)
                },
                _ => false, 
            }).count() as i32
    }

    pub fn collides_toward<T: ToEntity>(&self, e: &T, end: Point) -> bool {
        let start = e.to_entity().pos();
        self.near(e)
            .iter()
            .any(|other| start != other.pos()
                 && other.intersects_line(start, end))
    }
}

mod tests {
    #![cfg(test)]
    use super::*;

    #[test]
    fn test_insert() {
        let mut grid = Grid::new();
        grid.insert(&Location {x: 12.0, y: 12.0, rad: 12.0});
    }

    #[test]
    fn test_largest_planet() {
        let mut grid = Grid::new();
        let p1 = Location {x: 12.0, y: 12.0, rad: 16.0};
        let p2 = Location {x: 44.0, y: 12.0, rad: 16.0};
        let p3 = Location {x: 44.001, y: 12.0, rad: 16.0};
        grid.insert(&p1);
        assert_eq!(grid.collides(&p2), true);
        assert_eq!(grid.collides(&p3), false);
    }

    #[test]
    fn test_ship() {
        let mut grid = Grid::new();
        let s1 = Location {x: 383.5, y: 255.5, rad: 0.5};
        let s2 = Location {x: 383.5, y: 254.5, rad: 0.5};
        let s3 = Location {x: 383.5, y: 254.4999, rad: 0.5};
        grid.insert(&s1);
        assert_eq!(grid.collides(&s2), true);
        assert_eq!(grid.collides(&s3), false);
    }

    #[test]
    fn test_circle() {
        let circle = Entity::Point(Location { x: 0.0, y: 0.0, rad: 5.0 });
        assert!(circle.intersects_line((-5.0, 5.0), (5.0, 5.0)));
        assert!(circle.intersects_line((-5.0, 5.0), (0.0, 5.0)));
        assert!(circle.intersects_line((0.0, 5.0), (5.0, 5.0)));
    }

    #[test]
    fn test_offset_circle() {
        let circle = Entity::Point(Location { x: 5.0, y: 5.0, rad: 5.0 });
        assert!(circle.intersects_line((-10.0, 1.0), (10.0, 10.0)));
        assert!(circle.intersects_line((0.0, 0.0), (0.0, 10.0)));
        assert!(circle.intersects_line((0.0, 0.0), (10.0, 0.0)));
    }

    #[test]
    fn test_no_collision() {
        let circle = Entity::Point(Location { x: 5.0, y: 5.0, rad: 0.0 });
        assert!(!circle.intersects_line((0.0, 0.0), (1.0, 1.0)));
    }
}
