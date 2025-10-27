use std::{f32::consts::PI, rc::Rc};

use device_query::{DeviceQuery, DeviceState, Keycode};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  system::{Context, System},
  units::{PhysicsVector, UnitConvert, UnitConvert2},
};

const INPUT_FORCE: f32 = 0.1;
const EMPTY_VECTOR: Vector2<f32> = vector![0.0, 0.0];

struct KeyBindings {
  up: Keycode,
  down: Keycode,
  left: Keycode,
  right: Keycode,
}

fn handle_stick_input(keys: &Vec<Keycode>, bindings: KeyBindings) -> PhysicsVector {
  let component_vectors = [
    if keys.contains(&bindings.up) {
      vector![0.0, INPUT_FORCE]
    } else {
      EMPTY_VECTOR
    },
    if keys.contains(&bindings.down) {
      vector![0.0, -INPUT_FORCE]
    } else {
      EMPTY_VECTOR
    },
    if keys.contains(&bindings.left) {
      vector![-INPUT_FORCE, 0.0]
    } else {
      EMPTY_VECTOR
    },
    if keys.contains(&bindings.right) {
      vector![INPUT_FORCE, 0.0]
    } else {
      EMPTY_VECTOR
    },
  ];

  return PhysicsVector::from_vec(component_vectors.iter().sum());
}

#[derive(Clone)]
pub struct ControlsSystem {
  pub left_stick: PhysicsVector,
  pub right_stick: PhysicsVector,
  pub firing: bool,
  pub inventory: bool,
  pub pause: bool,
  pub last_frame: Option<Rc<ControlsSystem>>,
}

pub fn angle_from_vec(direction: PhysicsVector) -> f32 {
  let base_angle = direction.into_vec().angle(&vector![1.0, 0.0]);

  if direction.y() > 0.0 {
    2.0 * PI - base_angle
  } else {
    base_angle
  }
}

impl System for ControlsSystem {
  fn start(_: Context) -> Rc<dyn System> {
    return Rc::new(Self {
      left_stick: PhysicsVector::zero(),
      right_stick: PhysicsVector::zero(),
      firing: false,
      inventory: false,
      pause: false,
      last_frame: None,
    });
  }

  fn run(&self, _: &Context) -> Rc<dyn System> {
    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();

    return Rc::new(Self {
      left_stick: handle_stick_input(
        &keys,
        KeyBindings {
          up: Keycode::Up,
          down: Keycode::Down,
          left: Keycode::Left,
          right: Keycode::Right,
        },
      ),
      right_stick: handle_stick_input(
        &keys,
        KeyBindings {
          up: Keycode::W,
          down: Keycode::S,
          left: Keycode::A,
          right: Keycode::D,
        },
      ),
      firing: keys.contains(&Keycode::Space),
      inventory: keys.contains(&Keycode::E),
      pause: keys.contains(&Keycode::Enter),
      last_frame: Some(Rc::new(self.clone())),
    });
  }
}
