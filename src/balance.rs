use std::{env::current_dir, fs, path::Path, sync::LazyLock};

use serde::Deserialize;

use crate::ecs::StatusEffectsBalancing;

/**
 * Loads balancing data from assets/balancing.json
 *
 * Stores it in a static singleton object `balancing`
 */

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

#[derive(Deserialize)]
pub struct DefenderBalancing {
  pub aggro_range: f32,             // 20
  pub hold_force: f32,              // 0.2
  pub cooldown_initial_frames: i32, // 35
  pub ease_period: f32,             // 15
}

impl AraneaBalancing {
  pub fn stopping_force(&self) -> f32 {
    self.launch_force / self.stopping_frames as f32
  }
}

#[derive(Deserialize)]
pub struct SeekerBalancing {
  pub speed_cap: f32, // 5
  pub speed: f32,     // 0.3
}

#[derive(Deserialize)]
pub struct SeekerGeneratorBalancing {
  pub initial_force: f32,  // 5
  pub spawn_cooldown: i32, // 120
}

#[derive(Deserialize)]
pub struct EnemyBalancing {
  pub goblin: GoblinBalancing,
  pub imp: ImpBalancing,
  pub aranea: AraneaBalancing,
  pub defender: DefenderBalancing,
  pub seeker: SeekerBalancing,
  pub seeker_generator: SeekerGeneratorBalancing,
}

#[derive(Deserialize)]
pub struct Balancing {
  pub status_effects: StatusEffectsBalancing,
  pub enemies: EnemyBalancing,
}

const BALANCING_PATH: &str = "assets/balancing.json";

pub static BALANCING: LazyLock<Balancing> = LazyLock::new(|| {
  serde_json::from_str(
    &fs::read_to_string(Path::new(&current_dir().unwrap()).join(BALANCING_PATH)).unwrap(),
  )
  .expect("JSON was not well-formatted")
});
