#[macro_use]
extern crate log; 
extern crate simplelog;
extern crate failure;

extern crate MyBot;

use std::io::{BufRead, BufReader, BufWriter, Write};
use std::fs::File;

use simplelog::*;

use MyBot::{execute, State};

fn main() -> Result<(), failure::Error> {

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    let mut reader = BufReader::new(stdin.lock())
        .lines()
        .skip(1)
        .filter_map(|line| line.ok())
        .peekable();

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
    
    loop {
        // Game over
        if reader.peek().map_or(true, |line| line.is_empty()) {
            return Ok(())
        }

        state.update(&mut reader);

        for command in execute(&state) {
            write!(writer, "{} ", command.to_string())?;
        }

        write!(writer, "\n")?;
        writer.flush()?;
    }
}
