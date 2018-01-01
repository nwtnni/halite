use hlt::state::*;
use hlt::collision::*;
use hlt::constants::{SHIP_RADIUS, DOCK_RADIUS, SHIP_SPEED, CORRECTIONS, DELTA_THETA};

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
    else { distance }
}

fn offset(offset: f32, (x, y): Point, angle: f32) -> Point {
    (x - (offset * angle.cos()), y - (offset * angle.sin()))
}

pub fn navigate<T: ToEntity>(grid: &mut Grid, ship: &Ship, target: &T) -> Command {
    let target = target.to_entity();
    let (xt, yt) = target.pos();
    let mut angle = f32::atan2(yt - ship.y, xt - ship.x);
    let mut n = CORRECTIONS;

    let (xf, yf) = match target {
        Entity::Ship(_) => offset(SHIP_RADIUS, (xt, yt), angle),
        Entity::Planet(_) => {
            offset(DOCK_RADIUS + target.rad() - 0.50, (xt, yt), angle)
        },
        Entity::Point(_) => (xt, yt),
    };
    let thrust = thrust((yf - ship.y).hypot(xf - ship.x));

    loop {
        let (xf, yf) = (ship.x + thrust * angle.cos(),
                        ship.y + thrust * angle.sin());
        if grid.collides_toward(ship, (xf, yf)) && n > 0 {
            angle += DELTA_THETA;
            n -= 1;
        } else {
            let (xm, ym) = (((xf - ship.x)/2.0 + ship.x), ((yf - ship.y)/2.0 + ship.y));
            grid.remove(ship);
            grid.insert(&Location {x: xm, y: ym, rad: thrust/4.0, id: 0});
            grid.insert(&Location {x: xf, y: yf, rad: ship.rad, id: 0});
            angle = (angle.to_degrees() + 360.00) % 360.00;
            return Command::Thrust(ship.id, thrust as i32, angle as i32)
        }
    }
}
