use std::rc::Rc;

use device_query::{DeviceQuery, DeviceState, Keycode};
use rapier2d::{na::Vector2, prelude::*};

use crate::system::{Context, System};

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

const AIM_SPEED: f32 = 0.2;

fn reticle_angle_change(keys: &Vec<Keycode>) -> f32 {
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
  pub reticle_angle: f32,
}

impl System for ControlsSystem {
  fn start(_: Context) -> Rc<dyn System> {
    return Rc::new(Self {
      movement_direction: vector![0.0, 0.0],
      reticle_angle: 0.0,
    });
  }

  fn run(&self, _: &Context) -> Rc<dyn System> {
    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();

    return Rc::new(Self {
      movement_direction: handle_movement_input(&keys),
      reticle_angle: self.reticle_angle + reticle_angle_change(&keys),
    });
  }
}

pub struct DebugSystem;

impl System for DebugSystem {
  fn start(_: Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    return Rc::new(Self);
  }

  fn run(&self, ctx: &Context) -> Rc<dyn System> {
    let controls_system = ctx.get::<ControlsSystem>().unwrap();

    println!(
      "{}, {}",
      controls_system.movement_direction.x, controls_system.movement_direction.y
    );

    return Rc::new(Self);
  }
}
