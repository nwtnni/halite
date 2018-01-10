use fnv::FnvHashMap;
use hlt::state::*;
use hlt::constants::*;
use hlt::collision::*;

type Env = FnvHashMap<ID, (Vec<Ship>, Vec<Ship>)>;

pub struct Scout {
    combat: Env,
    planets: Env,
}

impl Scout {
    pub fn new() -> Self {
        Scout {
            combat: FnvHashMap::default(),
            planets: FnvHashMap::default(),
        }
    }

    fn insert<T: ToEntity>(env: &mut Env, g: &Grid, t: &T, r: f64, ships: &Ships) {
        let e = t.to_entity();
        env.insert(e.id(), g
            .near(t, e.rad() + r)
            .into_iter()
            .filter_map(|&entity| {
                match entity {
                    Entity::Ship(_, _, _, id) => Some(ships[&id].clone()),
                    _ => None,
                }
            }).partition(|ship| ship.owner == g.owner));
    }

    pub fn initialize(&mut self, grid: &Grid, ships: &Ships, planets: &Planets) {
        for ship in ships.values() {
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

    pub fn get_combat(&self, ship: ID) -> &(Vec<Ship>, Vec<Ship>) {
        &self.combat[&ship]
    }

    pub fn get_env(&self, planet: ID) -> &(Vec<Ship>, Vec<Ship>) {
        &self.planets[&planet]
    }
}
