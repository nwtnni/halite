use std::f32::consts::PI;

// Name
pub const BOT_NAME: &str = "Testing";

// Ship radius
pub const SHIP_RADIUS: f32 = 0.5;

// Max thrust
pub const SHIP_SPEED: f32 = 7.0;

// Weapon AoE
pub const WEAPON_RADIUS: f32 = 5.0;

// Damage per turn
pub const WEAPON_DAMAGE: i32 = 64;

// Planet explosion radius
pub const EXPLODE_RADIUS: f32 = 10.0;

// Maximum distance to dock
pub const DOCK_RADIUS: f32 = 4.0;

// Turns required to dock
pub const DOCK_TURNS: i32 = 5;

// Productivity per ship
pub const PRODUCTIVITY: i32 = 6;

//
// Implementation-Specific
//

// HashGrid bin sizes
pub const X_GRID_SCALE: f32 = 32.0;
pub const Y_GRID_SCALE: f32 = 32.0;

// Angle to turn in degrees if failed to navigate
pub const DELTA_THETA: f32 = PI / 50.0;

// How many times to re-attempt navigation
pub const CORRECTIONS: i32 = 200;
