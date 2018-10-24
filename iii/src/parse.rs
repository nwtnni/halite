use std::str::FromStr;

use data::*;

macro_rules! impl_next {
    ($name:ident, $ty:ident) => {
        fn $name<'a, S: Iterator<Item = &'a str>>(stream: &mut S) -> $ty {
            stream.next()
                .and_then(|x| $ty::from_str(x).ok())
                .expect("[INTERNAL ERROR]: expected usize")
        }
    }
}

impl_next!(next_usize, usize);
impl_next!(next_pid, PID);
impl_next!(next_id, ID);
impl_next!(next_dist, Dist);
impl_next!(next_time, Time);
impl_next!(next_halite, Halite);

impl State {
    pub fn initialize<S: Iterator<Item = String>>(stream: &mut S) -> Self {

        macro_rules! split {
            (|$s:ident| $e:expr) => {{
                let line = stream.next().expect("[INTERNAL ERROR]: invalid initialization data");
                let mut $s = line.split_whitespace();
                $e
            }}
        }

        let (num_players, id) = split!(|s| {
            (next_usize(&mut s), next_pid(&mut s))
        });

        let round = 0;
        let drops = Vec::with_capacity(0);
        let ships = Vec::with_capacity(0);
        let mut scores = vec![0; num_players];
        let mut yards = vec![Shipyard{ owner: 0, x: 0, y: 0 }; num_players];

        // Initialize players
        for _ in 0..num_players {
            split!(|s| {
                let id = next_pid(&mut s);
                let x = next_dist(&mut s);
                let y = next_dist(&mut s);
                scores[id as usize] = 0;
                yards[id as usize] = Shipyard{ owner: id, x, y };
            });
        }

        // Initialize map data
        let (width, height) = split!(|s| {
            (next_dist(&mut s), next_dist(&mut s))
        });

        let mut halite = Vec::with_capacity(
            width as usize * height as usize
        );

        for _ in 0..height {
            split!(|s| {
                for _ in 0..width {
                    halite.push(next_halite(&mut s));
                }
            });
        }

        State { id, width, height, round, scores, drops, ships, yards, halite }
    }

    pub fn update<S: Iterator<Item = String>>(&mut self, stream: &mut S) {

        macro_rules! split {
            (|$s:ident| $e:expr) => {{
                let line = stream.next().expect("[INTERNAL ERROR]: invalid update data");
                let mut $s = line.split_whitespace();
                $e
            }}
        }

        // Clear outdated ships and dropoffs
        self.ships = Vec::new();
        self.drops = Vec::new();
        self.round = split!(|s| next_time(&mut s));

        // Player updates
        for _ in 0..self.scores.len() {

            let (player, num_ships, num_dropoffs, score) = split!(|s| {
                (next_pid(&mut s), next_usize(&mut s), next_usize(&mut s), next_halite(&mut s))    
            });

            self.scores[player as usize] = score;

            // Ship updates
            for _ in 0..num_ships {
                split!(|s| {
                    self.ships.push(Ship {
                        owner:  player,
                        id:     next_id(&mut s),
                        x:      next_dist(&mut s),
                        y:      next_dist(&mut s),
                        halite: next_halite(&mut s),
                    });
                });
            }

            // Dropoff updates
            for _ in 0..num_dropoffs {
                split!(|s| {
                    let _ = next_usize(&mut s);
                    self.drops.push(Dropoff {
                        owner: player,
                        x: next_dist(&mut s),
                        y: next_dist(&mut s),
                    });
                });
            }
        }

        // Map updates
        let num_updates = split!(|s| next_usize(&mut s));
        for _ in 0..num_updates {
            split!(|s| {
                let x = next_usize(&mut s);
                let y = next_usize(&mut s);
                let halite = next_halite(&mut s);
                self.halite[y * (self.width as usize) + x] = halite;
            });
        }
    }
}
