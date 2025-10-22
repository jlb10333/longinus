use std::{rc::Rc, thread::sleep, time::Duration};

use macroquad::prelude::*;

use crate::{
  camera::CameraSystem,
  combat::{distance_projection, get_reticle_pos, get_slot_positions},
  controls::ControlsSystem,
  graphics_utils::draw_cuboid_collider,
  physics::PhysicsSystem,
  system::System,
  units::{PhysicsVector, ScreenVector},
};

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

const RETICLE_SIZE: f32 = 3.0;

/* DEBUG OPTIONS */
const SHOW_COLLIDERS: bool = true;
const SHOW_SLOTS: bool = true;

pub struct GraphicsSystem;

impl System for GraphicsSystem {
  fn start(_: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    return Rc::new(GraphicsSystem);
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let camera_system = ctx.get::<CameraSystem>().unwrap();
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();
    let controls_system = ctx.get::<ControlsSystem>().unwrap();

    /* Background */
    clear_background(RED);

    /* Debug */
    if SHOW_COLLIDERS {
      physics_system
        .collider_set
        .iter()
        .for_each(|(_, collider)| draw_cuboid_collider(collider, camera_system.translation));
    }

    /* Draw player */
    let player_screen_pos = PhysicsVector::new(
      *physics_system.rigid_body_set[physics_system.player_handle].translation(),
    )
    .into_screen_pos(camera_system.translation);

    draw_circle(player_screen_pos.x, player_screen_pos.y, 12.5, GREEN);

    /* Draw reticle */
    let reticle_pos = get_reticle_pos(controls_system.reticle_angle);

    draw_circle(
      player_screen_pos.x + reticle_pos.x,
      player_screen_pos.y + reticle_pos.y,
      RETICLE_SIZE,
      BLACK,
    );

    /* DEBUG - Draw slots */
    if SHOW_SLOTS {
      let slot_positions = get_slot_positions(controls_system.reticle_angle);
      slot_positions.iter().for_each(|(_, slot)| {
        let slot_screen_pos = player_screen_pos + slot.offset.into_screen();
        let slot_next_screen_pos =
          slot_screen_pos + ScreenVector::new(distance_projection(slot.angle, 7.0));

        draw_circle(slot_screen_pos.x, slot_screen_pos.y, 2.0, BLUE);
        draw_circle(slot_next_screen_pos.x, slot_next_screen_pos.y, 2.0, WHITE);
      });
    }

    /* Maintain target fps */
    let frame_time = get_frame_time();

    if frame_time < MIN_FRAME_TIME {
      let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms
      sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
    }

    return Rc::new(GraphicsSystem);
  }
}
