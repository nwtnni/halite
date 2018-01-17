use hlt::state::*;

// Ship radius
pub const SHIP_RADIUS: f64 = 0.5;

// Max thrust
pub const SHIP_SPEED: i32 = 7;

// Weapon AoE
pub const WEAPON_RADIUS: f64 = 6.0;

// Damage per turn
pub const WEAPON_DAMAGE: i32 = 64;

// Planet explosion radius
pub const EXPLODE_RADIUS: f64 = 10.0;

// Maximum distance to dock
pub const DOCK_RADIUS: f64 = 4.0;

// Turns required to dock
pub const DOCK_TURNS: i32 = 5;

// Productivity per ship
pub const PRODUCTIVITY: i32 = 6;

//
// Implementation-Specific
//

// Grid bin size
pub const GRID_SCALE: f64 = 14.0;
pub const GRID_SCALE_2: f64 = 7.0;
pub const LINE_RADIUS: f64 = 5.0;

// Navigation related
pub const DELTA_WIGGLE: i32 = 5;
pub const DELTA_THETA: i32 = 18;
pub const DELTA_THRUST: i32 = 2;
pub const MIN_THRUST: i32 = 1;
pub const EPSILON: f64 = 0.01;
pub const CORRECTIONS: i32 = 20;

// Scouting related
pub const COMBAT_RADIUS: f64 = 21.0;
pub const DEFEND_RADIUS: f64 = 21.0;
