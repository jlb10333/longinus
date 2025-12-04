use macroquad::prelude::*;
use rapier2d::{na::Vector2, prelude::*};

use crate::units::{PhysicsScalar, PhysicsVector, ScreenVector, UnitConvert, UnitConvert2};

pub fn draw_collider(
  collider: &Collider,
  camera_position: Vector2<f32>,
  label: Option<String>,
  color: Option<Color>,
) {
  let translation = PhysicsVector::from_vec(*collider.translation()).into_pos(camera_position);

  let alpha = if collider.is_enabled() && !collider.is_sensor() {
    1.0
  } else {
    0.5
  };

  if let Some(cuboid) = collider.shape().as_cuboid() {
    let half_extents = PhysicsVector::from_vec(cuboid.half_extents).convert();

    let top_left = ScreenVector::from_vec(translation.into_vec() - half_extents.into_vec());
    let dimensions = ScreenVector::from_vec(half_extents.into_vec().scale(2.0));

    let bottom_right = ScreenVector::from_vec(top_left.into_vec() + dimensions.into_vec());

    if (top_left.x() > 0.0
      && top_left.x() < screen_width()
      && top_left.y() > 0.0
      && top_left.y() < screen_height())
      || (bottom_right.x() > 0.0
        && bottom_right.x() < screen_width()
        && bottom_right.y() > 0.0
        && bottom_right.y() < screen_height())
    {
      draw_rectangle(
        top_left.x(),
        top_left.y(),
        dimensions.x(),
        dimensions.y(),
        color.unwrap_or(ORANGE).with_alpha(alpha),
      );

      label.map(|label| draw_text(label.as_ref(), top_left.x(), top_left.y(), 40.0, BLACK));
    }
  }

  if let Some(ball) = collider.shape().as_ball() {
    draw_circle(
      translation.x(),
      translation.y(),
      *PhysicsScalar(ball.radius).convert(),
      BLUE.with_alpha(alpha),
    );
  }

  if let Some(compound) = collider.shape().as_compound() {
    compound.shapes().iter().for_each(|(isometry, shape)| {
      if let Some(cuboid) = shape.as_cuboid() {
        let half_extents = PhysicsVector::from_vec(cuboid.half_extents).convert();

        let inner_translation = PhysicsVector::from_vec(isometry.translation.vector)
          .into_pos(camera_position)
          .into_vec();

        let top_left = inner_translation - half_extents.into_vec();

        let dimensions = half_extents.into_vec().scale(2.0);

        let bottom_right = top_left + dimensions;

        if (top_left.x > 0.0
          && top_left.x < screen_width()
          && top_left.y > 0.0
          && top_left.y < screen_height())
          || (bottom_right.x > 0.0
            && bottom_right.x < screen_width()
            && bottom_right.y > 0.0
            && bottom_right.y < screen_height())
        {
          draw_rectangle(
            top_left.x,
            top_left.y,
            dimensions.x,
            dimensions.y,
            color.unwrap_or(ORANGE).with_alpha(alpha),
          );
        }
      }
    });
  }
}
