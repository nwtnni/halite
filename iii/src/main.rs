extern crate log; 
extern crate failure;
extern crate simplelog;
extern crate serde_json;

extern crate my_bot;

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use simplelog::*;

use my_bot::{Executor, State};

fn main() -> Result<(), failure::Error> {

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut reader = BufReader::new(stdin.lock())
        .lines()
        .filter_map(|line| line.ok())
        .peekable();

    let initial = reader.next().expect("[INTERNAL ERROR]: missing constants");
    let constants = serde_json::from_str(&initial)?;
    let mut writer = BufWriter::new(stdout.lock());
    let mut state = State::initialize(&mut reader);
    let log = format!("halite-{}.log", state.id);

    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create(log)?
    )?;

    write!(writer, "nwtnni-{}\n", state.id)?;
    writer.flush()?;

    let total = state.halite.iter()
        .sum::<usize>();

    let mut executor = Executor::new(total);
    
    loop {
        // Game over
        if reader.peek().map_or(true, |line| line.is_empty()) {
            return Ok(())
        }

        state.update(&mut reader);

        for command in executor.execute(&constants, &state) {
            write!(writer, "{} ", command.to_string())?;
        }

        write!(writer, "\n")?;
        writer.flush()?;
    }
}
