#![allow(dead_code)]

pub const RETURN: usize = 50;

pub const CAPTURE_RADIUS: usize = 3;
pub const DEFAULT_MAP_HEIGHT: usize = 48;
pub const DEFAULT_MAP_WIDTH: usize = 48;
pub const DROPOFF_COST: usize = 4000;
pub const DROPOFF_PENALTY_RATIO: usize = 4;
pub const EXTRACT_RATIO: usize = 4;
pub const FACTOR_EXP_1: f32 = 2.0;
pub const FACTOR_EXP_2: f32 = 2.0;
pub const INITIAL_ENERGY: usize = 5000;
pub const INSPIRATION_ENABLED: bool =  true;
pub const INSPIRATION_RADIUS: usize = 4;
pub const INSPIRATION_SHIP_COUNT: usize = 2;
pub const INSPIRED_BONUS_MULTIPLER: f32 = 2.0;
pub const INSPIRED_EXTRACT_RATIO: usize = 4;
pub const INSPIRED_MOVE_COST_RATIO: usize = 10;
pub const MAX_CELL_PRODUCTION: usize = 1000;
pub const MAX_ENERGY: usize = 1000;
pub const MAX_PLAYERS: usize = 16;
pub const MAX_TURNS: usize = 400;
pub const MAX_TURN_THRESHOLD: usize = 64;
pub const MIN_CELL_PRODUCTION: usize = 900;
pub const MIN_TURNS: usize = 400;
pub const MIN_TURN_THRESHOLD: usize = 32;
pub const MOVE_COST_RATIO: usize = 10;
pub const NEW_ENTITY_ENERGY_COST: usize = 1000;
pub const PERSISTENCE: f32 = 0.7;
pub const SHIPS_ABOVE_FOR_CAPTURE: usize = 3;
pub const STRICT_ERRORS: bool = false;