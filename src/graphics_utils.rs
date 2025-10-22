use macroquad::prelude::*;
use rapier2d::{na::Vector2, prelude::*};

use crate::units::{PhysicsVector, ScreenVector};

pub fn screen_bounds_adjusted() -> PhysicsVector {
  return ScreenVector::new(vector![screen_width(), screen_height()]).into_physics();
}

pub fn draw_cuboid_collider(collider: &Collider, camera_position: Vector2<f32>) {
  let translation = PhysicsVector::new(*collider.translation()).into_screen_pos(camera_position);

  match collider.shape().as_cuboid() {
    Some(cuboid) => {
      let half_extents = PhysicsVector::new(cuboid.half_extents).into_screen();

      let top_left = translation - half_extents;
      let dimensions = half_extents.scale(1.8);

      draw_rectangle(top_left.x, top_left.y, dimensions.x, dimensions.y, ORANGE);
    }
    None => {}
  }

  match collider.shape().as_ball() {
    Some(ball) => {
      draw_circle(translation.x, translation.y, 20.0, BLUE);
    }
    None => {}
  }
}
