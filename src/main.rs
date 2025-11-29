use std::rc::Rc;

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
mod camera;
mod combat;
mod controls;
mod ecs;
mod enemy;
mod f;
mod graphics;
mod graphics_utils;
mod load_map;
mod menu;
mod physics;
mod save;
mod system;
mod units;

#[derive(Clone, Default)]
pub struct Start;

enum State {
  MainMenu,
  Game(SaveData),
  Exit,
}

#[macroquad::main("MyGame")]
async fn main() {
  let mut state = State::MainMenu;

  loop {
    state = match state {
      State::MainMenu => {
        let save_data = Process::new(&Start)
          .add_system(ControlsSystem::start)
          .add_system(SaveSystem::start)
          .add_system(MenuSystem::start)
          .add_system(GraphicsSystem::start)
          .start()
          .run_move(|ctx| {
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
        let quit_decision = &Rc::new(
          Process::new(&save_data)
            .add_system(SaveSystem::start)
            .add_system(CombatSystem::start)
            .add_system(MapSystem::start)
            .add_system(PhysicsSystem::start)
            .add_system(CameraSystem::start)
            .add_system(ControlsSystem::start)
            .add_system(MenuSystem::start)
            .add_system(EnemySystem::start)
            .add_system(AbilitySystem::start)
            .add_system(GraphicsSystem::start)
            .start(),
        )
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
