use crate::camera::CameraSystem;
use crate::combat::CombatSystem;
use crate::controls::ControlsSystem;
use crate::graphics::GraphicsSystem;
use crate::load_map::MapSystem;
use crate::physics::PhysicsSystem;
use crate::system::{Game, System};

mod camera;
mod combat;
mod controls;
mod entity;
mod graphics;
mod graphics_utils;
mod load_map;
mod physics;
mod system;
mod units;

#[macroquad::main("MyGame")]
async fn main() {
  Game::new()
    .add_system(CombatSystem::start)
    .add_system(MapSystem::start)
    .add_system(PhysicsSystem::start)
    .add_system(CameraSystem::start)
    .add_system(ControlsSystem::start)
    .add_system(GraphicsSystem::start)
    .run()
    .await;
}
