use std::cmp;
use std::iter;
use std::mem;
use std::usize;
use std::collections::BinaryHeap;

use fixedbitset::FixedBitSet;
use fnv::{FnvHashSet, FnvHashMap};

use constants::HALITE_TIME_RATIO;
use command::Command;
use data::*;

pub const DIRS: [Dir; 5] = [Dir::N, Dir::S, Dir::E, Dir::W, Dir::O];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    N, S, E, W, O
}

impl Dir {
    pub fn reflect(self) -> Self {
        match self {
        | Dir::N => Dir::S,
        | Dir::S => Dir::N,
        | Dir::E => Dir::W,
        | Dir::W => Dir::E,
        | Dir::O => Dir::O,
        }
    }
}

#[derive(Debug)]
pub struct Grid<'round> {
    width: Dist,
    height: Dist,
    round: Time,
    halite: &'round [Halite],
    reserved: &'round mut FnvHashSet<(Pos, Time)>,
    routes: &'round mut FnvHashMap<ID, Vec<Pos>>,
    allies: FixedBitSet,
    enemies: FixedBitSet,
    drops: FixedBitSet,
    spawn: Pos,
}

impl<'round> Grid<'round> {
    pub fn new(
        id: PID,
        width: Dist,
        height: Dist,
        round: Time,
        halite: &'round [Halite],
        ships: &[Ship],
        dropoffs: &[Dropoff],
        yards: &[Shipyard],
    ) -> Self {
        unimplemented!()
    }

    #[inline(always)]
    fn idx(&self, Pos(x, y): Pos) -> usize {
        self.width as usize * y as usize + x as usize
    }

    #[inline(always)]
    fn inv_idx(&self, idx: usize) -> Pos {
        let x = (idx % (self.width as usize)) as Dist;
        let y = (idx / (self.width as usize)) as Dist;
        Pos(x, y)
    }

    pub fn dx(&self, x1: Dist, x2: Dist) -> Dist {
        let dx = Dist::abs(x2 - x1);
        if dx < self.width / 2 { dx } else { self.width - dx }
    }

    pub fn dy(&self, y1: Dist, y2: Dist) -> Dist {
        let dy = Dist::abs(y2 - y1);
        if dy < self.height / 2 { dy } else { self.height - dy }
    }

    pub fn dist(&self, Pos(x1, y1): Pos, Pos(x2, y2): Pos) -> Dist {
        self.dx(x1, x2) + self.dy(y1, y2)
    }

    pub fn dist_from_yard(&self, ship: &Ship) -> Dist {
        self.dist(Pos(ship.x, ship.y), self.spawn)
    }

    pub fn step(&self, Pos(x, y): Pos, d: Dir) -> Pos {
        match d {
        | Dir::N => Pos(x, (y + self.height - 1) % self.height),
        | Dir::S => Pos(x, (y + 1) % self.height),
        | Dir::E => Pos((x + 1) % self.width, y),
        | Dir::W => Pos((x + self.width - 1) % self.width, y),
        | Dir::O => Pos(x, y),
        }
    }

    pub fn inv_step(&self, Pos(x1, y1): Pos, Pos(x2, y2): Pos) -> Dir {
        match (x2 - x1, y2 - y1) {
        | (0, dy) if dy == -1 || dy ==  self.height - 1 => Dir::N,
        | (0, dy) if dy == 1  || dy == -self.height + 1 => Dir::S,
        | (dx, 0) if dx == -1 || dx ==  self.width - 1  => Dir::W,
        | (dx, 0) if dx == 1  || dx == -self.width + 1  => Dir::E,
        | _ => unreachable!(),
        }
    }

    fn around(&self, Pos(x, y): Pos, radius: Dist) -> impl Iterator<Item = Pos> {
        let (w, h) = (self.width, self.height);
        (0..radius).flat_map(move |dy| {
            (0..radius)
                .filter(move |dx| !(*dx == 0 && dy == 0))
                .flat_map(move |dx| {
                    iter::once(Pos((x + w - dx) % w, (y + h - dy) % h))
                        .chain(iter::once(Pos((x + dx) % w,     (y + h - dy) % h)))
                        .chain(iter::once(Pos((x + w - dx) % w, (y + dy) % h)))
                        .chain(iter::once(Pos((x + dx) % w,     (y + dy) % h)))
            })
        })
    }

    pub fn allies_around(&self, pos: Pos, radius: Dist) -> usize {
        self.around(pos, radius)
            .filter(|pos| self.allies[self.idx(*pos)])
            .count()
    }

    pub fn enemies_around(&self, pos: Pos, radius: Dist) -> usize {
        self.around(pos, radius)
            .filter(|pos| self.enemies[self.idx(*pos)])
            .count()
    }

    pub fn fill_cost<F>(&self, costs: &mut Vec<Halite>, f: F)
        where F: Fn(&Self, Pos, Halite) -> Halite,
    {
        for y in 0..self.height {
            for x in 0..self.width {
                let pos = Pos(x, y);
                let index = self.idx(pos);
                let halite = self.halite[index];
                costs.push(f(self, pos, halite));
            }
        }
    }
}
