use std::{thread::sleep, time::Duration};

use macroquad::prelude::*;
use rapier2d::{na::Vector2, prelude::ColliderSet};

use crate::{
  combat::{ProjectileSlots, distance_projection},
  graphics_utils::draw_cuboid_collider,
  units::{PhysicsVector, ScreenVector},
};

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

const RETICLE_SIZE: f32 = 3.0;

/* DEBUG OPTIONS */
const SHOW_COLLIDERS: bool = true;
const SHOW_SLOTS: bool = true;

pub struct GraphicsDeps<'a> {
  pub player_translation: Vector2<f32>,
  pub camera_translation: Vector2<f32>,
  pub reticle_pos: ScreenVector,
  pub collider_set: &'a ColliderSet,
  pub slot_positions: ProjectileSlots,
}

pub fn run_graphics(deps: GraphicsDeps) {
  /* Background */
  clear_background(RED);

  /* Debug */
  if SHOW_COLLIDERS {
    deps
      .collider_set
      .iter()
      .for_each(|(_, collider)| draw_cuboid_collider(collider, deps.camera_translation));
  }

  /* Draw player */
  let player_screen_pos =
    PhysicsVector::new(deps.player_translation).into_screen_pos(deps.camera_translation);

  draw_circle(player_screen_pos.x, player_screen_pos.y, 12.5, GREEN);

  /* Draw reticle */
  draw_circle(
    player_screen_pos.x + deps.reticle_pos.x,
    player_screen_pos.y + deps.reticle_pos.y,
    RETICLE_SIZE,
    BLACK,
  );

  /* DEBUG - Draw slots */
  deps.slot_positions.iter().for_each(|slot| {
    let slot_screen_pos = player_screen_pos + slot.offset.into_screen();
    let slot_next_screen_pos =
      slot_screen_pos + ScreenVector::new(distance_projection(slot.angle, 7.0));

    draw_circle(slot_screen_pos.x, slot_screen_pos.y, 2.0, BLUE);
    draw_circle(slot_next_screen_pos.x, slot_next_screen_pos.y, 2.0, WHITE);
  });

  /* Maintain target fps */
  let frame_time = get_frame_time();

  if frame_time < MIN_FRAME_TIME {
    let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms
    sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
  }
}
