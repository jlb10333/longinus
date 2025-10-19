use device_query::Keycode;
use rapier2d::{na::Vector2, prelude::*};

const INPUT_FORCE: f32 = 0.1;
const EMPTY_VECTOR: Vector2<f32> = vector![0.0, 0.0];

pub fn handle_movement_input(keys: Vec<Keycode>) -> Vector2<f32> {
  let component_vectors = [
    if keys.contains(&Keycode::Up) {
      vector![0.0, INPUT_FORCE]
    } else {
      EMPTY_VECTOR
    },
    if keys.contains(&Keycode::Down) {
      vector![0.0, -INPUT_FORCE]
    } else {
      EMPTY_VECTOR
    },
    if keys.contains(&Keycode::Left) {
      vector![-INPUT_FORCE, 0.0]
    } else {
      EMPTY_VECTOR
    },
    if keys.contains(&Keycode::Right) {
      vector![INPUT_FORCE, 0.0]
    } else {
      EMPTY_VECTOR
    },
  ];

  return component_vectors.iter().sum();
}

const AIM_SPEED: f32 = 0.1;

pub fn handle_aiming_input(keys: Vec<Keycode>) -> f32 {
  let components = [
    if keys.contains(&Keycode::A) {
      -1.0 * AIM_SPEED
    } else {
      0.0
    },
    if keys.contains(&Keycode::D) {
      AIM_SPEED
    } else {
      0.0
    },
  ];
  return components.iter().sum();
}
