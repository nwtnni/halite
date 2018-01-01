#![allow(dead_code)]
use std::f32::consts::PI;

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
pub const DOCK_RADIUS: f32 = 4.00;

// Turns required to dock
pub const DOCK_TURNS: i32 = 5;

// Productivity per ship
pub const PRODUCTIVITY: i32 = 6;

//
// Implementation-Specific
//

// Grid bin size
pub const GRID_SCALE: f32 = 14.0;
pub const GRID_SCALE_2: f32 = 7.0;

// Angle to turn in degrees if failed to navigate
pub const DELTA_THETA: f32 = PI / 10.0;

// How many times to re-attempt navigation
pub const CORRECTIONS: i32 = 20;

// How favorable it is to have a large planet
pub const SIZE_MULTIPLIER: f32 = 1.5;

// Wider margin of error for collision detection
pub const FUDGE: f32 = 1.10;

