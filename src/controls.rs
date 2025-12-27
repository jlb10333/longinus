use std::{f32::consts::PI, marker::PhantomData, rc::Rc};

use gilrs::{Axis, Button, ConnectedGamepadsIterator, Gamepad, Gilrs};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  system::{ProcessContext, System},
  units::{PhysicsVector, UnitConvert2},
};

const INPUT_FORCE: f32 = 0.1;
const EMPTY_VECTOR: Vector2<f32> = vector![0.0, 0.0];

#[derive(Clone)]
pub struct ControlsSystem<Input> {
  pub left_stick: PhysicsVector,
  pub right_stick: PhysicsVector,
  pub firing: bool,
  pub inventory: bool,
  pub pause: bool,
  pub boost: bool,
  pub chain: bool,
  pub last_frame: Option<Rc<ControlsSystem<Input>>>,
  pub gilrs: Rc<Gilrs>,
  phantom: PhantomData<Input>,
}

pub fn angle_from_vec(direction: PhysicsVector) -> f32 {
  let base_angle = direction.into_vec().angle(&vector![1.0, 0.0]);

  if direction.y() > 0.0 {
    2.0 * PI - base_angle
  } else {
    base_angle
  }
}

struct StickBindings {
  vertical: Axis,
  horizontal: Axis,
}

fn handle_stick_input(
  gamepads: ConnectedGamepadsIterator,
  bindings: StickBindings,
) -> PhysicsVector {
  let input_vectors = gamepads
    .map(|(_, gamepad)| {
      let horizontal_axis_value = gamepad
        .axis_data(bindings.horizontal)
        .map(|axis_data| axis_data.value())
        .unwrap_or(0.0);
      let vertical_axis_value = gamepad
        .axis_data(bindings.vertical)
        .map(|axis_data| axis_data.value())
        .unwrap_or(0.0);

      vector![horizontal_axis_value, vertical_axis_value].normalize()
    })
    .collect::<Vec<_>>();

  PhysicsVector::from_vec(input_vectors.iter().sum() / input_vectors.len())
}

fn handle_button_input(gamepads: ConnectedGamepadsIterator, button: Button) -> bool {
  gamepads.any(|(_, gamepad)| {
    gamepad
      .button_data(button)
      .map(|button_data| button_data.is_pressed())
      .unwrap_or(false)
  })
}

impl<Input: Clone + 'static> System for ControlsSystem<Input> {
  type Input = Input;

  fn start(_: &ProcessContext<Input>) -> Rc<dyn System<Input = Self::Input>> {
    let gilrs = Rc::new(Gilrs::new().unwrap());

    Rc::new(Self {
      left_stick: handle_stick_input(
        gilrs.gamepads(),
        StickBindings {
          vertical: Axis::LeftStickX,
          horizontal: Axis::LeftStickY,
        },
      ),
      right_stick: handle_stick_input(
        gilrs.gamepads(),
        StickBindings {
          vertical: Axis::RightStickX,
          horizontal: Axis::RightStickY,
        },
      ),
      firing: keys.contains(&Keycode::Space),
      inventory: keys.contains(&Keycode::E),
      pause: keys.contains(&Keycode::Enter),
      boost: keys.contains(&Keycode::LControl),
      chain: keys.contains(&Keycode::C),
      last_frame: None,
      phantom: PhantomData,
    })
  }

  fn run(&self, _: &ProcessContext<Input>) -> Rc<dyn System<Input = Self::Input>> {
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
      chain: keys.contains(&Keycode::C),
      boost: keys.contains(&Keycode::LControl),
      last_frame: Some(Rc::new(self.clone())),
      phantom: PhantomData,
    });
  }
}
