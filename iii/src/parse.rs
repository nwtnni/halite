use std::str::FromStr;

use data;

fn next<'a, S: Iterator<Item = &'a str>>(stream: &mut S) -> usize {
    stream.next()
        .and_then(|x| usize::from_str(x).ok())
        .expect("[INTERNAL ERROR]: expected usize")
}

impl data::State {
    pub fn initialize<S: Iterator<Item = String>>(stream: &mut S) -> Self {

        macro_rules! split {
            (|$s:ident| $e:expr) => {{
                let line = stream.next().expect("[INTERNAL ERROR]: invalid initialization data");
                let mut $s = line.split_whitespace();
                $e
            }}
        }

        let (num_players, id) = split!(|s| { (next(&mut s), next(&mut s)) });
        let round = 0;
        let drops = Vec::with_capacity(0);
        let ships = Vec::with_capacity(0);
        let mut scores = vec![0; num_players];
        let mut yards = vec![data::Shipyard{ owner: 0, x: 0, y: 0 }; num_players];

        // Initialize players
        for _ in 0..num_players {
            split!(|s| {
                let id = next(&mut s);
                let x = next(&mut s);
                let y = next(&mut s);
                scores[id] = 0;
                yards[id] = data::Shipyard{ owner: id, x, y };
            });
        }

        // Initialize map data
        let (width, height) = split!(|s| (next(&mut s), next(&mut s)));
        let mut halite = Vec::with_capacity(width * height);
        for _ in 0..height {
            split!(|s| {
                for _ in 0..width {
                    halite.push(next(&mut s));
                }
            });
        }

        data::State { id, width, height, round, scores, drops, ships, yards, halite }
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
        self.round = split!(|s| next(&mut s));

        // Player updates
        for _ in 0..self.scores.len() {

            let (player, num_ships, num_dropoffs, score) = split!(|s| {
                (next(&mut s), next(&mut s), next(&mut s), next(&mut s))    
            });

            self.scores[player] = score;

            // Ship updates
            for _ in 0..num_ships {
                split!(|s| {
                    self.ships.push(data::Ship {
                        owner:  player,
                        id:     next(&mut s),
                        x:      next(&mut s),
                        y:      next(&mut s),
                        halite: next(&mut s),
                    });
                });
            }

            // Dropoff updates
            for _ in 0..num_dropoffs {
                split!(|s| {
                    let _ = next(&mut s);
                    self.drops.push(data::Dropoff {
                        owner: player,
                        x: next(&mut s),
                        y: next(&mut s),
                    });
                });
            }
        }

        // Map updates
        let num_updates = split!(|s| next(&mut s));
        for _ in 0..num_updates {
            split!(|s| {
                let x = next(&mut s);
                let y = next(&mut s);
                let halite = next(&mut s);
                self.halite[y * self.width + x] = halite;
            });
        }
    }
}
