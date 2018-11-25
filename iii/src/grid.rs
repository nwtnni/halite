use std::cmp;
use std::iter;
use std::usize;
use std::collections::{BinaryHeap, VecDeque};

use fixedbitset::FixedBitSet;
use fnv::{FnvHashSet, FnvHashMap};

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
    reserved: &'round mut FnvHashMap<(Pos, Time), ID>,
    routes: &'round mut FnvHashMap<ID, VecDeque<(Pos, Time)>>,
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
        reserved: &'round mut FnvHashMap<(Pos, Time), ID>,
        routes: &'round mut FnvHashMap<ID, VecDeque<(Pos, Time)>>,
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
    pub fn idx(&self, Pos(x, y): Pos) -> usize {
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
        !self.reserved.contains_key(&(self.spawn, self.round + 1))
    }

    // Should be called for current round?
    pub fn clear_route(&mut self, id: ID) {
        self.routes.remove(&id);
        self.reserved.retain(|(_, _), reserved| id != *reserved);
    }

    // Call to clean up after pathfinding a round
    pub fn clear_round(&mut self, round: Time) {
        self.reserved.retain(|(_, t), _| *t > round);
    }

    pub fn navigate(&mut self, ship: &Ship, end_pos: Pos, depth: Time, crash: bool) -> (Option<ID>, Command) {

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
        let round = self.round;

        // info!("Beginning navigation for {:?} from {:?} to {:?}", ship.id, start_pos, end_pos);

        // Try to follow cached route
        if let Some(mut route) = self.routes.remove(&ship.id) {

            // if route.len() > 1 {
                // info!("Found cached route: {:?}", route);
            // }

            let cached_start = route.pop_front();
            let cached_end = route.front().cloned();

            if let (Some((cached_start_pos, t1)), Some((cached_end_pos, t2))) = (cached_start, cached_end) {

                assert!(cached_start_pos == start_pos);
                assert!(self.dist(cached_start_pos, cached_end_pos) <= 1);
                assert!(t1 == round);
                assert!(t2 == round + 1);
                assert!(ship.halite >= cost);

                let cached_end_idx = self.idx(cached_end_pos);

                // If enemies around, sit and wait
                if self.enemies_around(cached_end_pos, 1) > 0 && !self.drops[cached_end_idx] {
                    route.push_front((cached_start_pos, t1)); 
                    let repath = self.reserved.insert((cached_start_pos, t1 + 1), ship.id);
                    for (_, t) in &mut route { *t += 1; }
                    self.routes.insert(ship.id, route);
                    return (repath, Command::Move(ship.id, Dir::O))
                }

                // Safe to follow cached route if no ally has overwritten our reservation
                if self.reserved.get(&(cached_end_pos, round + 1)) == Some(&ship.id) {
                    if route.len() > 1 { self.routes.insert(ship.id, route); }
                    self.reserved.remove(&(cached_start_pos, round));
                    let dir = self.inv_step(cached_start_pos, cached_end_pos);
                    // info!("Safe to follow cached route; stepping {:?}", dir);
                    return (None, Command::Move(ship.id, dir))
                }

                // info!("Route invalidated; beginning repathing");
            }
        }

        // Reset
        self.clear_route(ship.id);

        // Stuck
        if ship.halite < cost {
            // info!("Out of fuel or start == end; reserving {:?}", (start_pos, round + 1));
            let repath = self.reserved.insert((start_pos, round + 1), ship.id);
            return (repath, Command::Move(ship.id, Dir::O))
        }

        // Starting position is the same as ending position: check for enemies
        if start_pos == end_pos {
            let mut min_dir = Dir::O;
            let mut min_enemies = self.enemies_around(start_pos, 2);
            if min_enemies > 0 {
                for dir in &DIRS {
                    let step = self.step(start_pos, *dir);
                    let enemies = self.enemies_around(step, 2);
                    if enemies < min_enemies {
                        min_dir = *dir;
                        min_enemies = enemies;
                    }
                }
            }

            let end_pos = self.step(start_pos, min_dir);
            let repath = self.reserved.insert((end_pos, round + 1), ship.id);

            // info!("Start == end; reserving {:?}", (end_pos, round + 1));
            return (repath, Command::Move(ship.id, min_dir))
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
                        route.push_front((prev.pos, prev.round));
                        self.reserved.insert((prev.pos, prev.round), ship.id);
                        step = retrace.remove(&prev);
                    }

                    // info!("Reserving route for ship {} to {:?}: {:?}", ship.id, end_pos, route);
                    route.front()
                        .cloned()
                        .expect("[INTERNAL ERROR]: no next position in path")
                };

                return (None, Command::Move(ship.id, self.inv_step(start_pos, next.0)))
            }

            seen.insert((node_pos, node.round));

            for dir in DIRS.iter().chain(iter::once(&Dir::O)) {

                let next_pos = self.step(node_pos, *dir);
                let next_halite = if dir == &Dir::O { node.halite } else { node.halite - cost };
                let next_round = node.round + 1;
                let next_cost = costs[&(node_pos, node.round)]
                    + if dir == &Dir::O { 0 } else { 1 }
                    + 1;

                if (self.reserved.contains_key(&(next_pos, next_round)) && !(crash && next_pos == self.spawn))
                || (self.enemies_around(next_pos, 2) > 0 && self.dist(next_pos, self.spawn) >= 2 && self.dist(next_pos, start_pos) >= 2)
                || (self.enemies_around(next_pos, 0) > 0 && self.dist(next_pos, self.spawn) >= 1 && self.dist(next_pos, start_pos) >= 1)
                || seen.contains(&(next_pos, next_round)) {
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
        warn!("[INTERNAL ERROR]: pathfinding failed for ship {} from {:?} to {:?}", ship.id, start_pos, end_pos);
        let mut min_dir = Dir::O;
        let mut min_enemies = self.enemies_around(start_pos, 2);
        let mut min_dist = self.dist(start_pos, end_pos);

        if min_enemies > 0 {
            for dir in &DIRS {
                let step = self.step(start_pos, *dir);
                let enemies = self.enemies_around(step, 2);
                let dist = self.dist(step, end_pos);
                if (enemies == 0 && dist < min_dist) || enemies < min_enemies {
                    min_dir = *dir;
                    min_enemies = enemies;
                    min_dist = dist;
                }
            }
        }

        let end_pos = self.step(start_pos, min_dir);
        let repath = self.reserved.insert((end_pos, round + 1), ship.id);

        // info!("Start == end; reserving {:?}", (end_pos, round + 1));
        (repath, Command::Move(ship.id, min_dir))
    }
}
