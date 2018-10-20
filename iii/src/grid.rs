use std::cmp;
use std::usize;
use std::collections::BinaryHeap;

use fixedbitset::FixedBitSet;
use fnv::{FnvHashSet, FnvHashMap};

use constants::{MAX_CELL_PRODUCTION};
use command::Command;
use data::{Dropoff, Ship, Shipyard};

pub const DIRS: [Dir; 4] = [Dir::N, Dir::S, Dir::E, Dir::W];

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    N, S, E, W
}

impl Dir {
    pub fn reflect(self) -> Self {
        match self {
        | Dir::N => Dir::S,
        | Dir::S => Dir::N,
        | Dir::E => Dir::W,
        | Dir::W => Dir::E,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(pub usize, pub usize);

#[derive(Clone, Debug)]
pub struct Grid<'round> {
    width: usize,
    height: usize,
    halite: &'round [usize],
    occupied: FixedBitSet,
    base: Pos,
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
        for ship in ships {
            occupied.put(ship.y * width + ship.x);
        }
        for drop in drops {
            if drop.owner == id {
                dropoff.insert(Pos(drop.x, drop.y));
            }
        }
        let yard = yards[id];
        let base = Pos(yard.x, yard.y);
        Grid { width, height, halite, occupied, base, dropoff }
    }

    #[inline(always)]
    fn index(&self, x: usize, y: usize) -> usize {
        self.width * y + x
    }

    pub fn spawn(&mut self) {
        let index = self.index(self.base.0, self.base.1);
        self.occupied.put(index);
    }

    pub fn can_spawn(&self) -> bool {
        !self.occupied[self.index(self.base.0, self.base.1)]
    }

    pub fn dx(&self, x1: usize, x2: usize) -> usize {
        let min_x = usize::min(x1, x2);
        let max_x = usize::max(x1, x2);
        if (max_x - min_x) > (self.width / 2) {
            min_x + self.width - max_x
        } else {
            max_x - min_x
        }
    }

    pub fn dy(&self, y1: usize, y2: usize) -> usize {
        let min_y = usize::min(y1, y2);
        let max_y = usize::max(y1, y2);
        if (max_y - min_y) > (self.height / 2) {
            min_y + self.height - max_y
        } else {
            max_y - min_y
        }
    }

    pub fn dist(&self, a: Pos, b: Pos) -> usize {
        self.dx(a.0, b.0) + self.dy(a.1, b.1)
    }

    pub fn step(&self, p: Pos, d: Dir) -> Pos {
        match d {
        | Dir::N => Pos(p.0, (p.1 + self.height - 1) % self.height),
        | Dir::S => Pos(p.0, (p.1 + 1) % self.height),
        | Dir::E => Pos((p.0 + 1) % self.width, p.1),
        | Dir::W => Pos((p.0 + self.width - 1) % self.width, p.1),
        }
    }

    pub fn distance_from_yard(&self, ship: &Ship) -> usize {
        self.dist(Pos(ship.x, ship.y), self.base)
    }

    pub fn distances_from(&self, pos: Pos, min: usize, buffer: &mut Vec<usize>) {
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width {
                let col = row + x;
                let halite = self.halite[col];
                if halite > min && (Pos(x, y) == pos || !self.occupied[col]) {
                    buffer.push(
                        ((MAX_CELL_PRODUCTION - usize::min(halite, MAX_CELL_PRODUCTION)) / 500) +
                        self.dist(pos, Pos(x, y)) +
                        self.dist(Pos(x, y), self.base)
                    );
                } else {
                    buffer.push(usize::max_value());
                }
            }
        }
    }

    pub fn navigate(&mut self, ship: &Ship, end: Pos) -> Command {

        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        struct Node(Pos, usize);

        impl PartialOrd for Node {
            fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
                Some(self.cmp(rhs))
            }
        }

        impl Ord for Node {
            fn cmp(&self, rhs: &Self) -> cmp::Ordering {
                rhs.1.cmp(&self.1).then_with(|| self.0.cmp(&rhs.0))
            }
        }

        let start = Pos(ship.x, ship.y);
        let start_index = self.index(ship.x, ship.y);

        if self.halite[start_index] / 10 > ship.halite || start == end {
            return Command::Stay(ship.id)
        }

        let mut queue = BinaryHeap::default();
        let mut trace = FnvHashMap::default();
        let mut costs = FnvHashMap::default();
        let mut seen = FnvHashSet::default();

        costs.insert(start, 0);
        queue.push(Node(start, 0));

        while let Some(Node(node, _)) = queue.pop() {

            if node == end {
                let mut step = end;
                let mut dir = Dir::N;
                while let Some((prev, prev_dir)) = trace.get(&step) {
                    dir = *prev_dir;
                    if *prev == start { break }
                    step = *prev;
                }

                let step_index = self.index(step.0, step.1);
                if self.occupied[step_index] {
                    return Command::Stay(ship.id)
                } else {
                    self.occupied.set(start_index, false);
                    self.occupied.put(step_index);
                    info!("Ship {} moving from {:?} to {:?} in dir {:?} en route to {:?}",
                        ship.id,
                        start,
                        step,
                        dir,
                        end,
                    );
                    return Command::Move(ship.id, dir);
                }
            }

            seen.insert(node);

            for dir in &DIRS {

                let next = self.step(node, *dir);
                let next_index = self.index(next.0, next.1);

                if self.occupied[next_index] && next != end {
                    continue
                }

                let node_index = self.index(node.0, node.1);
                let next_cost = costs[&node] + self.halite[node_index] / 10;

                if let Some(prev_cost) = costs.get(&next) {
                    if *prev_cost <= next_cost {
                        continue
                    }
                }

                let heuristic = self.dist(next, end);
                trace.insert(next, (node, *dir));
                costs.insert(next, next_cost);
                queue.push(Node(next, next_cost + heuristic));
            }
        }

        return Command::Stay(ship.id)
    }
}
