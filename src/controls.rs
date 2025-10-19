use device_query::{DeviceQuery, DeviceState, Keycode};
use rapier2d::{na::Vector2, prelude::*};

use crate::system::System;

const INPUT_FORCE: f32 = 0.1;
const EMPTY_VECTOR: Vector2<f32> = vector![0.0, 0.0];

fn handle_movement_input(keys: &Vec<Keycode>) -> Vector2<f32> {
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

fn handle_aiming_input(keys: &Vec<Keycode>) -> f32 {
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

pub struct ControlsSystem {
  pub movement_direction: Vector2<f32>,
  pub reticle_angle_change: f32,
}

impl<'a> System<'a> for ControlsSystem {
  type Deps = ();

  fn start() -> Self
  where
    Self: Sized,
  {
    ControlsSystem {
      movement_direction: vector![0.0, 0.0],
      reticle_angle_change: 0.0,
    }
  }

  fn run(&self, _: &'a Self::Deps) -> Self
  where
    Self: Sized,
  {
    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();

    ControlsSystem {
      movement_direction: handle_movement_input(&keys),
      reticle_angle_change: handle_aiming_input(&keys),
    }
  }
}
