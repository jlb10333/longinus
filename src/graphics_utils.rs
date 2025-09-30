use macroquad::prelude::*;
use rapier2d::prelude::*;

pub fn draw_tile(collider: Collider) {
  let translation = collider.translation().scale(50.0);
  match collider.shape().as_cuboid() {
      Some(cuboid) => {
        let extents = cuboid.half_extents.scale(100.0);
        draw_rectangle(
          translation.x,
          screen_height() - translation.y,
          translation.x + extents.x, 
          screen_height() - translation.y + extents.y,
          ORANGE
        );
      },
      None => {},
  }()
}