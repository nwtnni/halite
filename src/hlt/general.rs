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

            // Get local information
            let enemies = self.grid.near_enemies(&ship, 12.0, &self.ships);
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

                // Flee towards ally
                if let Some(ally) = ally {
                    self.queue.push(
                        &navigate_to_ally(&mut self.grid, ship, &ally)
                    );
                    continue
                }

                // No ally nearby; flee from ship
                self.queue.push(
                    &navigate_to_point(&mut self.grid, ship, ship.retreat_from(enemy, 7))
                );
                continue
            }

            // Keep attacking docking enemies in weapon range
            if docking.len() > 0 {
                let docked = docking[0];
                self.plan.set(ship.id, Tactic::Attack(docked.id));
                self.queue.push(
                    &navigate_to_enemy(&mut self.grid, ship, docked)
                );
                continue
            }

            // Try defending nearby planets
            let defend = self.grid.near_planets(&ship, 21.0, &self.planets)
                .into_iter()
                .filter(|planet| planet.is_owned(self.id))
                .filter_map(|planet| {
                    // Send enough to outnumber enemies
                    let e = self.grid.near_enemies(
                        &planet, planet.rad + DOCK_RADIUS, &self.ships
                    ).len() as i32;
                    let o = self.plan.defending(planet.id);
                    if e > o/2 { Some((planet, e, o)) } else { None }
                })
                .min_by_key(|&(ref planet, e, o)| {
                    // Close planets with many ships docked are good
                    let d = ship.distance_to(planet) as i32;
                    let s = planet.docked();
                    d - s - (e - o).pow(3)
                }).map(|(planet, _, _)| planet);

            if let Some(ref planet) = defend {
                let enemy = &self.grid.near_enemies(
                    planet, planet.rad + DOCK_RADIUS, &self.ships
                )[0];

                self.plan.set(ship.id, Tactic::Defend(planet.id));
                self.queue.push(&navigate_to_enemy(&mut self.grid, ship, enemy));
                continue
            }

            // Try docking at a lonely planet
            let free = self.planets.values()
                .filter_map(|planet| {
                    if !planet.has_spots() || planet.is_enemy(self.id) {
                        None
                    } else {
                        // We want a planet with free spots and no enemies
                        let e = self.grid.near_enemies(
                            &planet, planet.rad + DOCK_RADIUS + 7.0, &self.ships
                        ).len() as i32;
                        let a = self.grid.near_allies(
                            &planet, planet.rad + DOCK_RADIUS + 7.0, &self.ships
                        ).iter().filter(|ally| !ally.is_docked()).count() as i32;
                        let o = self.plan.docking_at(planet.id);
                        let s = planet.spots();
                        if s - o > 0 && a >= e {
                            Some((planet, e, a, o, s))
                        } else { None }
                    }
                })
                .min_by_key(|&(ref planet, e, a, o, s)| {
                    let d = ship.distance_to(planet) as i32;
                    d.pow(2) + (e - a).pow(2) + (o - s)
                }).map(|(planet, _, _, _, _)| planet);

            // Found somewhere to dock
            if let Some(ref planet) = free {
                if ship.in_docking_range(planet) {
                    // Don't dock if there are more enemies than allies near
                    let e = self.grid.near_enemies(
                        planet, planet.rad + DOCK_RADIUS, &self.ships
                    ).len();
                    let a = self.grid.near_allies(
                        planet, planet.rad + DOCK_RADIUS, &self.ships
                    ).len() - planet.ships.len();
                    if a >= e {
                        self.plan.set(ship.id, Tactic::Dock(planet.id));
                        self.queue.push(&dock(ship, planet));
                        continue
                    }
                } else {
                    self.plan.set(ship.id, Tactic::Dock(planet.id));
                    self.queue.push(
                        &navigate_to_planet(&mut self.grid, ship, planet, &self.plan)
                    );
                    continue
                }
            }

            //
            // LONG RANGE STRATEGIES
            //

            // We've been recruited
            if let Some(id) = self.plan.has_target(ship.id) {
                let target = &self.ships[&id];
                self.queue.push(
                    &navigate_to_enemy(&mut self.grid, ship, &target)
                );
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
                }).map(|(_, docked, _)| {
                    docked.into_iter().cloned()
                    .min_by_key(|enemy| ship.distance_to(&enemy) as i32)
                    .unwrap()
                });

            if let Some(ref target) = weak {
                for ally in &ready {
                    self.plan.set(ally.id, Tactic::Attack(target.id));
                }
                self.plan.set(ship.id, Tactic::Attack(target.id));
                self.queue.push(
                    &navigate_to_enemy(&mut self.grid, ship, &target)
                );
                continue
            }

            // Hunt down nearby ships
            let near = self.grid.near_enemies(&ship, 35.0, &self.ships);
            if let Some(&target) = near.get(0) {
                self.plan.set(ship.id, Tactic::Attack(target.id));
                self.queue.push(
                    &navigate_to_enemy(&mut self.grid, ship, &target)
                );
                continue
            }
        }
    }
}
