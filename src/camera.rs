use std::rc::Rc;

use macroquad::{
  math::Rect,
  window::{screen_height, screen_width},
};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  load_map::MapSystem,
  physics::PhysicsSystem,
  system::System,
  units::{PhysicsVector, ScreenVector, UnitConvert, UnitConvert2, vec_zero},
};

const CAMERA_SCREEN_MARGIN: f32 = 0.4;
fn camera_screen_bounds() -> Rect {
  return Rect {
    x: CAMERA_SCREEN_MARGIN * screen_width(),
    y: CAMERA_SCREEN_MARGIN * screen_height(),
    w: (1.0 - (2.0 * CAMERA_SCREEN_MARGIN)) * screen_width(),
    h: (1.0 - (2.0 * CAMERA_SCREEN_MARGIN)) * screen_height(),
  };
}

fn get_camera_translation_change(player_translation: ScreenVector) -> Vector2<f32> {
  let bounds_offset_left = -1.0 * (camera_screen_bounds().x - player_translation.x()).max(0.0);
  let bounds_offset_right =
    (player_translation.x() - (camera_screen_bounds().x + camera_screen_bounds().w)).max(0.0);
  let bounds_offset_down = -1.0 * (camera_screen_bounds().y - player_translation.y()).max(0.0);
  let bounds_offset_up =
    (player_translation.y() - (camera_screen_bounds().y + camera_screen_bounds().h)).max(0.0);
  let bounds_offset_total = vector![
    bounds_offset_left + bounds_offset_right,
    bounds_offset_up + bounds_offset_down
  ];

  return if bounds_offset_total.magnitude() > 0.0 {
    bounds_offset_total
  } else {
    vector![0.0, 0.0]
  };
}

pub struct CameraSystem {
  pub translation: Vector2<f32>,
}

impl System for CameraSystem {
  fn start(ctx: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    let map_system = ctx.get::<MapSystem>().unwrap();

    return Rc::new(Self {
      translation: map_system
        .map
        .player_spawn
        .translation
        .into_pos(vec_zero())
        .into_vec()
        - vector![screen_width() / 2.0, screen_height() / 2.0],
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    let player_translation = PhysicsVector::from_vec(
      *physics_system.rigid_body_set[physics_system.player_handle].translation(),
    )
    .into_pos(self.translation);

    return Rc::new(Self {
      translation: self.translation + get_camera_translation_change(player_translation),
    });
  }
}
