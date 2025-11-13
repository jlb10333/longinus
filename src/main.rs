use crate::camera::CameraSystem;
use crate::combat::CombatSystem;
use crate::controls::ControlsSystem;
use crate::enemy::EnemySystem;
use crate::graphics::GraphicsSystem;
use crate::menu::MenuSystem;
use crate::save::SaveSystem;
use crate::system::{Game, System};

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
  Game::new()
    .add_system(SaveSystem::start)
    .add_system(CombatSystem::start)
    .add_system(CameraSystem::start)
    .add_system(ControlsSystem::start)
    .add_system(MenuSystem::start)
    .add_system(GraphicsSystem::start)
    .add_system(EnemySystem::start)
    .run()
    .await;
}
