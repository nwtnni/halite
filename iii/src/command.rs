use data::ID;
use grid::Dir;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Command {
    Spawn,
    Transform(ID),
    Move(ID, Dir),
}

impl ToString for Command {
    fn to_string(&self) -> String {
        match self {
        | Command::Spawn         => "g".to_string(),
        | Command::Transform(id) => format!("c {}", id),
        | Command::Move(id, dir) => format!("m {} {}", id, dir.to_string()),
        }
    }
}

impl ToString for Dir {
    fn to_string(&self) -> String {
        match self {
        | Dir::N => "n",
        | Dir::S => "s",
        | Dir::E => "e",
        | Dir::W => "w",
        | Dir::O => "o",
        }.to_string()
    }
}
