use fixedbitset::FixedBitSet;
use fnv::FnvHashMap;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Dir {
    N, S, E, W
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pos(usize, usize);

#[derive(Clone, Debug)]
pub struct Grid {
    width: usize,
    height: usize,
    halite: Vec<usize>,
    occupied: FixedBitSet,
    dropoff: FnvHashMap<Pos, bool>,
}
