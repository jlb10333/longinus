use std::rc::Rc;

use balance::BALANCING;
use macroquad::prelude::*;
use system::ProcessContextOptions;

use crate::ability::AbilitySystem;
use crate::camera::CameraSystem;
use crate::combat::CombatSystem;
use crate::controls::ControlsSystem;
use crate::enemy::EnemySystem;
use crate::graphics::GraphicsSystem;
use crate::load_map::MapSystem;
use crate::menu::{MenuSystem, QuitDecision};
use crate::physics::PhysicsSystem;
use crate::save::{SaveData, SaveSystem, load_save};
use crate::system::{Process, System};

mod ability;
mod balance;
mod camera;
mod combat;
mod controls;
mod easing;
mod ecs;
mod enemy;
mod f;
mod graphics;
mod graphics_utils;
mod load_map;
mod menu;
mod physics;
mod save;
mod sprite;
mod system;
mod units;

#[derive(Clone, Default)]
pub struct Start;

pub struct ProjectileTextures {
  pub plasma: Texture2D,
}

pub struct PickupTextures {
  pub health_tank: Texture2D,
}

pub struct BlockTextures {
  pub block: Texture2D,
}

pub struct ActivatorTextures {
  pub touch_sensor_activated: Texture2D,
  pub touch_sensor_deactivated: Texture2D,
}

pub struct EnemyTextures {
  pub goblin: Texture2D,
}

pub struct GameTextures {
  pub tiles_texture: Texture2D,
  pub player_texture: Texture2D,
  pub breakable_tile_texture: Texture2D,
  pub projectile_textures: ProjectileTextures,
  pub pickup_textures: PickupTextures,
  pub block_textures: BlockTextures,
  pub activator_textures: ActivatorTextures,
  pub enemy_textures: EnemyTextures,
}

impl Default for GameTextures {
  fn default() -> Self {
    panic!()
  }
}

#[derive(Clone, Default)]
pub struct GameInput {
  pub save_data: SaveData,
  pub textures: Rc<GameTextures>,
}

enum State {
  MainMenu,
  Game(SaveData),
  Exit,
}

fn window_conf() -> Conf {
  Conf {
    window_title: "Longinus".to_string(),
    window_width: 1920,
    window_height: 1080,
    ..Default::default()
  }
}

async fn load_texture_with_filter(path: &'static str) -> Texture2D {
  let texture = load_texture(path).await.unwrap();
  texture.set_filter(FilterMode::Nearest);
  texture
}

async fn load_game_textures() -> GameTextures {
  let tiles_texture = load_texture_with_filter("./assets/maps/tilesets/tiles.png").await;
  let player_texture = load_texture_with_filter("./assets/sprites/player.png").await;
  let plasma_texture = load_texture_with_filter("./assets/sprites/projectiles/plasma.png").await;
  let health_tank_texture =
    load_texture_with_filter("./assets/sprites/pickups/health_tank.png").await;
  let breakable_tile_texture =
    load_texture_with_filter("./assets/sprites/breakable_tile.png").await;
  let block_texture = load_texture_with_filter("./assets/sprites/blocks/block.png").await;
  let touch_sensor_deactivated_texture =
    load_texture_with_filter("./assets/sprites/activators/touch_sensor_deactivated.png").await;
  let touch_sensor_activated_texture =
    load_texture_with_filter("./assets/sprites/activators/touch_sensor_activated.png").await;
  let goblin_texture = load_texture_with_filter("./assets/sprites/enemies/goblin.png").await;

  GameTextures {
    tiles_texture,
    player_texture,
    breakable_tile_texture,
    projectile_textures: ProjectileTextures {
      plasma: plasma_texture,
    },
    pickup_textures: PickupTextures {
      health_tank: health_tank_texture,
    },
    block_textures: BlockTextures {
      block: block_texture,
    },
    activator_textures: ActivatorTextures {
      touch_sensor_activated: touch_sensor_activated_texture,
      touch_sensor_deactivated: touch_sensor_deactivated_texture,
    },
    enemy_textures: EnemyTextures {
      goblin: goblin_texture,
    },
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  // Load textures async
  let textures = Rc::new(load_game_textures().await);

  let mut state = State::MainMenu;

  loop {
    state = match state {
      State::MainMenu => {
        let save_data = Process::new(&Start)
          .add_system(ControlsSystem::start)
          .add_system(SaveSystem::start)
          .add_system(MenuSystem::start)
          .add_system(GraphicsSystem::start)
          .start(None)
          .run(|ctx| {
            ctx
              .get::<MenuSystem<_>>()
              .unwrap()
              .save_to_load
              .as_ref()
              .map(load_save)
          })
          .await;
        State::Game(save_data)
      }
      State::Game(save_data) => {
        let game_input = GameInput {
          save_data,
          textures: Rc::clone(&textures),
        };

        let quit_decision = Process::new(&game_input)
          .add_system(SaveSystem::start)
          .add_system(CombatSystem::start)
          .add_system(MapSystem::start)
          .add_system(CameraSystem::start)
          .add_system(AbilitySystem::start)
          .add_system(PhysicsSystem::start)
          .add_system(ControlsSystem::start)
          .add_system(MenuSystem::start)
          .add_system(EnemySystem::start)
          .add_system(GraphicsSystem::start)
          .start(Some(ProcessContextOptions {
            should_freeze_fixed: Some(|ctx| {
              !ctx.get::<MenuSystem<_>>().unwrap().active_menus.is_empty()
                || (BALANCING.graphics_config.hitstop_enabled
                  && ctx.get::<PhysicsSystem>().unwrap().hitstop_frames_left > 0)
            }),
            ..Default::default()
          }))
          .run(|ctx| ctx.get::<MenuSystem<_>>().unwrap().quit_decision.clone())
          .await;
        match quit_decision {
          QuitDecision::LoadSave(save_to_load) => {
            State::Game(load_save(&menu::SaveToLoad::SaveData(save_to_load.clone())))
          }
          QuitDecision::ToMainMenu => State::MainMenu,
          QuitDecision::ToDesktop => State::Exit,
        }
      }
      State::Exit => break,
    };
  }
}
