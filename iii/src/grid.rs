use std::cmp;
use std::iter;
use std::mem;
use std::usize;
use std::collections::{BinaryHeap, VecDeque};

use fixedbitset::FixedBitSet;
use fnv::{FnvHashSet, FnvHashMap};

use constants::HALITE_TIME_RATIO;
use command::Command;
use data::*;

pub const DIRS: [Dir; 4] = [Dir::N, Dir::S, Dir::E, Dir::W];

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
    routes: &'round mut FnvHashMap<ID, VecDeque<Pos>>,
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
        reserved: &'round mut FnvHashSet<(Pos, Time)>,
        routes: &'round mut FnvHashMap<ID, VecDeque<Pos>>,
    ) -> Self {

        let capacity = width as usize * height as usize;
        let mut allies = FixedBitSet::with_capacity(capacity);
        let mut enemies = FixedBitSet::with_capacity(capacity);
        let mut drops = FixedBitSet::with_capacity(capacity);

        for ship in ships {
            let ship_idx = (ship.y as usize)
                * (width as usize)
                + (ship.x as usize);
            if ship.owner == id {
                allies.put(ship_idx);
            } else {
                enemies.put(ship_idx);
            }
        }

        for drop in dropoffs {
            if drop.owner == id {
                let drop_idx = (drop.y as usize)
                    * (width as usize)
                    + (drop.x as usize);
                drops.put(drop_idx);
            }
        }

        let yard = yards[id as usize];
        let spawn = Pos(yard.x, yard.y);
        let spawn_idx = (yard.y as usize) * (width as usize) + (yard.x as usize);
        drops.put(spawn_idx);

        Grid {
            width,
            height,
            round,
            halite,
            allies,
            enemies,
            spawn,
            drops,
            reserved,
            routes,
        }
    }

    #[inline(always)]
    fn idx(&self, Pos(x, y): Pos) -> usize {
        self.width as usize * y as usize + x as usize
    }

    #[inline(always)]
    pub fn inv_idx(&self, idx: usize) -> Pos {
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

//     pub fn dist_from_yard(&self, ship: &Ship) -> Dist {
//         self.dist(Pos(ship.x, ship.y), self.spawn)
//     }

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
        | (0, 0) => Dir::O,
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
        .chain(iter::once(Pos(x, y)))
    }

//     pub fn allies_around(&self, pos: Pos, radius: Dist) -> usize {
//         self.around(pos, radius)
//             .filter(|pos| self.allies[self.idx(*pos)])
//             .count()
//     }

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

    pub fn can_spawn(&self) -> bool {
        !self.reserved.contains(&(self.spawn, self.round + 1))
        && DIRS.iter()
            .map(|dir| self.step(self.spawn, *dir))
            .filter(|pos| !self.reserved.contains(&(*pos, self.round + 1)))
            .count() >= 2
    }

    /// A route is invalid if:
    /// - The ship no longer exists
    /// - The ship's next step is blocked by an enemy
    /// - The ship doesn't have a route
    /// - The ship's current location doesn't match its route
    /// - The ship's new destination no longer matches its route
    pub fn execute_routes(&mut self, ships: &[Ship], commands: &mut Vec<Command>) -> Vec<ID> {

        let alive = ships.iter()
            .map(|ship| ship.id)
            .collect::<FnvHashSet<_>>();

        // Ships that no longer exist
        let dead = self.routes.keys()
            .filter(|id| !alive.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        for id in dead {
            self.clear_route(id);
        }

        info!("{}: Routes before execution", self.round);
        info!("{}: {:?}", self.round, self.routes);
        info!("{}: {:?}", self.round, self.reserved);

        let round = self.round;
        let mut routes = mem::replace(self.routes, FnvHashMap::default());
        let mut invalid = Vec::new();

        for ship in ships {

            if !routes.contains_key(&ship.id) {
                invalid.push(ship.id);
                continue
            }

            let (start, end) = {
                let route = routes.get_mut(&ship.id).unwrap();
                (route.pop_front(), route.front().cloned())
            };

            let ship_pos = Pos(ship.x, ship.y);
            let ship_idx = self.idx(ship_pos);

            match (start, end) {
            | (Some(s), Some(e)) => {

                assert!(s == ship_pos);

                // Invalidate route
                if self.enemies_around(e, 2) > 0 && !self.drops[self.idx(e)] {
                    let route = routes.remove(&ship.id).unwrap();

                    let mut round = round;
                    for pos in route {
                        self.reserved.remove(&(pos, round));
                        round += 1;
                    }

                    invalid.push(ship.id);
                    continue
                }

                // Otherwise good to go
                let dir = self.inv_step(s, e);
                self.reserved.remove(&(s, round));
                info!("{}: ship {} moving to cached dir {:?}", round, ship.id, dir);
                commands.push(Command::Move(ship.id, dir));
            }
            | (Some(s), None) if ship.halite < self.halite[ship_idx] / 10 => {
                assert!(s == ship_pos);
                info!("{}: out of halite; ship {} staying still", round, ship.id);
                self.reserved.insert((s, round + 1));
                commands.push(Command::Move(ship.id, Dir::O));
            }
            | _ => invalid.push(ship.id),
            }
        }

        mem::replace(self.routes, routes);
        self.reserved.retain(|(_, t)| *t >= round);

        info!("{}: Routes after execution", round);
        info!("{}: {:?}", round, self.routes);
        info!("{}: {:?}", round, self.reserved);

        invalid
    }

    // Should be called for current round?
    pub fn clear_route(&mut self, id: ID) {
        let route = self.routes.remove(&id);
        if let Some(route) = route {
            let mut round = self.round;
            for pos in route {
                self.reserved.remove(&(pos, round));
                round += 1;
            }
        }
    }

    pub fn plan_route(&mut self, ship: &Ship, end_pos: Pos, depth: Time) -> Command {

        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        struct Node {
            pos: Pos,
            halite: Halite,
            heuristic: Time,
            round: Time,
        }

        impl Ord for Node {
            fn cmp(&self, rhs: &Self) -> cmp::Ordering {
                rhs.heuristic.cmp(&self.heuristic)
                    .then_with(|| rhs.halite.cmp(&self.halite))
                    .then_with(|| rhs.round.cmp(&self.round))
                    .then_with(|| self.pos.cmp(&rhs.pos))
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
                Some(self.cmp(rhs))
            }
        }

        let start_pos = ship.into();
        let start_idx = self.idx(start_pos);
        let cost = self.halite[start_idx] / 10;

        // Starting position is the same as ending position or we're stuck
        if start_pos == end_pos || ship.halite < cost {
            for dir in iter::once(&Dir::O).chain(&DIRS) {
                let end_pos = self.step(start_pos, *dir);
                let end_round = self.round + 1;
                if !self.reserved.contains(&(end_pos, end_round)) {
                    self.routes.entry(ship.id)
                        .or_default()
                        .push_front(end_pos);
                    self.reserved.insert((end_pos, end_round));
                    return Command::Move(ship.id, *dir)
                }
            }

            // Doomed to crash
            return Command::Move(ship.id, Dir::O);
        }

        let cutoff = self.round + depth;
        let mut queue: BinaryHeap<Node> = BinaryHeap::default();
        let mut costs: FnvHashMap<(Pos, Time), Time> = FnvHashMap::default();
        let mut retrace: FnvHashMap<Node, Node> = FnvHashMap::default();
        let mut seen: FnvHashSet<(Pos, Time)> = FnvHashSet::default();

        costs.insert((start_pos, self.round), 0);

        queue.push(Node {
            pos: start_pos,
            halite: ship.halite,
            heuristic: 0,
            round: self.round,
        });

        while let Some(node) = queue.pop() {

            let node_pos = node.pos;
            let node_idx = self.idx(node_pos);
            let cost = self.halite[node_idx] / 10;

            // Stuck or found path
            if node.halite < cost || node.pos == end_pos || node.round == cutoff {

                let next = {
                    let mut step = Some(node);
                    let mut route = self.routes.entry(ship.id)
                        .or_default();

                    while let Some(prev) = step {
                        if retrace.get(&prev).is_none() { break }
                        route.push_front(prev.pos);
                        self.reserved.insert((prev.pos, prev.round));
                        step = retrace.remove(&prev);
                    }

                    if node.halite < cost {
                        self.reserved.insert((node.pos, node.round + 1));
                    }

                    info!("{}: reserving route for {:?} to {:?}: {:?}", self.round, ship, end_pos, route);
                    route.front()
                        .cloned()
                        .expect("[INTERNAL ERROR]: no next position in path")
                };

                return Command::Move(ship.id, self.inv_step(start_pos, next))
            }

            seen.insert((node_pos, node.round));

            for dir in DIRS.iter().chain(iter::once(&Dir::O)) {

                // Don't squat on spawn plz
                if node_pos == self.spawn && *dir == Dir::O {
                    continue
                }

                let next_pos = self.step(node_pos, *dir);
                let next_idx = self.idx(next_pos);
                let next_halite = if dir == &Dir::O { node.halite } else { node.halite - cost };
                let next_round = node.round + 1;
                let next_cost = costs[&(node_pos, node.round)]
                    + if dir == &Dir::O { 0 } else { 1 }
                    + 1;

                if self.reserved.contains(&(next_pos, next_round))
                || seen.contains(&(next_pos, next_round))
                || (next_halite < self.halite[next_idx] / 10 && self.reserved.contains(&(next_pos, next_round + 1)))
                {
                    continue
                }

                if let Some(prev_cost) = costs.get(&(next_pos, next_round)) {
                    if next_cost >= *prev_cost {
                        continue
                    }
                }

                let heuristic = self.dist(next_pos, end_pos) as Time * 2;

                let next_node = Node {
                    pos: next_pos,
                    halite: next_halite,
                    heuristic: next_cost + heuristic,
                    round: next_round,
                };

                costs.insert((next_pos, next_round), next_cost);
                queue.push(next_node);
                retrace.insert(next_node, node);
            }
        }

        // guess I'll die
        error!("[INTERNAL ERROR]: pathfinding failed");
        Command::Move(ship.id, Dir::O)
    }
}
