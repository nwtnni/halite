#![allow(dead_code, non_snake_case)]

pub const RETURN: usize = 800;

fn capture_enabled() -> bool { false }
fn capture_radius() -> usize { 3 }
fn default_map_height() -> usize { 48 }
fn default_map_width() -> usize { 48 }
fn dropoff_cost() -> usize { 4000 }
fn dropoff_penalty_ratio() -> usize { 4 }
fn extract_ratio() -> usize { 4 }
fn factor_exp_1() -> f32 { 2.0 }
fn factor_exp_2() -> f32 { 2.0 }
fn initial_energy() -> usize { 5000 }
fn inspiration_enabled() -> bool { true }
fn inspiration_radius() -> usize { 4 }
fn inspiration_ship_count() -> usize { 2 }
fn inspired_bonus_multiplier() -> f32 { 2.0 }
fn inspired_extract_ratio() -> usize { 4 }
fn inspired_move_cost_ratio() -> usize { 10 }
fn max_cell_production() -> usize { 1000 }
fn max_energy() -> usize { 1000 }
fn max_players() -> usize { 16 }
fn max_turns() -> usize { 400 }
fn max_turn_threshold() -> usize { 64 }
fn min_cell_production() -> usize { 900 }
fn min_turns() -> usize { 400 }
fn min_turn_threshold() -> usize { 32 }
fn move_cost_ratio() -> usize { 10 }
fn new_entity_energy_cost() -> usize { 1000 }
fn persistence() -> f32 { 0.7 }
fn ships_above_for_capture() -> usize { 3 }
fn strict_errors() -> bool { false }

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Constants {

    #[serde(default = "capture_enabled")]
    pub CAPTURE_ENABLED: bool,

    #[serde(default = "capture_radius")]
    pub CAPTURE_RADIUS: usize,

    #[serde(default = "default_map_height")]
    pub DEFAULT_MAP_HEIGHT: usize,

    #[serde(default = "default_map_width")]
    pub DEFAULT_MAP_WIDTH: usize,

    #[serde(default = "dropoff_cost")]
    pub DROPOFF_COST: usize,

    #[serde(default = "dropoff_penalty_ratio")]
    pub DROPOFF_PENALTY_RATIO: usize,

    #[serde(default = "extract_ratio")]
    pub EXTRACT_RATIO: usize,

    #[serde(default = "factor_exp_1")]
    pub FACTOR_EXP_1: f32,

    #[serde(default = "factor_exp_2")]
    pub FACTOR_EXP_2: f32,

    #[serde(default = "initial_energy")]
    pub INITIAL_ENERGY: usize,

    #[serde(default = "inspiration_enabled")]
    pub INSPIRATION_ENABLED: bool,

    #[serde(default = "inspiration_radius")]
    pub INSPIRATION_RADIUS: usize,

    #[serde(default = "inspiration_ship_count")]
    pub INSPIRATION_SHIP_COUNT: usize,

    #[serde(default = "inspired_bonus_multiplier")]
    pub INSPIRED_BONUS_MULTIPLIER: f32,

    #[serde(default = "inspired_extract_ratio")]
    pub INSPIRED_EXTRACT_RATIO: usize,

    #[serde(default = "inspired_move_cost_ratio")]
    pub INSPIRED_MOVE_COST_RATIO: usize,

    #[serde(default = "max_cell_production")]
    pub MAX_CELL_PRODUCTION: usize,

    #[serde(default = "max_energy")]
    pub MAX_ENERGY: usize,

    #[serde(default = "max_players")]
    pub MAX_PLAYERS: usize,

    #[serde(default = "max_turns")]
    pub MAX_TURNS: usize,

    #[serde(default = "max_turn_threshold")]
    pub MAX_TURN_THRESHOLD: usize,

    #[serde(default = "min_cell_production")]
    pub MIN_CELL_PRODUCTION: usize,

    #[serde(default = "min_turns")]
    pub MIN_TURNS: usize,

    #[serde(default = "min_turn_threshold")]
    pub MIN_TURN_THRESHOLD: usize,

    #[serde(default = "move_cost_ratio")]
    pub MOVE_COST_RATIO: usize,

    #[serde(default = "new_entity_energy_cost")]
    pub NEW_ENTITY_ENERGY_COST: usize,

    #[serde(default = "persistence")]
    pub PERSISTENCE: f32,

    #[serde(default = "ships_above_for_capture")]
    pub SHIPS_ABOVE_FOR_CAPTURE: usize,

    #[serde(default = "strict_errors")]
    pub STRICT_ERRORS: bool,

    #[serde(default, rename = "game_seed")]
    pub GAME_SEED: usize,
}
