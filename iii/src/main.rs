extern crate halite_iii;

use std::io::{BufRead, BufReader, BufWriter};

use halite_iii::State;

fn main() -> Result<(), std::io::Error> {

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut reader = BufReader::new(stdin.lock())
        .lines()
        .filter_map(|line| line.ok());

    let mut writer = BufWriter::new(stdout.lock());
    let mut state = match reader.next() {
    | Some(initial) => State::initialize(&mut initial.split_whitespace()),
    | None => panic!("[INTERNAL ERROR]: missing initial state"),
    };
    
    for line in reader {
        
        state.update(&mut line.split_whitespace());

    }

    Ok(())
}
