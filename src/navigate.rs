use state::{Point, Planet, Ship, Status};
use constants::{SHIP_RADIUS, DOCK_RADIUS, SHIP_SPEED, CORRECTIONS, DELTA_THETA};
use collision::{ToEntity, Entity, within};
use game::{Command, Game};

pub trait Commander {
    fn command<T: ToEntity>(&self, ship: &Ship, target: &T) -> Command;

    fn can_dock(ship: &Ship, planet: &Planet) -> bool {
        if within((ship.x, ship.y), ship.rad,
                (planet.x, planet.y), planet.rad, DOCK_RADIUS) {
            planet.spots > (planet.ships.len() as i32)
        } else {
            false  
        }
    }

    fn is_docked(ship: &Ship) -> bool {
        ship.status == Status::Docked
    }

    fn dock(ship: &Ship, planet: &Planet) -> Command {
        Command::Dock(ship.id, planet.id)
    }

    fn undock(ship: &Ship) -> Command {
        Command::Undock(ship.id)
    }
}

fn thrust(distance: f32) -> f32 {
    if distance > SHIP_SPEED { SHIP_SPEED }
    else { distance }
}

fn offset(offset: f32, (x, y): Point, angle: f32) -> Point {
    let angle = angle.to_radians();
    let dx = offset * angle.cos();
    let dy = offset * angle.sin();
    match (x.is_sign_positive(), y.is_sign_positive()) {
        (true, true) => (x - dx, y - dy),
        (true, false) => (x - dx, y + dy),
        (false, true) => (x + dx, y - dy),
        _ => (x + dx, y + dy),
    }
}

impl Commander for Game {
    fn command<T: ToEntity>(&self, ship: &Ship, target: &T) -> Command {
        let target = target.to_entity();
        let (xt, yt) = target.pos();
        let vector = (xt - ship.x, yt - ship.y);
        let mut angle = (f32::atan2(vector.0, vector.1).to_degrees() + 360.0) % 360.0;
        let mut n = CORRECTIONS;

        loop {
            let (xf, yf) = match target {
                Entity::Ship(_) => offset(SHIP_RADIUS*2.0, (xt, yt), angle),
                Entity::Planet(_) => {
                    offset(SHIP_RADIUS + DOCK_RADIUS + target.rad(), (xt, yt), angle)
                },
                Entity::Point(_) => (xt, yt),
            };
            if self.map.grid.collides_toward(ship, (xf, yf)) && n > 0 {
                angle = (angle + DELTA_THETA) % 360.0;
                n -= 1;
            } else {
                let thrust = thrust((yf - ship.y).hypot(xf - ship.x));
                return Command::Thrust(ship.id, thrust as i32, angle as i32)
            }
        }
    }
}
