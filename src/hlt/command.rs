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

pub fn thrust(distance: f32) -> f32 {
    if distance > SHIP_SPEED { SHIP_SPEED }
    else { distance.floor() }
}

fn offset(offset: f32, (x, y): Point, angle: f32) -> Point {
    let r = angle.to_radians();
    (x - (offset * r.cos()), y - (offset * r.sin()))
}

pub fn navigate<T: ToEntity>(grid: &mut Grid, ship: &Ship, target: &T) -> Command {
    let target = target.to_entity();
    let (xt, yt) = target.pos();
    let angle = f32::atan2(yt - ship.y, xt - ship.x).to_degrees().round();
    let (xf, yf) = match target {
        Entity::Ship(_) => offset(WEAPON_RADIUS - 0.50, (xt, yt), angle),
        Entity::Planet(_) => {
            offset(DOCK_RADIUS + target.rad() - 0.50, (xt, yt), angle)
        },
        Entity::Obstacle(_) | Entity::Beacon(_) => (xt, yt),
    };
    let thrust = thrust((yf - ship.y).hypot(xf - ship.x));
    let (x, y, thrust, angle) = grid.closest_free(ship, (xf, yf), thrust);
    let mut smoke = 0.5;
    while smoke < thrust - 1.0 {
        grid.insert(&Entity::Obstacle((
            x - smoke*angle.to_radians().cos(),
            y - smoke*angle.to_radians().sin(),
            SHIP_RADIUS)));
        smoke += 1.0;
    }
    grid.remove(&ship);
    grid.insert(&Entity::Ship((x, y, SHIP_RADIUS, ship.id)));
    Command::Thrust(ship.id, thrust as i32,
        ((angle.round() + 360.0) % 360.0) as i32)
}
