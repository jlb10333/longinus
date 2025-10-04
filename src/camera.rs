use macroquad::{math::Rect, window::{screen_height, screen_width}};
use rapier2d::prelude::*;

use crate::graphics_utils::ScreenVector;

const CAMERA_SCREEN_BOUND_RATIO: f32 = 0.8;
fn camera_screen_bounds() -> Rect {
  return Rect {
    x: (1.0-CAMERA_SCREEN_BOUND_RATIO) * screen_width(),
    y: CAMERA_SCREEN_BOUND_RATIO * screen_height(),
    w: CAMERA_SCREEN_BOUND_RATIO * screen_width(),
    h: (1.0-CAMERA_SCREEN_BOUND_RATIO) * screen_height()
  }
}

const CAMERA_MOVE_SPEED: f32 = 0.1;

pub fn camera_position(camera_translation: ScreenVector, player_translation: ScreenVector) -> ScreenVector {
  let bounds_offset_left = (camera_screen_bounds().x - player_translation.0.x).min(0.0);
  let bounds_offset_right = (player_translation.0.x - (camera_screen_bounds().x + camera_screen_bounds().w)).max(0.0);
  let bounds_offset_down = (camera_screen_bounds().y - player_translation.0.y).min(0.0);
  let bounds_offset_up = (player_translation.0.y - (camera_screen_bounds().y + camera_screen_bounds().h)).max(0.0);
  let bounds_offset_total = vector![bounds_offset_left + bounds_offset_right, bounds_offset_up + bounds_offset_down];

  return camera_translation + ScreenVector(bounds_offset_total.normalize() * CAMERA_MOVE_SPEED);
}