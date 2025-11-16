use macroquad::window::next_frame;

use crate::camera::CameraSystem;
use crate::combat::CombatSystem;
use crate::controls::ControlsSystem;
use crate::enemy::EnemySystem;
use crate::graphics::GraphicsSystem;
use crate::load_map::MapSystem;
use crate::menu::MenuSystem;
use crate::physics::PhysicsSystem;
use crate::save::{SaveData, SaveSystem};
use crate::system::{Game, GameState, System};

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

#[macroquad::main("MyGame")]
async fn main() {
  let initial_save = &Game::new(&())
    .add_system(SaveSystem::start)
    .start()
    .get::<SaveSystem<_>>()
    .unwrap()
    .loaded_save_data
    .as_ref()
    .unwrap()
    .clone();

  let mut context = Game::new(initial_save)
    .add_system(SaveSystem::start)
    .add_system(CombatSystem::start)
    .add_system(MapSystem::start)
    .add_system(PhysicsSystem::start)
    .add_system(CameraSystem::start)
    .add_system(ControlsSystem::start)
    .add_system(MenuSystem::start)
    .add_system(GraphicsSystem::start)
    .add_system(EnemySystem::start)
    .start();

  loop {
    context = Game::run(&context, |_| None::<()>).unwrap_right(); // return Context

    next_frame().await;
  }
}
