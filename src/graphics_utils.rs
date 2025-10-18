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
      let extents = PhysicsVector::new(cuboid.half_extents.scale(1.8)).into_screen();

      draw_rectangle(
        translation.x,
        translation.y,
        extents.x, 
        extents.y,
        ORANGE
      );
    },
    None => {},
  }()
}