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
}

impl Entity {
    pub fn pos(&self) -> (f64, f64) {
        use hlt::collision::Entity::*;
        match *self { Ship(x, y, _, _) | Planet(x, y, _, _) => (x, y), }
    }

    pub fn rad(&self) -> f64 {
        use hlt::collision::Entity::*;
        match *self { Ship(_, _, r, _) | Planet(_, _, r, _) => r }
    }

    fn key(&self) -> String {
        use hlt::collision::Entity::*;
        match *self {
            Ship(_, _, _, id) => "s".to_string() + &id.to_string(),
            Planet(_, _, _, id) => "p".to_string() + &id.to_string(),
        }
    }
}

pub trait ToEntity {
    fn to_entity(&self) -> Entity;
}

impl<'a> ToEntity for &'a Ship {
    fn to_entity(&self) -> Entity {
        Entity::Ship(self.x, self.y, SHIP_RADIUS, self.id)
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
    pub width: f64,
    pub height: f64,
    moved: FnvHashMap<String, (Point, Point)>,
    grid: FnvHashMap<Cell, Vec<Entity>>,
}

impl Grid {
    pub fn new() -> Self {
        Grid {
            owner: 0,
            width: 0.0,
            height: 0.0,
            moved: FnvHashMap::default(),
            grid: FnvHashMap::default(),
        }
    }

    pub fn insert<T: ToEntity>(&mut self, e: &T) {
        let entity = e.to_entity();
        let cells = Self::circle_to_cells(entity.pos(), entity.rad());
        for &cell in &cells {
            self.grid.entry(cell).or_insert(Vec::new()).push(entity);
        }
    }

    pub fn update(&mut self, ship: &Ship, end: Point) {
        let entity = ship.to_entity();
        self.moved.insert(entity.key(), ((ship.x, ship.y), end));
        let cells = Self::line_to_cells((ship.x, ship.y), end);
        for &cell in &cells {
            self.grid.entry(cell).or_insert(Vec::new()).push(entity);
        }
    }

    pub fn moved(&self, ship: &Ship) -> Option<Point> {
        self.moved.get(&ship.to_entity().key()).map(|&(_, end)| end)
    }

    fn to_cell(x: f64, y: f64) -> Cell {
        ((x / GRID_SCALE) as i32, (y / GRID_SCALE) as i32)
    }

    fn circle_to_cells((x, y): Point, r: f64) -> Vec<Cell> {
        let mut cells = Vec::new();
        let (x1, y1) = Self::to_cell(x - SQRT_2*r, y - SQRT_2*r);
        let (mut x1, mut y1) = (x1 as f64 * GRID_SCALE, y1 as f64 * GRID_SCALE);
        let (x2, y2) = Self::to_cell(x + SQRT_2*r, y + SQRT_2*r);
        let (x2, y2) = (x2 as f64 * GRID_SCALE, y2 as f64 * GRID_SCALE);
        while x1 <= x2 {
            let mark = y1;
            while y1 <= y2 {
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

    fn line_to_cells((x1, y1): Point, (x2, y2): Point) -> Vec<Cell> {
        let mut cells = Vec::new();
        cells.extend(Self::circle_to_cells((x1, y1), LINE_RADIUS));
        cells.extend(Self::circle_to_cells((x2, y2), LINE_RADIUS));
        cells.sort_unstable();
        cells.dedup();
        cells
    }

    fn intersects_line((ax1, ay1): Point, (ax2, ay2): Point,
                       (bx1, by1): Point, (bx2, by2): Point, r: f64) -> bool {
        let (x1, y1) = (ax1 - bx1, ay1 - by1);
        let (x2, y2) = (ax2 - bx2, ay2 - by2);
        let (dx, dy) = (x2 - x1, y2 - y1);
        let a = dx*dx + dy*dy;
        let b = 2.0 * (x1*dx + y1*dy);
        let c = x1*x1 + y1*y1 - r*r;
        let d = b*b - 4.0*a*c;
        if d < 0.0 { return false }

        let d = d.sqrt();
        let (t1, t2) = ((-b - d)/(2.0*a), (-b + d)/(2.0*a));
        (t1 >= 0.0 && t1 <= 1.0) || (t2 >= 0.0 && t2 <= 1.0)
    }

    fn intersects_segment(a: Point, b: Point, c: Point, d: Point) -> bool {
        fn ccw((ax, ay): Point, (bx, by): Point, (cx, cy): Point) -> bool {
            (cy - ay)*(bx - ax) > (by - ay)*(cx - ax)
        }
        ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
    }

    pub fn collides_border(&self, ship: &Ship, (x2, y2): Point) -> bool {
        Self::intersects_segment(
            (ship.x, ship.y), (x2, y2), (0.0, 0.0), (self.width, 0.0)
        ) || Self::intersects_segment(
            (ship.x, ship.y), (x2, y2), (0.0, 0.0), (0.0, self.height)
        ) || Self::intersects_segment(
            (ship.x, ship.y), (x2, y2), (0.0, self.height), (self.width, self.height)
        ) || Self::intersects_segment(
            (ship.x, ship.y), (x2, y2), (self.width, 0.0), (self.width, self.height)
        )
    }

    pub fn collides_toward(&self, ship: &Ship, (x2, y2): Point) -> bool {
        let key = ship.to_entity().key();
        let mut close = Self::line_to_cells((ship.x, ship.y), (x2, y2))
            .into_iter()
            .filter_map(|cell| self.grid.get(&cell))
            .flat_map(|bucket| bucket.iter())
            .collect::<Vec<_>>();
        close.sort_unstable_by_key(|&entity| entity.key());
        close.dedup_by_key(|&mut entity| entity.key());
        close.into_iter()
            .filter(|&other| other.key() != key)
            .any(|&other| { match self.moved.get(&other.key()) {
                None => { Self::intersects_line(
                    (ship.x, ship.y), (x2, y2),
                    other.pos(), other.pos(),
                    SHIP_RADIUS + other.rad() + EPSILON
                )},
                Some(&(start, end)) => { Self::intersects_line(
                    (ship.x, ship.y), (x2, y2),
                    start, end, SHIP_RADIUS + other.rad() + EPSILON
                )},
            }})
    }

    fn wiggle(n: i32, m: i32, a: i32, t: i32, max: i32, target: i32)
        -> (i32, i32, i32, i32) {
        if n == m {
            let t = max - (m/DELTA_WIGGLE)*DELTA_THRUST;
            (0, m+DELTA_WIGGLE, target, if t > MIN_THRUST {t} else {MIN_THRUST})
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
            let collides = self.collides_toward(&ship, (xf, yf));
            let border = self.collides_border(&ship, (xf, yf));
            if (collides || border) && m < CORRECTIONS {
                let (n2, m2, a2, t2) = Self::wiggle(n, m, a, t, thrust, target);
                n = n2; m = m2; a = a2; t = t2;
            } else if collides || border {
                return (ship.x, ship.y, 0, 0)
            } else {
                return (xf, yf, t, a)
            }
        }
    }
}
