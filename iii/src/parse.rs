use std::io::BufRead;
use std::str::FromStr;

use data;

fn parse_usize<'a, S: Iterator<Item = &'a str>>(stream: &mut S) -> usize {
    stream.next()
        .and_then(|x| usize::from_str(x).ok())
        .expect("[INTERNAL ERROR]: expected usize")
}

impl data::State {
    pub fn initialize<'a, S: Iterator<Item = &'a str>>(stream: &mut S) -> Self {
        let num_players = parse_usize(stream);
        let id = parse_usize(stream);
        let drops = Vec::with_capacity(0);
        let ships = Vec::with_capacity(0);
        let mut scores = Vec::with_capacity(num_players);
        let mut yards = Vec::with_capacity(num_players);

        // Initialize players
        for _ in 0..num_players {
            let id = parse_usize(stream);
            let x = parse_usize(stream);
            let y = parse_usize(stream);
            scores[id] = 0;
            yards[id] = data::Shipyard{ owner: id, x, y };
        }

        // Initialize ap data
        let round = 0;
        let width = parse_usize(stream);
        let height = parse_usize(stream);
        let mut halite = Vec::with_capacity(width * height);
        for _ in 0..width * height {
            halite.push(parse_usize(stream));
        }

        data::State { id, width, height, round, scores, drops, ships, yards, halite }
    }

    pub fn update<'a, S: Iterator<Item = &'a str>>(&mut self, stream: &mut S) {

        // Clear outdated ships and dropoffs
        self.ships = Vec::new();
        self.drops = Vec::new();
        self.round = parse_usize(stream);

        // Player updates
        for _ in 0..self.scores.len() {
            let player = parse_usize(stream);
            let num_ships = parse_usize(stream);
            let num_dropoffs = parse_usize(stream);
            self.scores[player] = parse_usize(stream);

            // Ship updates
            for _ in 0..num_ships {
                let id = parse_usize(stream);
                let x = parse_usize(stream);
                let y = parse_usize(stream);
                let halite = parse_usize(stream);
                self.ships.push(data::Ship { owner: player, id, x, y, halite });
            }

            // Dropoff updates
            for _ in 0..num_dropoffs {
                let _ = parse_usize(stream);
                let x = parse_usize(stream);
                let y = parse_usize(stream);
                self.drops.push(data::Dropoff { owner: player, x, y });
            }
        }

        // Map updates
        for _ in 0..parse_usize(stream) {
            let x = parse_usize(stream);
            let y = parse_usize(stream);
            self.halite[y * self.width + x] = parse_usize(stream);
        }
    }
}
