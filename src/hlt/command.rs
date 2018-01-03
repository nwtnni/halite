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
    let mut smoke = 0.0;
    while (smoke as i32) < thrust {
        grid.create_obstacle(
            ship.x + smoke*(angle as f64).to_radians().cos(),
            ship.y + smoke*(angle as f64).to_radians().sin(),
            SHIP_RADIUS);
        smoke += 0.5;
    }
    grid.remove(&ship);
    grid.create_ship(x, y, ship.id);
    Command::Thrust(ship.id, thrust, (angle + 360) % 360)
}

pub fn navigate_to_ally(grid: &mut Grid, s: &Ship, a: &Ship) -> Command {
    let (x, y) = offset((s.x, s.y), (a.x, a.y), 0.0, 90.0);
    navigate(grid, s, (x, y))
}

pub fn navigate_to_enemy(grid: &mut Grid, s: &Ship, e: &Ship) -> Command {
    let (x, y) = offset((s.x, s.y), (e.x, e.y), WEAPON_RADIUS, 0.0);
    navigate(grid, s, (x, y))
}

pub fn navigate_to_planet(grid: &mut Grid, s: &Ship, p: &Planet, plan: &Plan)
    -> Command {
    let o = plan.docking_at(p.id) + p.docked();
    let delta = (1.10 / p.rad).to_degrees().round() as i32;
    let theta = (delta * if o % 2 == 0 { -o/2 } else { o/2 }) as f64;
    let (x, y) = offset((s.x, s.y), (p.x, p.y), DOCK_RADIUS + p.rad - 0.5, theta);
    navigate(grid, s, (x, y))
}

pub fn navigate_to_point(grid: &mut Grid, s: &Ship, (x, y): Point) -> Command {
    navigate(grid, s, (x, y))
}
