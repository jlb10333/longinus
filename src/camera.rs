use std::rc::Rc;

use macroquad::{
  math::Rect,
  window::{screen_height, screen_width},
};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  load_map::MapSystem,
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, ScreenVector, UnitConvert2, vec_zero},
};

const CAMERA_SCREEN_MARGIN: f32 = 0.4;
fn camera_screen_bounds() -> Rect {
  Rect {
    x: CAMERA_SCREEN_MARGIN * screen_width(),
    y: CAMERA_SCREEN_MARGIN * screen_height(),
    w: (1.0 - (2.0 * CAMERA_SCREEN_MARGIN)) * screen_width(),
    h: (1.0 - (2.0 * CAMERA_SCREEN_MARGIN)) * screen_height(),
  }
}

fn get_camera_translation_change(player_translation: ScreenVector) -> Vector2<f32> {
  let bounds_offset_left = -(camera_screen_bounds().x - player_translation.x()).max(0.0);
  let bounds_offset_right =
    (player_translation.x() - (camera_screen_bounds().x + camera_screen_bounds().w)).max(0.0);
  let bounds_offset_down = -(camera_screen_bounds().y - player_translation.y()).max(0.0);
  let bounds_offset_up =
    (player_translation.y() - (camera_screen_bounds().y + camera_screen_bounds().h)).max(0.0);
  let bounds_offset_total = vector![
    bounds_offset_left + bounds_offset_right,
    bounds_offset_up + bounds_offset_down
  ];

  if bounds_offset_total.magnitude() > 0.0 {
    bounds_offset_total
  } else {
    vector![0.0, 0.0]
  }
}

pub struct CameraSystem {
  pub translation: Vector2<f32>,
  pub map_top_left: Vector2<f32>,
  pub map_bottom_right: Vector2<f32>,
}

impl System for CameraSystem {
  type Input = SaveData;
  fn start(ctx: &crate::system::ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    let map_system = ctx.get::<MapSystem>().unwrap();
    let map = map_system.map.as_ref().unwrap();

    return Rc::new(Self {
      translation: map
        .player_spawns
        .iter()
        .find(|player_spawn| player_spawn.id == map_system.target_player_spawn_id)
        .unwrap()
        .translation
        .into_pos(vec_zero())
        .into_vec()
        - vector![screen_width() / 2.0, screen_height() / 2.0],
      map_top_left: map.top_left,
      map_bottom_right: map.bottom_right,
    });
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let map_system = ctx.get::<MapSystem>().unwrap();

    if let Some(map) = map_system.map.as_ref() {
      return Rc::new(Self {
        translation: map
          .player_spawns
          .iter()
          .find(|player_spawn| player_spawn.id == map_system.target_player_spawn_id)
          .unwrap()
          .translation
          .into_pos(vec_zero())
          .into_vec()
          - vector![screen_width() / 2.0, screen_height() / 2.0],
        map_top_left: map.top_left,
        map_bottom_right: map.bottom_right,
      });
    }

    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    let player_translation = PhysicsVector::from_vec(
      *physics_system.rigid_body_set[physics_system.player_handle].translation(),
    )
    .into_pos(self.translation);

    let attempted_translation =
      self.translation + get_camera_translation_change(player_translation);

    let map_top_left = PhysicsVector::from_vec(self.map_top_left).into_pos(attempted_translation);
    let map_bottom_right =
      PhysicsVector::from_vec(self.map_bottom_right).into_pos(attempted_translation);

    let map_bounds_offset_left = map_top_left.x().max(0.0);
    let map_bounds_offset_right = (map_bottom_right.x() - screen_width()).min(0.0);
    let map_bounds_offset_top = map_top_left.y().max(0.0);
    let map_bounds_offset_bottom = (map_bottom_right.y() - screen_height()).min(0.0);

    let map_bounds_offset = vector![
      if map_bottom_right.x() - map_top_left.x() < screen_width() {
        0.0
      } else {
        map_bounds_offset_left + map_bounds_offset_right
      },
      if map_bottom_right.y() - map_top_left.y() < screen_height() {
        0.0
      } else {
        map_bounds_offset_top + map_bounds_offset_bottom
      },
    ];

    return Rc::new(Self {
      translation: self.translation
        + get_camera_translation_change(player_translation)
        + map_bounds_offset,
      map_top_left: self.map_top_left,
      map_bottom_right: self.map_bottom_right,
    });
  }
}
