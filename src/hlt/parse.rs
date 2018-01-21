use std::str::FromStr;
use fnv::FnvHashMap;
use hlt::state::*;
use hlt::collision::Grid;
use hlt::constants::SHIP_RADIUS;

pub trait FromStream {
    fn take(stream: &mut Vec<&str>) -> Self;
}

impl FromStream for i32 {
    fn take(stream: &mut Vec<&str>) -> Self {
        let n = stream.pop().expect("Unexpected EOF");
        i32::from_str(n).expect("Expected i32")
    }
}

impl FromStream for f64 {
    fn take(stream: &mut Vec<&str>) -> Self {
        let n = stream.pop().expect("Unexpected EOF");
        f64::from_str(n).expect("Expected f64")
    }
}

impl FromStream for usize {
    fn take(stream: &mut Vec<&str>) -> Self {
        let n = stream.pop().expect("Unexpected EOF");
        usize::from_str(n).expect("Expected usize")
    }
}

impl FromStream for Status {
    fn take(stream: &mut Vec<&str>) -> Self {
        match i32::take(stream) {
            0 => Status::Undocked,
            1 => Status::Docking,
            2 => Status::Docked,
            3 => Status::Undocking,
            _ => panic!("Expected docking status"),
        }
    }
}

impl FromStream for Ship {
    fn take(stream: &mut Vec<&str>) -> Self {
        let id = usize::take(stream);
        let x = f64::take(stream);
        let y = f64::take(stream);
        let hp = i32::take(stream);
        let _deprecated = stream.pop();
        let _deprecated = stream.pop();
        let status = Status::take(stream);
        let planet = if let Status::Docked = status {
            Some(usize::take(stream))
        } else {
            stream.pop();
            None
        };
        let progress = i32::take(stream);
        let _deprecated = stream.pop();
        Ship {id, x, y, hp, status, planet, progress, owner: 0}
    }
}

impl FromStream for Planet {
    fn take(stream: &mut Vec<&str>) -> Self {
        let id = usize::take(stream);
        let x = f64::take(stream);
        let y = f64::take(stream);
        let hp = i32::take(stream);
        let rad = f64::take(stream);
        let spots = usize::take(stream);
        let spawn = i32::take(stream);
        let _deprecated = stream.pop();
        let owned = i32::take(stream);
        let owner = if owned == 1 {
            Some(usize::take(stream))
        } else {
            stream.pop();
            None
        };
        let mut ships = Vec::new();
        for _ in 0..(i32::take(stream)) {
            ships.push(usize::take(stream));
        }
        Planet {id, x, y, hp, rad, spots, spawn, owner, ships}
    }
}

pub fn take(stream: &mut Vec<&str>) -> (
        Vec<Player>, FnvHashMap<ID, Planet>,
        FnvHashMap<ID, Ship>, Grid
    ) {
        let mut players = Vec::new();
        let mut planets = FnvHashMap::default();
        let mut ships = FnvHashMap::default();
        let mut grid = Grid::new();

        for _ in 0..(i32::take(stream)) {
            let id = usize::take(stream);
            let mut player_ships = Vec::new();

            for _ in 0..(i32::take(stream)) {
                let mut ship = Ship::take(stream);
                ship.owner = id;
                player_ships.push(ship.id);
                grid.insert(&&ship);
                ships.insert(ship.id, ship);
            }
            players.push(Player {id, ships: player_ships});
        }

        for _ in 0..(i32::take(stream)) {
            let planet = Planet::take(stream);
            grid.insert(&&planet);
            planets.insert(planet.id, planet);
        }
        (players, planets, ships, grid)
}
