use std::cmp;
use std::iter;
use std::mem;
use std::usize;
use std::collections::BinaryHeap;

use fixedbitset::FixedBitSet;
use fnv::{FnvHashSet, FnvHashMap};

use constants::HALITE_TIME_RATIO;
use command::Command;
use data::{Dropoff, Ship, Shipyard};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(pub usize, pub usize);

#[derive(Debug)]
pub struct Grid<'round> {
    width: usize,
    height: usize,
    round: usize,
    halite: &'round [usize],
    allies: FixedBitSet,
    enemies: FixedBitSet,
    stuck: FixedBitSet,
    base: Pos,
    drops: FnvHashSet<Pos>,
    planned: Vec<(usize, Dir, Pos, bool)>,
}

impl<'round> Grid<'round> {
    pub fn new(
        id: usize,
        width: usize,
        height: usize,
        round: usize,
        halite: &'round [usize],
        ships: &[Ship],
        dropoffs: &[Dropoff],
        yards: &[Shipyard],
    ) -> Self {
        let mut allies = FixedBitSet::with_capacity(width * height);
        let mut enemies = FixedBitSet::with_capacity(width * height);
        let mut stuck = FixedBitSet::with_capacity(width * height);
        let mut drops = FnvHashSet::default();

        for ship in ships {
            if ship.owner == id {
                let ship_index = ship.y * width + ship.x;
                if ship.halite < halite[ship_index] / 10 {
                    stuck.put(ship_index);
                }
                allies.put(ship_index);
            } else {
                enemies.put(ship.y * width + ship.x);
            }
        }

        for drop in dropoffs {
            if drop.owner == id {
                drops.insert(Pos(drop.x, drop.y));
            }
        }

        let yard = yards[id];
        let base = Pos(yard.x, yard.y);
        let planned = Vec::new();

        Grid {
            width,
            height,
            round,
            halite,
            allies,
            enemies,
            stuck,
            base,
            drops,
            planned,
        }
    }

    #[inline(always)]
    fn index(&self, pos: Pos) -> usize {
        self.width * pos.1 + pos.0
    }

    pub fn is_stuck(&self, pos: Pos) -> bool {
        self.stuck.contains(self.index(pos))
    }

    pub fn distance_from_yard(&self, ship: &Ship) -> usize {
        self.dist(Pos(ship.x, ship.y), self.base)
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
        | Dir::O => p,
        }
    }

    fn around(&self, pos: Pos, radius: usize) -> impl Iterator<Item = Pos> {
        let (w, h) = (self.width, self.height);
        (0..radius).flat_map(move |y| {
        (0..radius).flat_map(move |x| {
            iter::once(Pos((pos.0 + w - x) % w, (pos.1 + h - y) % h))
                .chain(iter::once(Pos((pos.0 + x) % w,     (pos.1 + h - y) % h)))
                .chain(iter::once(Pos((pos.0 + w - x) % w, (pos.1 + y) % h)))
                .chain(iter::once(Pos((pos.0 + x) % w,     (pos.1 + y) % h)))
        })
        })
    }

    // pub fn allies_around(&self, pos: Pos, radius: usize) -> usize {
    //     self.around(pos, radius)
    //         .filter(|pos| self.allies[self.index(*pos)])
    //         .count()
    // }

    pub fn enemies_around(&self, pos: Pos, radius: usize) -> usize {
        self.around(pos, radius)
            .filter(|pos| self.enemies[self.index(*pos)])
            .count()
    }

//     pub fn halite_around(&self, pos: Pos, radius: usize) -> usize {
//         self.around(pos, radius)
//             .map(|pos| self.halite[self.index(pos)])
//             .sum()
//     }

//     pub fn average_halite(&self) -> usize {
//         self.halite.iter().sum::<usize>() / self.halite.len()
//     }

    pub fn fill_cost<F>(&self, costs: &mut Vec<usize>, f: F)
        where F: Fn(&Self, Pos, usize) -> usize,
    {
        for y in 0..self.height {
            let row = y * self.width;
            for x in 0..self.width {
                let index = row + x;
                let pos = Pos(x, y);
                let halite = self.halite[index];
                costs.push(f(self, pos, halite));
            }
        }
    }

    pub fn plan_route(&mut self, ship: &Ship, end: Pos, crash: bool) {

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
        let start_index = self.index(start);

        if self.halite[start_index] / 10 > ship.halite || start == end {
            self.planned.push((ship.id, Dir::O, start, crash));
            return
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
                let mut dir = Dir::O;
                while let Some((prev, prev_dir)) = trace.get(&step) {
                    dir = *prev_dir;
                    if *prev == start { break }
                    step = *prev;
                }

                let next = self.step(start, dir);
                self.planned.push((ship.id, dir, next, crash));
                return
            }

            seen.insert(node);

            for dir in &DIRS {

                let next = self.step(node, *dir);
                let next_index = self.index(next);

                if seen.contains(&next) || self.stuck[next_index]
                || (self.enemies_around(next, 1) > 0 && next != self.base) {
                    continue
                }

                let node_index = self.index(node);
                let crowd_cost = if self.allies[next_index] {
                    // Don't even think about trying anything fancy
                    if start == self.base { 1000000 } else { 1 }
                } else {
                    0
                };
                let halite_cost = (self.halite[node_index] / 10) / HALITE_TIME_RATIO;
                let time_cost = 1;
                let next_cost = costs[&node] + crowd_cost + halite_cost + time_cost;

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

        warn!("[{}]: unable to path to {:?}", ship.id, end);
        self.planned.push((ship.id, Dir::O, start, crash));
    }

    pub fn resolve_routes(&mut self) -> (bool, Vec<Command>) {

        let mut planned = mem::replace(&mut self.planned, Vec::with_capacity(0));
        let routes = planned.len();
        let mut resolved = Vec::with_capacity(routes);
        let mut change;

        loop {

            change = None;

            'outer: for i in 0..routes {
                let (id_a, dir_a, next_a, crash_a) = planned[i];
                for j in i + 1..routes {
                    let (id_b, dir_b, next_b, crash_b) = planned[j];
                    if next_a == next_b {
                        if next_a == self.base && (crash_a || crash_b) {
                            continue
                        } else if dir_a == Dir::O {
                            change = Some(id_b);
                        } else if dir_b == Dir::O {
                            change = Some(id_a);
                        } else {
                            change = Some(id_b);
                        }
                        break 'outer;
                    }
                }
            }

            if let Some(id) = change {
                for plan in &mut planned {
                    if id == plan.0 {
                        plan.2 = self.step(plan.2, plan.1.reflect());
                        plan.1 = Dir::O;
                        break
                    }
                }
            } else {
                break
            }
        }

        let mut spawnable = true;
        for (id, dir, next, _) in planned {
            if next == self.base { spawnable = false; }
            resolved.push(Command::Move(id, dir));
        }

        (spawnable, resolved)
    }
}
