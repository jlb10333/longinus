use std::{cell::RefCell, f32::consts::PI, marker::PhantomData, rc::Rc};

use gilrs::{Axis, Button, Gilrs};
use macroquad::input::{KeyCode, MouseButton, is_key_down, is_mouse_button_down, mouse_position};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  camera::CameraSystem,
  physics::PhysicsSystem,
  save::SaveData,
  system::{ProcessContext, System},
  units::{PhysicsVector, UnitConvert, UnitConvert2, vec_zero},
};

const INPUT_FORCE: f32 = 0.1;

#[derive(Clone, Copy)]
pub enum ControlMode {
  GamePad,
  Keyboard,
}

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
  pub map: bool,
  pub boost: bool,
  pub chain: bool,
  pub last_frame: Option<Rc<ControlsSystem<Input>>>,
  pub gilrs: Rc<RefCell<Gilrs>>,
  pub control_mode: ControlMode,
  pub phantom: PhantomData<Input>,
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

const CONTROLLER_DEADZONE: f32 = 0.2;

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

      if base_vec.magnitude() < CONTROLLER_DEADZONE {
        vec_zero()
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
      map: false,
      gilrs: Rc::new(RefCell::new(gilrs)),
      last_frame: None,
      control_mode: ControlMode::Keyboard,
      phantom: PhantomData,
    })
  }

  fn run(&self, ctx: &ProcessContext<Input>) -> Rc<dyn System<Input = Self::Input>> {
    let kbd_w_pressed = is_key_down(KeyCode::W);
    let kbd_a_pressed = is_key_down(KeyCode::A);
    let kbd_s_pressed = is_key_down(KeyCode::S);
    let kbd_d_pressed = is_key_down(KeyCode::D);

    let kbd_e_pressed = is_key_down(KeyCode::E);
    let kbd_esc_pressed = is_key_down(KeyCode::Escape);
    let kbd_tab_pressed = is_key_down(KeyCode::Tab);
    let kbd_ctl_pressed = is_key_down(KeyCode::LeftControl);
    let kbd_c_pressed = is_key_down(KeyCode::C);

    let lmb_pressed = is_mouse_button_down(MouseButton::Left);
    let rmb_pressed = is_mouse_button_down(MouseButton::Right);

    let incoming_kbd_mouse_input = kbd_w_pressed
      | kbd_a_pressed
      | kbd_s_pressed
      | kbd_d_pressed
      | kbd_e_pressed
      | kbd_esc_pressed
      | kbd_tab_pressed
      | kbd_ctl_pressed
      | kbd_c_pressed
      | lmb_pressed
      | rmb_pressed;

    let mut gilrs = self.gilrs.as_ref().borrow_mut();

    let mut incoming_gamepad_input = false;

    while let Some(event) = gilrs.next_event() {
      if matches!(event.event, gilrs::EventType::ButtonChanged(_, _, _)) {
        incoming_gamepad_input = true
      }
    }

    let control_mode = if incoming_kbd_mouse_input && !incoming_gamepad_input {
      ControlMode::Keyboard
    } else if incoming_gamepad_input && !incoming_kbd_mouse_input {
      ControlMode::GamePad
    } else {
      self.control_mode
    };

    Rc::new(match control_mode {
      ControlMode::GamePad => Self {
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
        pause: handle_button_input(&gilrs, Button::Select),
        map: handle_button_input(&gilrs, Button::North),
        boost: handle_button_input(&gilrs, Button::LeftTrigger2),
        chain: handle_button_input(&gilrs, Button::LeftTrigger),
        menu_cancel: handle_button_input(&gilrs, Button::East),
        menu_confirm: handle_button_input(&gilrs, Button::South),
        gilrs: Rc::clone(&self.gilrs),
        control_mode,
        last_frame: Some(Rc::new(self.clone())),
        phantom: PhantomData,
      },
      ControlMode::Keyboard => {
        let left_stick_denormalized = vector![
          if kbd_a_pressed { -1.0 } else { 0.0 } + if kbd_d_pressed { 1.0 } else { 0.0 },
          if kbd_w_pressed { 1.0 } else { 0.0 } + if kbd_s_pressed { -1.0 } else { 0.0 }
        ];

        let right_stick_denormalized = if let Some(ctx) = ctx.downcast::<SaveData>() {
          let physics_system = ctx.get::<PhysicsSystem>().unwrap();
          let camera_system = ctx.get::<CameraSystem>().unwrap();

          let mouse_pos = mouse_position();

          let player_screen_position = PhysicsVector::from_vec(
            *physics_system.rigid_body_set[physics_system.player_handle].translation(),
          )
          .into_pos(camera_system.translation);

          let base_stick = vector![mouse_pos.0, mouse_pos.1] - player_screen_position.into_vec();
          vector![base_stick[0], -base_stick[1]]
        } else {
          vec_zero()
        };

        Self {
          left_stick: PhysicsVector::from_vec(
            if left_stick_denormalized == vec_zero() {
              vec_zero()
            } else {
              left_stick_denormalized.normalize()
            } * INPUT_FORCE,
          ),
          right_stick: PhysicsVector::from_vec(if right_stick_denormalized == vec_zero() {
            vec_zero()
          } else {
            right_stick_denormalized.normalize()
          }),
          menu_up: kbd_w_pressed,
          menu_down: kbd_s_pressed,
          menu_left: kbd_a_pressed,
          menu_right: kbd_d_pressed,
          firing: lmb_pressed,
          inventory: kbd_e_pressed,
          pause: kbd_esc_pressed,
          map: kbd_tab_pressed,
          boost: kbd_ctl_pressed,
          chain: kbd_c_pressed,
          menu_cancel: rmb_pressed,
          menu_confirm: lmb_pressed,
          gilrs: Rc::clone(&self.gilrs),
          control_mode,
          last_frame: Some(Rc::new(self.clone())),
          phantom: PhantomData,
        }
      }
    })
  }
}
