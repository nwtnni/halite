use state::{Point, Planet, Ship, Status};
use constants::{SHIP_RADIUS, DOCK_RADIUS, SHIP_SPEED, CORRECTIONS, DELTA_THETA};
use collision::*;
use game::Command;

pub fn can_dock(ship: &Ship, planet: &Planet) -> bool {
    if within((ship.x, ship.y), ship.rad,
            (planet.x, planet.y), planet.rad, DOCK_RADIUS) {
        planet.spots > (planet.ships.len() as i32)
    } else {
        false
    }
}

pub fn is_docked(ship: &Ship) -> bool {
    ship.status == Status::Docked
}

pub fn dock(ship: &Ship, planet: &Planet) -> Command {
    Command::Dock(ship.id, planet.id)
}

pub fn undock(ship: &Ship) -> Command {
    Command::Undock(ship.id)
}

fn thrust(distance: f32) -> f32 {
    if distance > SHIP_SPEED { SHIP_SPEED }
    else { distance }
}

fn offset(offset: f32, (x, y): Point, angle: f32) -> Point {
    (x - (offset * angle.cos()), y - (offset * angle.sin()))
}

pub fn navigate<T: ToEntity>(grid: &mut HashGrid, ship: &Ship, target: &T) -> Command {
    let target = target.to_entity();
    let (xt, yt) = target.pos();
    let mut angle = f32::atan2(yt - ship.y, xt - ship.x);
    let mut n = CORRECTIONS;

    loop {
        let (xf, yf) = match target {
            Entity::Ship(_) => offset(SHIP_RADIUS, (xt, yt), angle),
            Entity::Planet(_) => {
                offset(DOCK_RADIUS + target.rad(), (xt, yt), angle)
            },
            Entity::Point(_) => (xt, yt),
        };
        let thrust = thrust((yf - ship.y).hypot(xf - ship.x));
        let (xf, yf) = (ship.x + thrust * angle.cos(),
                        ship.y + thrust * angle.sin());
        if grid.collides_toward(ship, (xf, yf)) && n > 0 {
            angle += DELTA_THETA;
            n -= 1;
        } else {
            grid.remove((ship.x, ship.y));
            grid.insert(&Location {x: xf, y: yf, rad: ship.rad});
            angle = (angle.to_degrees() + 360.00) % 360.00;
            return Command::Thrust(ship.id, thrust as i32, angle as i32)
        }
    }
}
