use std::{env::current_dir, fs, path::Path, sync::LazyLock};

use serde::Deserialize;

use crate::ecs::StatusEffectsBalancing;

/**
 * Loads balancing data from assets/balancing.json
 *
 * Stores it in a static singleton object `BALANCING`
 */

#[derive(Deserialize)]
pub struct PlayerBalancing {
  pub size: f32,
  pub acceleration_mod: f32,
}

#[derive(Deserialize)]
pub struct StatusEffectBalancing {
  pub steps: i32,
}

#[derive(Deserialize)]
pub struct GoblinBalancing {
  pub aggro_range: f32,
  pub lunge_force: f32,
  pub max_lunge_distance: f32,
  pub slowing_frames: i32,
  pub recovering_frames: i32,
}

impl GoblinBalancing {
  pub fn slowing_force(&self) -> f32 {
    self.lunge_force / self.slowing_frames as f32
  }
}

#[derive(Deserialize)]
pub struct ImpBalancing {
  pub aggro_range: f32,
  pub moving_initial_frames: i32,
  pub shooting_cooldown_initial_frames: i32,
  pub move_force: f32,
  pub move_distance: f32,
  pub projectile_speed: f32,
  pub projectile_damage: f32,
}

#[derive(Deserialize)]
pub struct AraneaBalancing {
  pub launch_force: f32,
  pub stopping_frames: i32,
  pub cooldown_initial_frames: i32,
  pub shooting_force: f32,
  pub projectile_damage: f32,
  pub hold_force: f32,
}

impl AraneaBalancing {
  pub fn stopping_force(&self) -> f32 {
    self.launch_force / self.stopping_frames as f32
  }
}

#[derive(Deserialize)]
pub struct AraneaQueenBalancing {
  pub max_health: f32,
  pub status_effect_threshold: f32,
  pub contact_damage: f32,
  pub colliders_side_length: f32,
  pub launch_force: f32,
  pub stopping_frames: i32,
  pub spraying_speed: f32,
  pub num_spraying: i32,
  pub spray_interval: i32,
  pub first_launch_spraying_frames: i32,
  pub first_launch_cooldown_frames: i32,
  pub phase_1_chance_of_egg_launch: f32,
  pub phase_1_spraying_frames: i32,
  pub phase_1_launch_to_egg_cooldown_frames: i32,
  pub phase_1_launch_to_player_cooldown_frames: i32,
  pub phase_2_chance_of_egg_launch: f32,
  pub phase_2_spraying_frames: i32,
  pub phase_2_launch_to_egg_cooldown_frames: i32,
  pub phase_2_launch_to_player_cooldown_frames: i32,
  pub phase_2_bounce_cooldown_frames: i32,
  pub phase_2_max_num_bounces: i32,
}

impl AraneaQueenBalancing {
  pub fn stopping_force(&self) -> f32 {
    self.launch_force / self.stopping_frames as f32
  }
}

#[derive(Deserialize)]
pub struct DefenderBalancing {
  pub damage: f32,
  pub aggro_range: f32,
  pub hold_force: f32,
  pub cooldown_initial_frames: i32,
  pub ease_period: f32,
}

#[derive(Deserialize)]
pub struct SeekerBalancing {
  pub speed_cap: f32,
  pub speed: f32,
}

#[derive(Deserialize)]
pub struct SeekerGeneratorBalancing {
  pub initial_force: f32,
  pub spawn_cooldown: i32,
}

#[derive(Deserialize)]
pub struct SniperBalancing {
  pub aggro_range: f32,
  pub cooldown_initial_frames: i32,
  pub projectile_damage: f32,
  pub shooting_force: f32,
  pub hold_force: f32,
}

#[derive(Deserialize)]
pub struct SniperGeneratorBalancing {
  pub generating_initial_frames: i32,
  pub num_snipers_generated: i32,
  pub cooldown_initial_frames: i32,
  pub generating_initial_force: f32,
}

impl SniperGeneratorBalancing {
  pub fn generating_interval(&self) -> i32 {
    self.generating_initial_frames / self.num_snipers_generated
  }
}

#[derive(Deserialize)]
pub struct LaserGateBalancing {
  pub health: f32,
  pub damage: f32,
  pub deteriorate_apply_amount: f32,
  pub beam_thickness: f32,
}

#[derive(Deserialize)]
pub struct EnemyBalancing {
  pub goblin: GoblinBalancing,
  pub imp: ImpBalancing,
  pub aranea: AraneaBalancing,
  pub aranea_queen: AraneaQueenBalancing,
  pub defender: DefenderBalancing,
  pub seeker: SeekerBalancing,
  pub seeker_generator: SeekerGeneratorBalancing,
  pub sniper: SniperBalancing,
  pub sniper_generator: SniperGeneratorBalancing,
  pub laser_gate: LaserGateBalancing,
}

#[derive(Deserialize)]
pub struct PlasmaBalancing {
  pub base_speed: f32,
}

#[derive(Deserialize)]
pub struct WeaponBalancing {
  pub plasma: PlasmaBalancing,
}

#[derive(Deserialize)]
pub struct ManaTankBalancing {
  pub capacity: f32,
  pub recharge_rate: f32,
}

#[derive(Deserialize)]
pub struct BoostBalancing {
  pub force_mod: f32,
  pub mana_use: f32,
  pub max_cooldown: f32,
}

#[derive(Deserialize)]
pub struct AbilityBalancing {
  pub mana_tanks: ManaTankBalancing,
  pub boost: BoostBalancing,
}

#[derive(Deserialize)]
pub struct GraphicsConfig {
  pub hitstop_enabled: bool,
  pub rounding_factor: f32,
  pub scaling_factor: f32,
}

impl GraphicsConfig {
  pub fn adjusted_scaling(&self) -> f32 {
    self.scaling_factor * 50.0
  }
}

#[derive(Deserialize)]
pub struct DebugConfig {
  pub show_colliders: bool,
}

#[derive(Deserialize)]
pub struct Balancing {
  pub player: PlayerBalancing,
  pub status_effects: StatusEffectsBalancing,
  pub enemies: EnemyBalancing,
  pub weapons: WeaponBalancing,
  pub abilities: AbilityBalancing,
  pub graphics_config: GraphicsConfig,
  pub debug: DebugConfig,
}

const BALANCING_PATH: &str = "assets/balancing.json";

pub static BALANCING: LazyLock<Balancing> = LazyLock::new(|| {
  serde_json::from_str(
    &fs::read_to_string(Path::new(&current_dir().unwrap()).join(BALANCING_PATH)).unwrap(),
  )
  .expect("JSON was not well-formatted")
});
