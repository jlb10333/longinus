use std::{thread::sleep, time::Duration};

use macroquad::prelude::*;
use rapier2d::{na::Vector2, prelude::ColliderSet};

use crate::{graphics_utils::draw_cuboid_collider, units::PhysicsVector};

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

const SHOW_COLLIDERS: bool = true;

pub struct GraphicsDeps<'a> {
  pub player_translation: Vector2<f32>,
  pub camera_translation: Vector2<f32>,
  pub collider_set: &'a ColliderSet,
}

pub fn run_graphics(deps: GraphicsDeps) {
  clear_background(RED);

  if SHOW_COLLIDERS {
    deps
      .collider_set
      .iter()
      .for_each(|(_, collider)| draw_cuboid_collider(collider, deps.camera_translation));
  }

  let player_screen_pos =
    PhysicsVector::new(deps.player_translation).into_screen_pos(deps.camera_translation);

  draw_circle(player_screen_pos.x, player_screen_pos.y, 12.5, GREEN);

  let frame_time = get_frame_time();

  if frame_time < MIN_FRAME_TIME {
    let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms
    sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
  }
}
