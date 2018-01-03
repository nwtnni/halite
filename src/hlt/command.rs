use hlt::state::*;
use hlt::collision::*;
use hlt::constants::*;

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

pub fn thrust(distance: f32) -> i32 {
    let d = distance.floor() as i32;
    if d > SHIP_SPEED { SHIP_SPEED }
    else { d }
}

fn offset(offset: f32, (x, y): Point, angle: f32) -> Point {
    (x - (offset*angle.cos()), y - (offset*angle.sin()))
}

pub fn navigate<T: ToEntity>(grid: &mut Grid, ship: &Ship, target: &T) -> Command {
    let target = target.to_entity();
    let (xt, yt) = target.pos();
    let angle = f32::atan2(yt - ship.y, xt - ship.x);
    let (xf, yf) = match target {
        Entity::Ship(_, _, _, _) => {
            offset(WEAPON_RADIUS, (xt, yt), angle)
        },
        Entity::Planet(_, _, _, _) => {
            offset(DOCK_RADIUS + target.rad() - 0.50, (xt, yt), angle)
        },
        Entity::Obstacle(_, _, _, _) => (xt, yt),
    };
    let thrust = thrust((yf - ship.y).hypot(xf - ship.x));
    let (x, y, thrust, angle) = grid.closest_free(ship, (xf, yf), thrust);
    let mut smoke = 0.0;
    while (smoke as i32) < thrust {
        grid.create_obstacle(
            ship.x + smoke*(angle as f32).to_radians().cos(),
            ship.y + smoke*(angle as f32).to_radians().sin(),
            SHIP_RADIUS);
        smoke += 0.5;
    }
    grid.remove(&ship);
    grid.create_ship(x, y, ship.id);
    Command::Thrust(ship.id, thrust, (angle + 360) % 360)
}
