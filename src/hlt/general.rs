use hlt::collision::*;
use hlt::constants::*;
use hlt::command::*;
use hlt::state::*;
use hlt::strategy::*;

pub trait General {
    fn run(self);
    fn step(&mut self);
}

impl General for State {
    fn run(mut self) {
        loop {
            self.update();
            self.step();
            self.queue.flush();
        }
    }

    fn step(&mut self) {
        let player = &self.players[self.id];
        for id in &player.ships {
            let ship = &self.ships[id];
            if ship.is_docked() { continue }

            //
            // SHORT RANGE STRATEGIES
            //

            let enemies = self.grid.near_enemies(&ship, WEAPON_RADIUS, &self.ships);
            let docking = enemies.iter()
                .filter(|enemy| enemy.is_docked())
                .collect::<Vec<_>>();
            let allies = self.grid.near_allies(&ship, 14.0, &self.ships);

            // Non-docking enemies nearby
            if enemies.len() - docking.len() > (allies.len() / 2) {
                let enemy = &enemies[0];
                let ally = allies.iter()
                    .find(|ally| ally.distance_to(enemy) > WEAPON_RADIUS);
                self.plan.set(ship.id, Tactic::Retreat(enemy.id));

                if let Some(ally) = ally {
                    let point = Entity::Obstacle(ally.x, ally.y, 0.0, 0);
                    self.queue.push(
                        &navigate(&mut self.grid, ship, &point)
                    );
                    continue
                }

                let (x, y) = ship.retreat_from(enemy, 7);
                let point = Entity::Obstacle(x, y, 0.0, 0);
                self.queue.push(
                    &navigate(&mut self.grid, ship, &point)
                );
                continue
            }

            // Keep attacking docking enemies in weapon range
            if docking.len() > 0 {
                let docked = docking[0];
                self.plan.set(ship.id, Tactic::Attack(docked.id));
                self.queue.push(
                    &navigate(&mut self.grid, ship, docked)
                );
                continue
            }

            // Try defending
            let defend = self.grid.near_planets(&ship, 21.0, &self.planets)
                .into_iter()
                .filter(|planet| planet.is_owned(self.id))
                .filter_map(|planet| {
                    let e = self.grid.near_enemies(
                        &planet, planet.rad + DOCK_RADIUS, &self.ships
                    ).len() as i32;
                    let o = self.plan.defending(planet.id);
                    if e > o/2 { Some((planet, e, o)) } else { None }
                })
                .min_by_key(|&(ref planet, e, o)| {
                    let d = ship.distance_to(planet) as i32;
                    let s = planet.docked();
                    d - s - (e - o).pow(3)
                }).map(|(planet, _, _)| planet);

            if let Some(ref planet) = defend {
                let ally = &self.ships[&planet.ships[0]];
                self.plan.set(ship.id, Tactic::Defend(planet.id));
                self.queue.push(&navigate(&mut self.grid, ship, &ally));
                continue
            }

            // Try docking
            let free = self.planets.values()
                .filter(|planet| planet.has_spots() && !planet.is_enemy(self.id))
                .min_by_key(|planet| {
                    let d = ship.distance_to(planet) as i32;
                    let e = self.grid.near_enemies(
                        planet, planet.rad + DOCK_RADIUS + 7.0, &self.ships
                    ).len() as i32;
                    let a = self.grid.near_allies(
                        planet, planet.rad + DOCK_RADIUS + 7.0, &self.ships
                    ).len() as i32;
                    let o = self.plan.docking_at(planet.id);
                    let s = planet.spots();
                    d.pow(2) + (e - a).pow(2) + (o - s)
                });

            // Found somewhere to dock
            if let Some(ref planet) = free {
                self.plan.set(ship.id, Tactic::Dock(planet.id));
                if ship.in_docking_range(planet) {
                    self.queue.push(&dock(ship, planet));
                } else {
                    self.queue.push(&navigate(&mut self.grid, ship, planet));
                }
                continue
            }

            //
            // LONG RANGE STRATEGIES
            //

            // We've been recruited
            if let Some(id) = self.plan.has_target(ship.id) {
                let target = &self.ships[&id];
                self.queue.push(&navigate(&mut self.grid, ship, &target));
                continue
            }

            let ready = allies.iter()
                .filter(|ally| self.plan.is_available(ally.id))
                .collect::<Vec<_>>();

            // Find a weak enemy planet
            let weak = self.planets.values()
                .filter(|planet| planet.is_enemy(self.id))
                .filter_map(|planet| {
                    let enemies = self.grid.near_enemies(
                        &planet, planet.rad + DOCK_RADIUS + 7.0, &self.ships
                    );
                    let threats = enemies.len();
                    let docked = enemies.into_iter()
                        .filter(|enemy| enemy.is_docked())
                        .collect::<Vec<_>>();

                    let adv = ready.len() - (threats - docked.len())*2;
                    if adv > 0 {
                        Some((planet, docked, adv as i32))
                    } else { None }
                }).min_by_key(|&(planet, _, adv)| {
                    let d = ship.distance_to(&planet) as i32;
                    d + adv.pow(2)
                }).map(|(_, docked, _)| docked[0].clone());

            if let Some(ref target) = weak {
                for ally in &ready {
                    self.plan.set(ally.id, Tactic::Attack(target.id));
                }
                self.plan.set(ship.id, Tactic::Attack(target.id));
                self.queue.push(&navigate(&mut self.grid, ship, &target));
            }
        }
    }
}
