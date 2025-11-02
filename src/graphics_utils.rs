use macroquad::prelude::*;
use rapier2d::{na::Vector2, prelude::*};

use crate::units::{PhysicsScalar, PhysicsVector, ScreenScalar, UnitConvert, UnitConvert2};

pub fn screen_bounds_adjusted() -> PhysicsVector {
  return PhysicsVector::new(
    ScreenScalar(screen_width()).convert(),
    ScreenScalar(screen_height()).convert(),
  );
}

pub fn draw_collider(collider: &Collider, camera_position: Vector2<f32>, label: Option<&str>) {
  let translation = PhysicsVector::from_vec(*collider.translation()).into_pos(camera_position);

  match collider.shape().as_cuboid() {
    Some(cuboid) => {
      let half_extents = PhysicsVector::from_vec(cuboid.half_extents).convert();

      let top_left = PhysicsVector::from_vec(translation.into_vec() - half_extents.into_vec());
      let dimensions = PhysicsVector::from_vec(half_extents.into_vec().scale(1.8));

      draw_rectangle(
        top_left.x(),
        top_left.y(),
        dimensions.x(),
        dimensions.y(),
        ORANGE,
      );

      label.map(|label| draw_text(label, top_left.x(), top_left.y(), 40.0, BLACK));
    }
    None => {}
  }

  match collider.shape().as_ball() {
    Some(ball) => {
      draw_circle(
        translation.x(),
        translation.y(),
        *PhysicsScalar(ball.radius).convert(),
        BLUE,
      );
    }
    None => {}
  }
}
