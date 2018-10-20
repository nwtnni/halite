use std::usize;

use fixedbitset::FixedBitSet;
use fnv::FnvHashSet;

use command::Command;
use data::{Dropoff, Ship, Shipyard};

pub const DIRS: [Dir; 4] = [Dir::N, Dir::S, Dir::E, Dir::W];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    N, S, E, W
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(usize, usize);

#[derive(Clone, Debug)]
pub struct Grid<'round> {
    width: usize,
    height: usize,
    halite: &'round [usize],
    occupied: FixedBitSet,
    dropoff: FnvHashSet<Pos>,
}

impl<'round> Grid<'round> {
    pub fn new(
        id: usize,
        width: usize,
        height: usize,
        halite: &'round [usize],
        ships: &[Ship],
        drops: &[Dropoff],
        yards: &[Shipyard],
    ) -> Self {
        let mut occupied = FixedBitSet::with_capacity(width * height);
        let mut dropoff = FnvHashSet::default();
        for ship in ships { occupied.put(ship.y * width + ship.x); }
        for drop in drops {
            if drop.owner == id {
                dropoff.insert(Pos(drop.x, drop.y));
            }
        }
        let yard = yards[id];
        dropoff.insert(Pos(yard.x, yard.y));
        Grid { width, height, halite, occupied, dropoff }
    }

    pub fn sqdist(&self, a: Pos, b: Pos) -> usize {
        let min_x = usize::min(a.0, b.0);
        let max_x = usize::max(a.0, b.0);
        let min_y = usize::min(a.1, b.1);
        let max_y = usize::max(a.1, b.1);
        let dx = if (max_x - min_x) > (self.width / 2) {
            min_x + self.width - max_x
        } else {
            max_x - min_x
        };
        let dy = if (max_y - min_y) > (self.height / 2) {
            min_y + self.height - max_y
        } else {
            max_y - min_y
        };
        dx * dx + dy * dy
    }

    pub fn step(&self, p: Pos, d: Dir) -> Pos {
        match d {
        | Dir::N => Pos(p.0, (p.1 + 1) % self.height),
        | Dir::S => Pos(p.0, (p.1 + self.height - 1) % self.height),
        | Dir::E => Pos((p.1 + 1) % self.width, p.1),
        | Dir::W => Pos((p.1 + self.width - 1) % self.width, p.1),
        }
    }

    pub fn distances_from(&self, pos: Pos, min: usize, buffer: &mut Vec<usize>) {
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width {
                let col = row + x;
                if self.halite[col] >= min {
                    buffer.push(self.sqdist(pos, Pos(x, y)));
                } else {
                    buffer.push(usize::max_value());
                }
            }
        }
    }

    pub fn navigate(&mut self, ship: Ship, dest: Pos) -> Command {
        let pos = Pos(ship.x, ship.y);
        if pos == dest {
            return Command::Stay(ship.id)
        }
        let closest = DIRS.iter()
            .map(|dir| (dir, self.step(pos, *dir)))
            .filter(|(_, pos)| !self.occupied.contains(pos.1 * self.width + pos.0))
            .min_by_key(|(_, pos)| self.sqdist(*pos, dest));
        if let Some((dir, next)) = closest {
            self.occupied.set(pos.1 * self.width + pos.0, false);
            self.occupied.put(next.1 * self.width + next.0);
            Command::Move(ship.id, *dir)
        } else {
            Command::Stay(ship.id)
        }
    }
}
