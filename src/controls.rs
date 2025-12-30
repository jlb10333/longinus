use std::{cell::RefCell, f32::consts::PI, marker::PhantomData, rc::Rc};

use gilrs::{Axis, Button, ConnectedGamepadsIterator, Gamepad, Gilrs};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  system::{ProcessContext, System},
  units::{PhysicsVector, UnitConvert, UnitConvert2, vec_zero},
};

const INPUT_FORCE: f32 = 0.1;

#[derive(Clone)]
pub struct ControlsSystem<Input> {
  pub left_stick: PhysicsVector,
  pub right_stick: PhysicsVector,
  pub menu_up: bool,
  pub menu_down: bool,
  pub menu_left: bool,
  pub menu_right: bool,
  pub menu_confirm: bool,
  pub menu_cancel: bool,
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

      let base_vec = vector![horizontal_axis_value, vertical_axis_value];

      if base_vec == vec_zero() {
        base_vec
      } else {
        base_vec.normalize() * INPUT_FORCE
      }
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
      left_stick: PhysicsVector::zero(),
      right_stick: PhysicsVector::zero(),
      boost: false,
      chain: false,
      firing: false,
      inventory: false,
      menu_down: false,
      menu_left: false,
      menu_right: false,
      menu_up: false,
      menu_confirm: false,
      menu_cancel: false,
      pause: false,
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
      menu_up: handle_button_input(&gilrs, Button::DPadUp),
      menu_down: handle_button_input(&gilrs, Button::DPadDown),
      menu_left: handle_button_input(&gilrs, Button::DPadLeft),
      menu_right: handle_button_input(&gilrs, Button::DPadRight),
      firing: handle_button_input(&gilrs, Button::RightTrigger2),
      inventory: handle_button_input(&gilrs, Button::West),
      pause: handle_button_input(&gilrs, Button::North),
      boost: handle_button_input(&gilrs, Button::LeftTrigger2),
      chain: handle_button_input(&gilrs, Button::LeftTrigger),
      menu_cancel: handle_button_input(&gilrs, Button::East),
      menu_confirm: handle_button_input(&gilrs, Button::South),
      gilrs: Rc::clone(&self.gilrs),
      last_frame: Some(Rc::new(self.clone())),
      phantom: PhantomData,
    })
  }
}
