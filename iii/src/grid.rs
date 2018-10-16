#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    N, S, E, W
}

impl ToString for Dir {
    fn to_string(&self) -> String {
        match self {
        | Dir::N => "n",
        | Dir::S => "s",
        | Dir::E => "e",
        | Dir::W => "w",
        }.to_string()
    }
}
