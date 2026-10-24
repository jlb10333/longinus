#![feature(associated_type_defaults)]

use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::LazyLock;

use balance::BALANCING;

use macroquad::prelude::*;
use system::{ProcessContext, new_process};

use crate::ability::AbilitySystem;
use crate::camera::CameraSystem;
use crate::combat::CombatSystem;
use crate::controls::ControlsSystem;
use crate::enemy::EnemySystem;
use crate::graphics::GraphicsSystem;
use crate::load_map::MapSystem;
pub use crate::load_sprites::*;
use crate::menu::{MenuSystem, QuitDecision};
use crate::physics::PhysicsSystem;
use crate::save::{SaveData, SaveSystem, load_save};
use crate::system::System;

mod ability;
mod balance;
mod camera;
mod combat;
mod controls;
mod easing;
mod ecs;
mod effects;
mod enemy;
mod f;
mod generated;
mod graphics;
mod load_map;
mod load_sprites_utils;
mod menu;
mod physics;
mod save;
mod shaders;
mod sprite;
mod system;
mod units;

pub use generated::*;

pub const EXE_PATH: LazyLock<PathBuf> =
  LazyLock::new(|| env::current_dir().expect("Failed to read current dir"));

#[derive(Clone)]
pub struct Start {
  pub textures: Rc<SpriteTextures>,
}

impl Default for Start {
  fn default() -> Self {
    panic!()
  }
}

#[derive(Clone)]
pub struct GameInput {
  pub save_data: SaveData,
  pub textures: Rc<SpriteTextures>,
}

impl Default for GameInput {
  fn default() -> Self {
    panic!()
  }
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

#[macroquad::main(window_conf)]
async fn main() {
  // Load textures async
  let textures = Rc::new(load_game_textures().await);

  let mut state = State::MainMenu;

  loop {
    state = match state {
      State::MainMenu => {
        let save_data = new_process(
          &Start {
            textures: Rc::clone(&textures),
          },
          None,
        )
        .add_system(ControlsSystem::<Start>::start)
        .add_system(SaveSystem::<Start>::start)
        .add_system(MenuSystem::<Start>::start)
        .add_system(GraphicsSystem::<Start>::start)
        .run(
          |ctx| {
            ctx
              .get::<MenuSystem<_>, _>()
              .save_to_load
              .as_ref()
              .map(load_save)
          },
          |_| false,
        )
        .await;
        State::Game(save_data.expect("Failed to load save"))
      }
      State::Game(save_data) => {
        let game_input = GameInput {
          save_data,
          textures: Rc::clone(&textures),
        };

        let quit_decision = new_process(&game_input, None)
          .add_system(SaveSystem::<GameInput>::start)
          .add_system(CombatSystem::start)
          .add_system(MapSystem::start)
          .add_system(CameraSystem::start)
          .add_system(AbilitySystem::start)
          .add_system(PhysicsSystem::start)
          .add_system(ControlsSystem::<GameInput>::start)
          .add_system(MenuSystem::<GameInput>::start)
          .add_system(EnemySystem::start)
          .add_system(GraphicsSystem::<GameInput>::start)
          .run(
            |ctx| ctx.get::<MenuSystem<_>, _>().quit_decision.clone(),
            |ctx: Rc<ProcessContext<_, _>>| {
              !ctx.get::<MenuSystem<_>, _>().active_menus.is_empty()
                || (BALANCING.graphics_config.hitstop_enabled
                  && ctx.get::<PhysicsSystem, _>().hitstop_frames_left > 0)
            },
          )
          .await;
        match quit_decision {
          QuitDecision::LoadSave(save_to_load) => State::Game(
            load_save(&menu::SaveToLoad::SaveData(save_to_load.clone()))
              .expect("Failed to load save"),
          ),
          QuitDecision::ToMainMenu => State::MainMenu,
          QuitDecision::ToDesktop => State::Exit,
        }
      }
      State::Exit => break,
    };
  }
}
