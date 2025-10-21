use std::f32::consts::PI;

use crate::units::{PhysicsVector, ScreenVector};
use rapier2d::{na::Vector2, prelude::*};

pub fn distance_projection(angle: f32, distance: f32) -> Vector2<f32> {
  return vector![angle.cos() * distance, angle.sin() * distance];
}

const RETICLE_DISTANCE_SCREEN: f32 = 20.0;

pub fn get_reticle_pos(angle: f32) -> ScreenVector {
  return ScreenVector::new(distance_projection(angle, RETICLE_DISTANCE_SCREEN));
}

pub struct Slot {
  pub offset: PhysicsVector,
  pub angle: f32,
}

pub type ProjectileSlots = [Slot; 12]; // 12

const SLOT_DISTANCE_PHYSICS: f32 = 0.2;

pub fn get_slot_positions(reticle_angle: f32) -> ProjectileSlots {
  let slot = |position_angle_offset: f32, shoot_direction_angle_offset: f32| {
    return Slot {
      offset: PhysicsVector::new(distance_projection(
        reticle_angle + position_angle_offset,
        SLOT_DISTANCE_PHYSICS,
      )),
      angle: reticle_angle + shoot_direction_angle_offset,
    };
  };

  /* FRONT */

  let front_ahead = slot(0.0, 0.0);

  let front_double_left = slot(-PI / 8.0, 0.0);
  let front_double_right = slot(PI / 8.0, 0.0);

  let front_45_left = slot(-PI / 4.0, -PI / 4.0);
  let front_45_right = slot(PI / 4.0, PI / 4.0);

  /* SIDE */

  let side_left = slot(-PI / 2.0, -PI / 2.0);
  let side_right = slot(PI / 2.0, PI / 2.0);

  /* BACK */

  let back_ahead = slot(PI, PI);

  let back_double_left = slot(PI - PI / 8.0, PI);
  let back_double_right = slot(PI + PI / 8.0, PI);

  let back_45_left = slot(PI - PI / 4.0, PI - PI / 4.0);
  let back_45_right = slot(PI + PI / 4.0, PI + PI / 4.0);

  return [
    front_ahead,
    front_double_left,
    front_double_right,
    front_45_left,
    front_45_right,
    side_left,
    side_right,
    back_ahead,
    back_double_left,
    back_double_right,
    back_45_left,
    back_45_right,
  ];

  /*  */
}

const FRONT_AHEAD: i32 = 0;
const FRONT_DOUBLE_LEFT: i32 = 1;
const FRONT_DOUBLE_RIGHT: i32 = 2;
const FRONT_45_LEFT: i32 = 3;
const FRONT_45_RIGHT: i32 = 4;
const SIDE_LEFT: i32 = 5;
const SIDE_RIGHT: i32 = 6;
const BACK_AHEAD: i32 = 7;
const BACK_DOUBLE_LEFT: i32 = 8;
const BACK_DOUBLE_RIGHT: i32 = 9;
const BACK_45_LEFT: i32 = 10;
const BACK_45_RIGHT: i32 = 11;

struct Projectile {
  collider: Collider,
  initial_force: PhysicsVector,
  damage: f32,
}

enum ProjectileType {
  Plasma,
  Missle,
  Laser,
}

struct Weapon {
  projectile_type: ProjectileType,
  damage_mod: f32,
  velocity_mod: f32,
}
