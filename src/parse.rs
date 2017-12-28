use std::str::FromStr;
use fnv::FnvHashMap;
use state::*;

pub trait FromStream {
    fn take(stream: &mut Vec<&str>) -> Self;
}

impl FromStream for i32 {
    fn take(stream: &mut Vec<&str>) -> Self {
        let n = stream.pop().expect("Unexpected EOF");
        i32::from_str(n).expect("Expected i32")
    }
}

impl FromStream for f32 {
    fn take(stream: &mut Vec<&str>) -> Self {
        let n = stream.pop().expect("Unexpected EOF");
        f32::from_str(n).expect("Expected f32")
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
        let x = f32::take(stream);
        let y = f32::take(stream);
        let hp = i32::take(stream);
        let _deprecated = stream.pop();
        let _deprecated = stream.pop();
        let rad = f32::take(stream);
        let status = Status::take(stream);
        let planet = if let Status::Docked = status {
            Some(usize::take(stream))
        } else {
            stream.pop();
            None
        };
        let progress = i32::take(stream);
        let _deprecated = stream.pop();
        Ship {id, x, y, hp, rad, status, planet, progress}
    }
}

impl FromStream for Planet {
    fn take(stream: &mut Vec<&str>) -> Self {
        let id = usize::take(stream);
        let x = f32::take(stream);
        let y = f32::take(stream);
        let hp = i32::take(stream);
        let rad = f32::take(stream);
        let spots = i32::take(stream);
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

impl FromStream for Map {
    fn take(stream: &mut Vec<&str>) -> Self {
        let mut players = Vec::new();
        let mut planets = Vec::new();
        let mut ships = FnvHashMap::default();

        for _ in 0..(i32::take(stream)) {
            let id = usize::take(stream);
            let mut player_ships = Vec::new();

            for _ in 0..(i32::take(stream)) {
                let ship = Ship::take(stream);
                player_ships.push(ship.id);
                ships.insert(ship.id, ship);
            }
            players.push(Player {id, ships: player_ships});
        }

        for _ in 0..(i32::take(stream)) {
            planets.push(Planet::take(stream));
        }
        Map {players, planets, ships}
    }
}
