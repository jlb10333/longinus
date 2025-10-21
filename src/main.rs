use macroquad::window::next_frame;
use rapier2d::prelude::*;

use crate::camera::camera_position;
use crate::combat::{get_reticle_pos, get_slot_positions};
use crate::controls::{ControlsSystem, DebugSystem};
use crate::graphics::{GraphicsDeps, run_graphics};
use crate::load_map::{COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALL};
use crate::system::{Game, System};
use crate::units::PhysicsVector;

mod camera;
mod combat;
mod controls;
mod entity;
mod graphics;
mod graphics_utils;
mod load_map;
mod system;
mod units;

async fn game_loop() {
  Game::new()
    .add_system(ControlsSystem::start)
    .add_system(DebugSystem::start)
    .run()
    .await;
}

#[macroquad::main("MyGame")]
async fn main() {
  game_loop().await;

  let mut rigid_body_set = RigidBodySet::new();
  let mut collider_set = ColliderSet::new();

  /* Load objects from the map */
  let map_read_path = "./assets/maps/map1.json";

  let mut camera_translation = vector![0.0, 0.0];

  let map = load_map::load(map_read_path, camera_translation).unwrap();

  map.colliders.iter().for_each(|map_tile| {
    match map_tile {
      load_map::MapTile::Wall(wall) => collider_set.insert(wall.collider.clone()),
    };
  });

  /* Create the bouncing ball. */
  let rigid_body = RigidBodyBuilder::dynamic()
    .translation(vector![10.0, 10.0])
    .build();
  let ball_collider = ColliderBuilder::ball(0.25)
    .restitution(0.7)
    .collision_groups(InteractionGroups {
      memberships: COLLISION_GROUP_PLAYER,
      filter: COLLISION_GROUP_WALL,
    })
    .build();
  let ball_body_handle = rigid_body_set.insert(rigid_body);
  collider_set.insert_with_parent(ball_collider.clone(), ball_body_handle, &mut rigid_body_set);

  /* Create other structures necessary for the simulation. */
  let gravity = vector![0.0, 0.0];
  let integration_parameters = IntegrationParameters::default();
  let mut physics_pipeline = PhysicsPipeline::new();
  let mut island_manager = IslandManager::new();
  let mut broad_phase = DefaultBroadPhase::new();
  let mut narrow_phase = NarrowPhase::new();
  let mut impulse_joint_set = ImpulseJointSet::new();
  let mut multibody_joint_set = MultibodyJointSet::new();
  let mut ccd_solver = CCDSolver::new();
  let physics_hooks = ();
  let event_handler = ();

  // let mut controls_system = ControlsSystem::start();

  /* Run the game loop, stepping the simulation once per frame. */
  loop {
    // physics

    physics_pipeline.step(
      &gravity,
      &integration_parameters,
      &mut island_manager,
      &mut broad_phase,
      &mut narrow_phase,
      &mut rigid_body_set,
      &mut collider_set,
      &mut impulse_joint_set,
      &mut multibody_joint_set,
      &mut ccd_solver,
      &physics_hooks,
      &event_handler,
    );

    let player_body = &mut rigid_body_set[ball_body_handle];

    // input

    // controls_system = controls_system.run(&());

    // player_body.apply_impulse(controls_system.movement_direction, true);

    // let reticle_pos = get_reticle_pos(controls_system.reticle_angle);
    let reticle_pos = get_reticle_pos(0.0);

    // let slot_positions = get_slot_positions(controls_system.reticle_angle);
    let slot_positions = get_slot_positions(0.0);

    // camera

    camera_translation += camera_position(
      PhysicsVector::new(*player_body.translation()).into_screen_pos(camera_translation),
    );

    // graphics

    run_graphics(GraphicsDeps {
      camera_translation,
      collider_set: &collider_set,
      player_translation: *player_body.translation(),
      reticle_pos,
      slot_positions,
    });

    next_frame().await;
  }
}
