use fnv::FnvHashMap;
use std::i32;
use std::f64::consts::SQRT_2;
use hlt::state::*;
use hlt::constants::*;

type Cell = (i32, i32);

#[derive(Debug, Copy, Clone)]
pub enum Entity {
    Ship(f64, f64, f64, ID),
    Planet(f64, f64, f64, ID),
    Obstacle(f64, f64, f64, ID),
}

impl Entity {
    pub fn pos(&self) -> (f64, f64) {
        use hlt::collision::Entity::*;
        match *self {
            Ship(x, y, _, _)
            | Planet(x, y, _, _)
            | Obstacle(x, y, _, _) => (x, y),
        }
    }

    pub fn rad(&self) -> f64 {
        use hlt::collision::Entity::*;
        match *self {
            Ship(_, _, r, _)
            | Planet(_, _, r, _)
            | Obstacle(_, _, r, _) => r,
        }
    }

    fn key(&self) -> String {
        use hlt::collision::Entity::*;
        match *self {
            Ship(_, _, _, id) => "s".to_string() + &id.to_string(),
            Planet(_, _, _, id) => "p".to_string() + &id.to_string(),
            Obstacle(_, _, _, id) => "o".to_string() + &id.to_string(),
        }
    }

    fn to_ship(&self) -> Option<(f64, f64, ID)> {
        match *self {
            Entity::Ship(x, y, _, id) => Some((x, y, id)),
            _ => None,
        }
    }

    fn to_planet(&self) -> Option<(f64, f64, ID)> {
        match *self {
            Entity::Planet(x, y, _, id) => Some((x, y, id)),
            _ => None,
        }
    }

    // From https://stackoverflow.com/questions/1073336/circle-line-segment-collision-detection-algorithm
    pub fn intersects_line(&self, (x1, y1): Point, (x2, y2): Point) -> bool {
        let (x, y) = self.pos();
        let r = self.rad() + SHIP_RADIUS;
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

    pub fn intersects(&self, (x2, y2): Point, r: f64) -> bool {
        let (x1, y1) = self.pos();
        (y2 - y1).hypot(x2 - x1) <= self.rad() + r
    }
}

pub trait ToEntity {
    fn to_entity(&self) -> Entity;
}

impl<'a> ToEntity for &'a Ship {
    fn to_entity(&self) -> Entity {
        Entity::Ship(self.x, self.y, self.rad, self.id)
    }
}

impl<'a> ToEntity for &'a Planet {
    fn to_entity(&self) -> Entity {
        Entity::Planet(self.x, self.y, self.rad, self.id)
    }
}

impl ToEntity for Entity {
    fn to_entity(&self) -> Entity {
        *self
    }
}

#[derive(Debug, Default)]
pub struct Grid {
    pub owner: ID,
    count: ID,
    place: FnvHashMap<String, Vec<Cell>>,
    grid: FnvHashMap<Cell, Vec<Entity>>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            owner: 0,
            count: 0,
            place: FnvHashMap::default(),
            grid: FnvHashMap::default(),
        }
    }

    fn to_cell(x: f64, y: f64) -> Cell {
        ((x / GRID_SCALE) as i32, (y / GRID_SCALE) as i32)
    }

    fn to_cells((x, y): Point, r: f64) -> Vec<Cell> {
        let mut cells = Vec::new();
        let (x1, y1) = Self::to_cell(x - SQRT_2*r, y - SQRT_2*r);
        let (mut x1, mut y1) = (x1 as f64 * GRID_SCALE, y1 as f64 * GRID_SCALE);
        let (x2, y2) = Self::to_cell(x + SQRT_2*r, y + SQRT_2*r);
        let (x2, y2) = (x2 as f64 * GRID_SCALE, y2 as f64 * GRID_SCALE);
        while x1 <= x2 {
            let mark = y1;
            while y1 <= y2 {
                // From https://stackoverflow.com/questions/401847/circle-rectangle-collision-detection-intersection
                let (rx, ry) = (x1 + GRID_SCALE_2, y1 + GRID_SCALE_2);
                let (cx, cy) = ((x - rx).abs(), (y - ry).abs());
                if cx > GRID_SCALE_2 + r || cy > GRID_SCALE_2 + r {}
                else if cx <= GRID_SCALE_2 || cy <= GRID_SCALE_2
                || (cx - GRID_SCALE_2).hypot(cy - GRID_SCALE_2) <= r {
                    let cell = Self::to_cell(rx, ry);
                    cells.push(cell);
                }
                y1 += GRID_SCALE;
            }
            y1 = mark;
            x1 += GRID_SCALE;
        }
        cells
    }

    pub fn create_ship(&mut self, x: f64, y: f64, id: ID) {
        self.insert(&Entity::Ship(x, y, SHIP_RADIUS, id));
    }

    pub fn create_obstacle(&mut self, x: f64, y: f64, r: f64) {
        let id = self.count;
        self.insert(&Entity::Obstacle(x, y, r, id));
        self.count += 1;
    }

    pub fn insert<T: ToEntity>(&mut self, e: &T) {
        let entity = e.to_entity();
        let cells = Self::to_cells(entity.pos(), entity.rad());
        for &cell in &cells {
            self.grid.entry(cell).or_insert(Vec::new());
            self.grid.get_mut(&cell).unwrap().push(entity);
        }
        self.place.insert(entity.key(), cells);
    }

    pub fn remove<T: ToEntity>(&mut self, e: &T) {
        let entity = e.to_entity();
        let key = entity.key();
        let cells = self.place.remove(&key).expect("Illegal remove");
        for cell in cells {
            match self.grid.get_mut(&cell) {
                None => continue,
                Some(bucket) => bucket.retain(|other| other.key() != key),
            }
        }
    }

    fn near<'a, T: ToEntity>(&'a self, e: &T, r: f64) -> Vec<&'a Entity> {
        let entity = e.to_entity();
        let pos = entity.pos();
        let key = entity.key();
        let cells = Self::to_cells(pos, r);
        let mut nearby = cells.iter()
            .filter_map(|cell| self.grid.get(cell))
            .flat_map(|ref bucket| bucket.iter())
            .filter(|&other| other.key() != key && other.intersects(pos, r))
            .collect::<Vec<_>>();
        nearby.sort_unstable_by_key(|&entity| entity.key());
        nearby.dedup_by_key(|&mut entity| { entity.key() });
        nearby
    }

    fn near_entity<F, T: ToEntity>(&self, e: &T, r: f64, f: F) -> Vec<(f64, f64, ID)>
        where F: Fn(&Entity)-> Option<(f64, f64, ID)> {
        let (x1, y1) = e.to_entity().pos();
        let mut nearby = self.near(e, r)
            .into_iter()
            .filter_map(|entity| f(entity))
            .collect::<Vec<_>>();
        nearby.sort_unstable_by_key(|&(x2, y2, _)| (y2 - y1).hypot(x2 - x1) as i32);
        nearby
    }

    pub fn near_enemies<'a, T: ToEntity>(&self, e: &T, r: f64, ships: &'a Ships)
        -> Vec<&'a Ship> {
        self.near_entity(e, r, |entity| entity.to_ship())
            .into_iter()
            .filter_map(|(_, _, id)| { ships.get(&id).map_or(None, |ship| {
                if ship.owner != self.owner { Some(ship) } else { None }
            })}).collect()
    }

    pub fn near_allies<'a, T: ToEntity>(&self, e: &T, r: f64, ships: &'a Ships)
        -> Vec<&'a Ship> {
        self.near_entity(e, r, |entity| entity.to_ship())
            .into_iter()
            .filter_map(|(_, _, id)| { ships.get(&id).map_or(None, |ship| {
                if ship.owner == self.owner { Some(ship) } else { None }
            })}).collect()
    }

    pub fn near_planets<'a, T: ToEntity>(&self, e: &T, r: f64, planets: &'a Planets)
        -> Vec<&'a Planet> {
        self.near_entity(e, r, |entity| entity.to_planet())
            .into_iter()
            .filter_map(|(_, _, id)| {
                planets.get(&id).map_or(None, |planet| { Some(planet) })
            }).collect()
    }

    pub fn collides_at(&self, ship: &Ship, (x, y): Point) -> bool {
        self.near(&Entity::Ship(x, y, ship.rad, ship.id), SHIP_RADIUS).len() > 0
    }

    pub fn collides_toward<T: ToEntity>(&self, e: &T, (x2, y2): Point) -> bool {
        let entity = e.to_entity();
        let (x1, y1) = entity.pos();
        self.near(e, (y2 -y1).hypot(x2 - x1))
            .into_iter()
            .any(|&other| other.intersects_line((x1, y1), (x2, y2)))
    }

    fn wiggle(n: i32, m: i32, a: i32, t: i32, target: i32)
        -> (i32, i32, i32, i32) {
        if n == m {
            let t = 7 - (m/10)*DELTA_THRUST;
            (0, m+10, target, if t > MIN_THRUST {t} else {MIN_THRUST})
        } else {
            match n % 2 {
                0 => (n+1, m, a - n*DELTA_THETA, t),
                1 => (n+1, m, a + n*DELTA_THETA, t),
                _ => unreachable!()
            }
        }
    }

    pub fn closest_free(&self, ship: &Ship, (x, y): Point, thrust: i32)
        -> (f64, f64, i32, i32) {
        let target = f64::atan2(y - ship.y, x - ship.x).to_degrees().round() as i32;
        let (mut a, mut t, mut n, mut m) = (target, thrust, 0, 0);
        loop {
            let r = (a as f64).to_radians();
            let (xf, yf) = (ship.x + (t as f64)*r.cos(),
                            ship.y + (t as f64)*r.sin());

            let at = self.collides_at(&ship, (xf, yf));
            let toward = self.collides_toward(&ship, (xf, yf));

            if (at || toward) && m < CORRECTIONS {
                let (n2, m2, a2, t2) = Self::wiggle(n, m, a, t, target);
                n = n2; m = m2; a = a2; t = t2;
            } else if at || toward {
                return (ship.x, ship.y, 0, 0)
            } else {
                return (xf, yf, t, a)
            }
        }
    }
}

mod tests {
    #![cfg(test)]
    use hlt::collision::*;

    #[test]
    fn test_insert() {
        let mut grid = Grid::new();
        grid.insert(&Entity::Obstacle((12.0, 12.0, 12.0)));
    }

    #[test]
    fn test_circle() {
        let circle = Entity::Obstacle((0.0, 0.0, 5.0));
        assert!(circle.intersects_line((-5.0, 5.0), (5.0, 5.0)));
        assert!(circle.intersects_line((-5.0, 5.0), (0.0, 5.0)));
        assert!(circle.intersects_line((0.0, 5.0), (5.0, 5.0)));
    }

    #[test]
    fn test_offset_circle() {
        let circle = Entity::Obstacle((5.0, 5.0, 5.0));
        assert!(circle.intersects_line((-10.0, 1.0), (10.0, 10.0)));
        assert!(circle.intersects_line((0.0, 0.0), (0.0, 10.0)));
        assert!(circle.intersects_line((0.0, 0.0), (10.0, 0.0)));
    }

    #[test]
    fn test_no_collision() {
        let circle = Entity::Obstacle((5.0, 5.0, 0.0));
        assert!(!circle.intersects_line((0.0, 0.0), (1.0, 1.0)));
    }

    #[test]
    fn test_ship_collision() {
        let ship = Entity::Ship((5.0, 5.0, SHIP_RADIUS, 0));
        assert_eq!(true, ship.intersects_line((6.0, 0.0), (6.0, 10.0)));
        assert_eq!(false, ship.intersects_line((6.01, 0.0), (6.01, 10.0)));
    }

    #[test]
    fn test_ship_diagonal() {
        let ship = Entity::Ship((0.0, 0.0, SHIP_RADIUS, 0));
        assert_eq!(true, ship.intersects_line((1.0, 0.0), (0.0, 1.0)));
        assert_eq!(true, ship.intersects_line((0.0, 1.0), (-1.0, 0.0)));
        assert_eq!(true, ship.intersects_line((-1.0, 0.0), (0.0, 1.0)));
        assert_eq!(true, ship.intersects_line((0.0, -1.0), (1.0, 0.0)));
    }

    #[test]
    fn test_planet_diagonal() {
        let planet = Entity::Planet((0.0, 0.0, SHIP_RADIUS, 0));
        assert_eq!(true, planet.intersects_line((1.0, 0.0), (0.0, 1.0)));
        assert_eq!(true, planet.intersects_line((0.0, 1.0), (-1.0, 0.0)));
        assert_eq!(true, planet.intersects_line((-1.0, 0.0), (0.0, 1.0)));
        assert_eq!(true, planet.intersects_line((0.0, -1.0), (1.0, 0.0)));
    }
}
