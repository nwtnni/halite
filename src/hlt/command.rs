use hlt::state::*;
use hlt::collision::*;
use hlt::constants::*;
use hlt::strategy::Plan;

pub enum Command {
    Dock(usize, usize),
    Thrust(usize, i32, i32),
}

#[derive(Default, Debug)]
pub struct Queue {
    commands: String,
}

impl Queue {
    pub fn new() -> Self {
        Queue { commands: String::new() }
    }

    pub fn push(&mut self, command: &Command) {
        use self::Command::*;
        let string = match *command {
            Dock(ship, planet) => format!("d {} {} ", ship, planet),
            Thrust(ship, m, a) => format!("t {} {} {} ", ship, m, a),
        };
        self.commands.push_str(&string);
    }

    pub fn flush(&mut self) {
        println!("{}", self.commands);
        self.commands.clear();
    }
}

pub fn dock(ship: &Ship, planet: &Planet) -> Command {
    Command::Dock(ship.id, planet.id)
}

pub fn thrust(distance: f64) -> i32 {
    let d = distance.floor() as i32;
    if d > SHIP_SPEED { SHIP_SPEED }
    else { d }
}

fn offset((x1, y1): Point, (x2, y2): Point, offset: f64, theta: f64) -> Point {
    let angle = f64::atan2(y2 - y1, x2 - x1).to_degrees().round() + theta;
    (x2 - (offset*angle.to_radians().cos()), y2 - (offset*angle.to_radians().sin()))
}

fn navigate(grid: &mut Grid, ship: &Ship, (x, y): Point) -> Command {
    let thrust = thrust((y - ship.y).hypot(x - ship.x));
    let (x, y, thrust, angle) = grid.closest_free(ship, (x, y), thrust);
    grid.update(&ship, (x, y));
    Command::Thrust(ship.id, thrust, (angle + 360) % 360)
}

pub fn navigate_to_enemy(grid: &mut Grid, s: &Ship, e: &Ship) -> Command {
    let (x, y) = offset((s.x, s.y), (e.x, e.y), WEAPON_RADIUS, 0.0);
    navigate(grid, s, (x, y))
}

// Assumes sorted by distance to enemy
pub fn navigate_clump_to_enemy(grid: &mut Grid, mut s: &[Ship], e: &Ship) 
    -> Vec<Command> {
    let far = &s[s.len() - 1].clone();
    let (dx, dy) = (e.x - far.x, e.y - far.y);
    let thrust = f64::min(7.0, dy.hypot(dx));
    let angle = f64::atan2(dy, dx);
    let end = (far.x + thrust*angle.cos(), far.y + thrust*angle.sin());
    let mut queue = Vec::new();
    for ship in s {
        queue.push(navigate_to_point(grid, &ship, end));
    }
    queue 
}

// Assumes sorted in reverse
pub fn navigate_clump_from_enemy(grid: &mut Grid, mut s: &[Ship], e: &Ship)
    -> Vec<Command> {
    let close = &s[s.len() - 1].clone();
    let (dx, dy) = (close.x - e.x, close.y - e.y);
    let angle = f64::atan2(dy, dx);
    let end = (close.x + 7.0*angle.cos(), close.y + 7.0*angle.sin());
    let mut queue = Vec::new();
    for ship in s {
        queue.push(navigate_to_point(grid, &ship, end));
    }
    queue 
}

pub fn navigate_to_distract(grid: &mut Grid, s: &Ship, e: &Vec<&Ship>) -> Command {
    let (x, y) = e.iter().map(|&enemy| (enemy.x, enemy.y))
        .fold((0.0, 0.0), |(x, y), (xe, ye)| (x + xe, y + ye));
    let (x, y) = (x / e.len() as f64, y / e.len() as f64);
    let angle = f64::atan2(s.y - y, s.x - x);
    let (x, y) = (s.x + 7.0*angle.cos(), s.y + 7.0*angle.sin());
    navigate(grid, s, (x, y))
}

pub fn navigate_to_planet(grid: &mut Grid, s: &Ship, p: &Planet) -> Command {
    let (x, y) = offset((s.x, s.y), (p.x, p.y), DOCK_RADIUS + p.rad - 1.0, 0.0);
    navigate(grid, s, (x, y))
}

pub fn navigate_to_point(grid: &mut Grid, s: &Ship, (x, y): Point) -> Command {
    navigate(grid, s, (x, y))
}
