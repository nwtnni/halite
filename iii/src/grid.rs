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

    /// A route is invalid if:
    /// - The ship no longer exists
    /// - The ship is stuck this turn
    /// - The ship's next step is blocked by an enemy
    /// - The ship doesn't have a route
    /// - The ship's current location doesn't match its route
    /// - The ship's new destination no longer matches its route
    pub fn invalidate_routes(&mut self, ships: &[Ship], destinations: &[Pos]) -> Vec<ID> {
        let alive = ships.iter()
            .map(|ship| ship.id)
            .collect::<FnvHashSet<_>>();

        // Ships that no longer exist
        let dead = self.routes.keys()
            .filter(|id| !alive.contains(id))
            .cloned()
            .collect::<Vec<_>>();

        let mut invalid = Vec::new();

        // Check that ships aren't stuck, blocked, or re-routed
        for (i, ship) in ships.iter().enumerate() {
            let idx = self.idx(ship.into());
            let cost = self.halite[idx] / 10;
            let enemies = self.enemies_around(ship.into(), 1);
            if ship.halite >= cost && enemies == 0 {
                let first = self.peek_first(ship.id);
                let last = self.peek_last(ship.id);
                if first == Some(ship.into())
                && last == Some(destinations[i]) {
                    continue
                }
            }
            invalid.push(ship.id);
        }

        // Clean up reservations
        for id in dead.iter().chain(&invalid) {
            self.clear_route(*id);
        }

        // Return ships that need to repath
        invalid
    }

    fn peek_first(&self, id: ID) -> Option<Pos> {
        self.routes.get(&id)
            .and_then(|route| route.front())
            .cloned()
    }

    fn peek_last(&self, id: ID) -> Option<Pos> {
        self.routes.get(&id)
            .and_then(|route| route.back())
            .cloned()
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

    /// Returns cached commands
    /// Should be called after invalidating routes
    pub fn execute_routes(&mut self) -> Vec<Command> {
        let mut commands = Vec::new();
        let mut routes = mem::replace(self.routes, FnvHashMap::default());
        for (id, route) in routes.iter_mut() {
            let start = route.pop_front();
            let end = route.front();
            match (start, end) {
            | (Some(s), Some(e)) => {
                let dir = self.inv_step(s, *e);
                self.reserved.remove(&(s, self.round));
                commands.push(Command::Move(*id, dir));
            }
            | (Some(s), None) => {
                self.reserved.remove(&(s, self.round));
                commands.push(Command::Move(*id, Dir::O));
            }
            | _ => panic!("[INTERNAL ERROR]: no route left"),
            }
        }
        mem::replace(self.routes, routes);
        commands
    }

    pub fn plan_route(&mut self, ship: &Ship, end: Pos) -> Command {

        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        struct Node {
            pos: Pos,
            halite: Halite,
            round: Time,
        }

        impl Ord for Node {
            fn cmp(&self, rhs: &Self) -> cmp::Ordering {
                rhs.round.cmp(&self.round)
                    .then_with(|| rhs.halite.cmp(&self.halite))
                    .then_with(|| self.pos.cmp(&rhs.pos))
            }
        }

        impl PartialOrd for Node {
            fn partial_cmp(&self, rhs: &Self) -> Option<cmp::Ordering> {
                Some(self.cmp(rhs))
            }
        }

        unimplemented!()
    }
}
