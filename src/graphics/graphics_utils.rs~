use std::f32::consts::PI;

use macroquad::prelude::*;
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  graphics::{COLOR_2, COLOR_4, VIRTUAL_SCREEN_HEIGHT, VIRTUAL_SCREEN_WIDTH},
  units::{PhysicsScalar, PhysicsVector, ScreenVector, UnitConvert, UnitConvert2},
};

pub fn draw_label(
  physics_translation: PhysicsVector,
  camera_position: Vector2<f32>,
  label: String,
  color: Option<Color>,
) {
  let screen_translation = physics_translation.into_pos(camera_position);
  draw_text(
    label.as_ref(),
    screen_translation.x(),
    screen_translation.y(),
    20.0,
    color.unwrap_or(COLOR_4),
  );
}

pub fn draw_collider(
  collider: &Collider,
  camera_position: Vector2<f32>,
  label: Option<String>,
  color: Option<Color>,
  alpha: Option<f32>,
) {
  let translation = PhysicsVector::from_vec(*collider.translation()).into_pos(camera_position);
  let rotation = collider.rotation().angle();

  let alpha = alpha.unwrap_or(if collider.is_enabled() && !collider.is_sensor() {
    1.0
  } else {
    0.5
  });

  let alpha = alpha * 0.3;

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
      draw_rectangle_ex(
        translation.x(),
        translation.y(),
        dimensions.x(),
        dimensions.y(),
        DrawRectangleParams {
          offset: Vec2 { x: 0.5, y: 0.5 },
          rotation: -rotation,
          color: color.unwrap_or(COLOR_2).with_alpha(alpha),
        },
      );

      if let Some(label) = label.as_ref() {
        draw_text(
          label.as_ref(),
          top_left.x(),
          top_left.y(),
          40.0,
          color.unwrap_or(BLACK).with_alpha(alpha),
        );
      };
    }
  }

  if let Some(ball) = collider.shape().as_ball() {
    draw_circle(
      translation.x(),
      translation.y(),
      *PhysicsScalar(ball.radius).convert(),
      color.unwrap_or(COLOR_2).with_alpha(alpha),
    );

    if let Some(label) = label.as_ref() {
      draw_text(
        label.as_ref(),
        translation.x(),
        translation.y(),
        40.0,
        color.unwrap_or(BLACK).with_alpha(alpha),
      );
    };
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
            color.unwrap_or(COLOR_4).with_alpha(alpha),
          );
        }
      }
    });
  }
}

pub fn all_offscreen(colliders: &[Collider], camera_translation: Vector2<f32>) -> bool {
  colliders
    .iter()
    .all(|collider| is_offscreen(collider, camera_translation))
}

pub fn is_offscreen(collider: &Collider, camera_translation: Vector2<f32>) -> bool {
  let shape = collider.shape();

  let half_extents = if let Some(ball) = shape.as_ball() {
    vector![ball.radius, ball.radius]
  } else if let Some(cuboid) = shape.as_cuboid() {
    let farthest_extent =
      (cuboid.half_extents.x.powf(2.0) + cuboid.half_extents.y.powf(2.0)).sqrt();
    vector![farthest_extent, farthest_extent]
  } else {
    panic!("Unhandled shape kind {:?}", shape.shape_type());
  };

  let half_extents_screen = PhysicsVector::from_vec(half_extents).convert();
  let collider_screen =
    PhysicsVector::from_vec(*collider.translation()).into_pos(camera_translation);

  let between_x = collider_screen.x() - half_extents_screen.x() > 0.0
    && collider_screen.x() + half_extents_screen.x() < VIRTUAL_SCREEN_WIDTH;

  let between_y = collider_screen.y() - half_extents_screen.y() > 0.0
    && collider_screen.y() + half_extents_screen.y() < VIRTUAL_SCREEN_HEIGHT;

  !(between_x && between_y)
}

pub fn angle_from_vec(direction: Vector2<f32>) -> f32 {
  let base_angle = direction.angle(&vector![1.0, 0.0]);

  if direction.y > 0.0 {
    2.0 * PI - base_angle
  } else {
    base_angle
  }
}
