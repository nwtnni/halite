use std::str::FromStr;
use std::collections::hash_map::HashMap;
use fnv::{FnvBuildHasher, FnvHashMap};
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
        let rad = SHIP_RADIUS;
        let status = Status::take(stream);
        let planet = if let Status::Docked = status {
            Some(usize::take(stream))
        } else {
            stream.pop();
            None
        };
        let progress = i32::take(stream);
        let _deprecated = stream.pop();
        Ship {id, x, y, hp, rad, status, planet, progress, owner: 0}
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

pub fn take(stream: &mut Vec<&str>) -> (
        Vec<Player>, HashMap<ID, Planet, FnvBuildHasher>,
        HashMap<ID, Ship, FnvBuildHasher>, Grid
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
                grid.insert(&ship);
                ships.insert(ship.id, ship);
            }
            players.push(Player {id, ships: player_ships});
        }

        for _ in 0..(i32::take(stream)) {
            let planet = Planet::take(stream);
            grid.insert(&planet);
            planets.insert(planet.id, planet);
        }
        (players, planets, ships, grid)
}

mod tests {
    #![cfg(test)]
    use hlt::state::*;
    use hlt::parse::{FromStream, take};

    #[test]
    fn test_status_take() {
        let string = String::from(" 0");
        let mut stream = string.split_whitespace().collect::<Vec<_>>();
        assert_eq!(Status::take(&mut stream), Status::Undocked);
        assert_eq!(stream.len(), 0);
    }

    #[test]
    fn test_ship_take() {
        let string = String::from("3 188.0907 107.6403 255 0.0000 0.0000 1 7 5 0");
        let mut stream = string.split_whitespace().rev().collect::<Vec<_>>();
        Ship::take(&mut stream);
    }

    #[test]
    fn test_planet_take() {
        let string = String::from("7 184.5627 114.5568 1373 5.3870 2 0 775 1 1 2 5 3");
        let mut stream = string.split_whitespace().rev().collect::<Vec<_>>();
        Planet::take(&mut stream);
    }

    #[test]
    fn test_map_take() {
        let string = String::from("2 0 3 \
        0  99.1403  98.0343 255 0.0000 0.0000 0 0 0 0 \
        1 100.5614  86.3555 255 0.0000 0.0000 0 0 0 0 \
        2  99.6768 100.5296 255 0.0000 0.0000 0 0 0 0 \
        1 3 \
        3 188.0907 107.6403 255 0.0000 0.0000 1 7 5 0 \
        4 185.1240 106.4292 255 0.0000 0.0000 0 0 0 0 \
        5 189.9773 109.3196 255 0.0000 0.0000 1 7 4 0 \
        9 \
        0  142.2093  98.2093 1713 6.7190 3 0  967 0 0 0 \
        1  121.7907  98.2093 1713 6.7190 3 0  967 0 0 0 \
        2  121.7907  77.7907 1713 6.7190 3 0  967 0 0 0 \
        3  142.2093  77.7907 1713 6.7190 3 0  967 0 0 0 \
        4   31.9692 101.9547 1373 5.3870 2 0  775 0 0 0 \
        5   79.4373  61.4432 1373 5.3870 2 0  775 0 0 0 \
        6  232.0308  74.0453 1373 5.3870 2 0  775 0 0 0 \
        7  184.5627 114.5568 1373 5.3870 2 0  775 1 1 2 5 3 \
        8  171.7497  18.2646 1986 7.7885 3 0 1121 0 0 0");
        let mut stream = string.split_whitespace().rev().collect::<Vec<_>>();
        take(&mut stream);
    }
}
