use std::{cell::RefCell, f32::consts::PI, marker::PhantomData, rc::Rc};

use gilrs::{Axis, Button, ConnectedGamepadsIterator, Gamepad, Gilrs};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  system::{ProcessContext, System},
  units::{PhysicsVector, UnitConvert, UnitConvert2},
};

const INPUT_FORCE: f32 = 0.1;

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
  pub gilrs: Rc<RefCell<Gilrs>>,
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

fn handle_stick_input(gilrs: &Gilrs, bindings: StickBindings) -> PhysicsVector {
  let input_vectors = gilrs
    .gamepads()
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

  if input_vectors.is_empty() {
    PhysicsVector::zero()
  } else {
    PhysicsVector::from_vec(input_vectors.iter().sum::<Vector2<f32>>() / input_vectors.len() as f32)
  }
}

fn handle_button_input(gilrs: &Gilrs, button: Button) -> bool {
  gilrs.gamepads().any(|(_, gamepad)| {
    gamepad
      .button_data(button)
      .map(|button_data| button_data.is_pressed())
      .unwrap_or(false)
  })
}

impl<Input: Clone + 'static> System for ControlsSystem<Input> {
  type Input = Input;

  fn start(_: &ProcessContext<Input>) -> Rc<dyn System<Input = Self::Input>> {
    let gilrs = Gilrs::new().unwrap();

    Rc::new(Self {
      left_stick: handle_stick_input(
        &gilrs,
        StickBindings {
          vertical: Axis::LeftStickX,
          horizontal: Axis::LeftStickY,
        },
      ),
      right_stick: handle_stick_input(
        &gilrs,
        StickBindings {
          vertical: Axis::RightStickX,
          horizontal: Axis::RightStickY,
        },
      ),
      firing: handle_button_input(&gilrs, Button::RightTrigger2),
      inventory: handle_button_input(&gilrs, Button::Select),
      pause: handle_button_input(&gilrs, Button::Start),
      boost: handle_button_input(&gilrs, Button::LeftTrigger2),
      chain: handle_button_input(&gilrs, Button::LeftTrigger),
      gilrs: Rc::new(RefCell::new(gilrs)),
      last_frame: None,
      phantom: PhantomData,
    })
  }

  fn run(&self, _: &ProcessContext<Input>) -> Rc<dyn System<Input = Self::Input>> {
    let mut gilrs = self.gilrs.as_ref().borrow_mut();

    while gilrs.next_event().is_some() {}

    Rc::new(Self {
      left_stick: handle_stick_input(
        &gilrs,
        StickBindings {
          vertical: Axis::LeftStickY,
          horizontal: Axis::LeftStickX,
        },
      ),
      right_stick: handle_stick_input(
        &gilrs,
        StickBindings {
          vertical: Axis::RightStickY,
          horizontal: Axis::RightStickX,
        },
      ),
      firing: handle_button_input(&gilrs, Button::RightTrigger2),
      inventory: handle_button_input(&gilrs, Button::Select),
      pause: handle_button_input(&gilrs, Button::Start),
      boost: handle_button_input(&gilrs, Button::LeftTrigger2),
      chain: handle_button_input(&gilrs, Button::LeftTrigger),
      gilrs: Rc::clone(&self.gilrs),
      last_frame: Some(Rc::new(self.clone())),
      phantom: PhantomData,
    })
  }
}
