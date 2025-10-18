use macroquad::{math::Rect, window::{screen_height, screen_width}};
use rapier2d::{na::Vector2, prelude::*};

use crate::units::ScreenVector;

const CAMERA_SCREEN_MARGIN: f32 = 0.3;
fn camera_screen_bounds() -> Rect {
  return Rect {
    x: CAMERA_SCREEN_MARGIN * screen_width(),
    y: CAMERA_SCREEN_MARGIN * screen_height(),
    w: (1.0 - (2.0 * CAMERA_SCREEN_MARGIN)) * screen_width(),
    h: (1.0 - (2.0 * CAMERA_SCREEN_MARGIN)) * screen_height()
  }
}

pub fn camera_position(player_translation: ScreenVector) -> Vector2<f32> {
  let bounds_offset_left = -1.0 * (camera_screen_bounds().x - player_translation.x).max(0.0);
  let bounds_offset_right = (player_translation.x - (camera_screen_bounds().x + camera_screen_bounds().w)).max(0.0);
  let bounds_offset_down = -1.0 * (camera_screen_bounds().y - player_translation.y).max(0.0);
  let bounds_offset_up = (player_translation.y - (camera_screen_bounds().y + camera_screen_bounds().h)).max(0.0);
  let bounds_offset_total = vector![bounds_offset_left + bounds_offset_right, bounds_offset_up + bounds_offset_down];

  println!("{}, {}", bounds_offset_total.x, bounds_offset_total.y);

  return if bounds_offset_total.magnitude() > 0.0 { bounds_offset_total } else { vector![0.0, 0.0] }
}

// x < 