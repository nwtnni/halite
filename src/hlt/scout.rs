use fnv::FnvHashMap;
use hlt::state::*;
use hlt::constants::*;
use hlt::collision::*;

type Environment = FnvHashMap<ID, (i32, i32)>;

pub struct Scout {
    combat: Environment,
    planets: Environment,
}

impl Scout {
    pub fn new() -> Self {
        Scout {
            combat: FnvHashMap::default(),
            planets: FnvHashMap::default(),
        }
    }

    fn insert<T: ToEntity>(env: &mut Environment, grid: &Grid,
                           t: &T, r: f64, ships: &Ships) {
        let (allies, enemies): (Vec<_>, Vec<_>) = grid
            .near(t, r)
            .into_iter()
            .filter_map(|&entity| {
                match entity {
                    Entity::Ship(_, _, _, id) => Some(&ships[&id]),
                    _ => None,
                }
            }).partition(|ship| ship.owner == grid.owner);
        let id = t.to_entity().id();
        env.insert(id, (allies.len() as i32, enemies.len() as i32));
    }

    pub fn initialize(&mut self, grid: &Grid, ships: &Ships, planets: &Planets) {
        for ship in ships.values() {
            if ship.owner != grid.owner { continue }
            Self::insert(&mut self.combat, &grid, &ship, COMBAT_RADIUS, ships);
        }

        for planet in planets.values() {
            if planet.is_owned(grid.owner) {
                Self::insert(&mut self.planets, &grid, &planet, DEFEND_RADIUS, ships);
            } else if planet.is_enemy(grid.owner) {
                Self::insert(&mut self.planets, &grid, &planet, RAID_RADIUS, ships);
            } else {
                Self::insert(&mut self.planets, &grid, &planet, CLAIM_RADIUS, ships);
            }
        }
    }

    pub fn get_combat(&self, ship: ID) -> (i32, i32) {
        self.combat[&ship]
    }

    pub fn get_env(&self, planet: ID) -> (i32, i32) {
        self.planets[&planet]
    }
}
