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

pub struct GameTextures {
  pub tiles_texture: Texture2D,
  pub player_texture: Texture2D,
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

async fn load_game_textures() -> GameTextures {
  let tiles_texture = load_texture("./assets/maps/tilesets/tiles.png")
    .await
    .unwrap();
  tiles_texture.set_filter(FilterMode::Nearest);

  let player_texture = load_texture("./assets/sprites/player.png").await.unwrap();
  player_texture.set_filter(FilterMode::Nearest);

  GameTextures {
    tiles_texture,
    player_texture,
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
