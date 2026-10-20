use std::{
  any::Any,
  marker::PhantomData,
  rc::Rc,
  thread::sleep,
  time::{Duration, Instant},
};

use frunk::{HCons, HList, hlist, hlist::*};
use macroquad::window::next_frame;

pub trait System: Any {
  type Input: Clone + 'static;
  type Deps: HList;

  fn start<ProcessSystems: Clone + HList + Sculptor<Self::Deps, HNil> + 'static>(
    _: Rc<ProcessContext<Self::Input, ProcessSystems>>,
  ) -> Rc<Self>
  where
    Self: Sized;

  fn update<ProcessSystems: Clone + HList + Sculptor<Self::Deps, HNil> + 'static>(
    self: Rc<Self>,
    _: Rc<ProcessContext<Self::Input, ProcessSystems>>,
  ) -> Rc<Self>;

  fn fixed_update<ProcessSystems: Clone + HList + Sculptor<Self::Deps, HNil> + 'static>(
    self: Rc<Self>,
    _: Rc<ProcessContext<Self::Input, ProcessSystems>>,
  ) -> Rc<Self>;
}

#[derive(Clone)]
pub struct ProcessContextOptions {
  /// Interval to target for fixed_update calls, in seconds
  pub fixed_time_interval: f32,
  pub target_fps: i32,
}

impl Default for ProcessContextOptions {
  fn default() -> Self {
    Self {
      fixed_time_interval: 1.0 / 60.0,
      target_fps: 60,
    }
  }
}

trait RunSystemHList<Input: Clone + 'static, Systems: HList>: Sized {
  fn update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self;

  fn fixed_update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self;
}

impl<Input: Clone + 'static, Systems: HList> RunSystemHList<Input, Systems> for HCons<(), HNil> {
  fn update(self, _: Rc<ProcessContext<Input, Systems>>) -> Self {
    self
  }

  fn fixed_update(self, _: Rc<ProcessContext<Input, Systems>>) -> Self {
    self
  }
}

impl<
  Input: Clone + 'static,
  Deps: HList,
  Systems: Clone + HList + Sculptor<Deps, HNil> + 'static,
  H: System<Input = Input, Deps = Deps>,
  T: HList + RunSystemHList<Input, Systems>,
> RunSystemHList<Input, Systems> for HCons<Rc<H>, T>
{
  fn update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self {
    let (h, t) = self.pop();

    let new_h = h.update(ctx.clone());

    let new_t = t.update(ctx);

    h_cons(new_h, new_t)
  }

  fn fixed_update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self {
    let (h, t) = self.pop();

    let new_h = h.fixed_update(ctx.clone());

    let new_t = t.fixed_update(ctx);

    h_cons(new_h, new_t)
  }
}

pub struct ProcessContext<Input: Clone + 'static, Systems: HList> {
  pub systems: Systems,
  pub input: Input,
  options: ProcessContextOptions,
}

impl<Input: Clone + 'static, Systems: HList + Clone> Clone for ProcessContext<Input, Systems> {
  fn clone(&self) -> Self {
    Self {
      systems: self.systems.clone(),
      input: self.input.clone(),
      options: self.options.clone(),
    }
  }
}

impl<Input: Clone + 'static, Systems: HList + 'static> ProcessContext<Input, Systems> {
  pub fn downcast<Target: Clone + 'static>(&self) -> Option<&ProcessContext<Target, Systems>> {
    (self as &dyn Any).downcast_ref::<ProcessContext<Target, Systems>>()
  }

  pub fn get_deps<'a, Deps: HList>(self: Rc<Self>) -> Deps
  where
    Systems: Sculptor<Deps, HNil> + Clone,
  {
    self.systems.clone().sculpt().0
  }
}

impl<Input: Clone + 'static, Systems: HList + RunSystemHList<Input, Systems> + Clone + 'static>
  ProcessContext<Input, Systems>
{
  fn with_systems(&self, new_systems: Systems) -> Self {
    Self {
      systems: new_systems,
      input: self.input.clone(),
      options: self.options.clone(),
    }
  }

  pub async fn run<'a, Output, Terminator, ShouldFreeze>(
    self,
    terminator: Terminator,
    should_freeze_fixed: Option<ShouldFreeze>,
  ) -> Output
  where
    Terminator: Fn(&ProcessContext<Input, Systems>) -> Option<Output>,
    ShouldFreeze: Fn(&ProcessContext<Input, Systems>) -> bool,
  {
    let mut game_state = Rc::new(self);
    let mut acc_time = 0.0;

    loop {
      let result = terminator(&game_state);

      if let Some(output) = result {
        return output;
      }

      let now = Instant::now();

      let new_systems = game_state.systems.clone().update(game_state.clone());
      game_state = game_state.with_systems(new_systems.into()).into();

      next_frame().await;

      let skipping_fixed = should_freeze_fixed
        .as_ref()
        .map(|should_freeze_fixed| should_freeze_fixed(&game_state))
        .unwrap_or(false);

      if !skipping_fixed {
        let fixed_update_count =
          (acc_time / (game_state.options.fixed_time_interval * 1_000_000.0)).floor() as i32;

        acc_time -=
          game_state.options.fixed_time_interval * 1_000_000.0 * fixed_update_count as f32;

        let new_systems = game_state.systems.clone().fixed_update(game_state.clone());
        game_state = game_state.with_systems(new_systems.into()).into();
      }

      let frame_time = now.elapsed().as_millis() as f32;

      let min_frame_time = 1000.0 / game_state.options.target_fps as f32;

      if frame_time < min_frame_time {
        let time_to_sleep = min_frame_time - frame_time;

        sleep(Duration::from_millis(time_to_sleep as u64));
      }

      if !skipping_fixed {
        acc_time += now.elapsed().as_micros() as f32;
      }
    }
  }

  pub fn add_system<Deps: HList, NewSystem: System<Input = Input, Deps = Deps>>(
    self: Rc<Self>,
    new_system_initializer: fn(Rc<Self>) -> Rc<NewSystem>,
  ) -> Rc<ProcessContext<Input, HCons<NewSystem, Systems>>>
  where
    Systems: Sculptor<Deps, HNil>,
  {
    let new_system = new_system_initializer(self.clone());

    ProcessContext::<Input, HCons<NewSystem, Systems>> {
      input: self.input.clone(),
      systems: h_cons(new_system, self.systems.clone()),
      options: self.options.clone(),
    }
    .into()
  }
}

pub fn new_process<Input: Clone + 'static>(
  input: &Input,
  options: Option<ProcessContextOptions>,
) -> Rc<ProcessContext<Input, HCons<(), HNil>>> {
  ProcessContext::<Input, HCons<(), HNil>> {
    input: input.clone(),
    systems: h_cons((), HNil),
    options: options.unwrap_or_default(),
  }
  .into()
}

// #[derive(Clone)]
// pub struct EmptySystem<Input>(PhantomData<Input>);

// impl<Input: Clone + 'static> System for EmptySystem<Input> {
//   type Deps = HNil;
//   type Input = Input;

//   fn start<ProcessSystems: Clone + HList + Sculptor<Self::Deps, i32> + 'static>(
//     _: Rc<ProcessContext<Self::Input, ProcessSystems>>,
//   ) -> Rc<Self>
//   where
//     Self: Sized,
//   {
//     EmptySystem(PhantomData).into()
//   }

//   fn update<ProcessSystems: Clone + HList + Sculptor<Self::Deps, i32> + 'static>(
//     self: Rc<Self>,
//     _: Rc<ProcessContext<Self::Input, ProcessSystems>>,
//   ) -> Rc<Self> {
//     self
//   }

//   fn fixed_update<ProcessSystems: Clone + HList + Sculptor<Self::Deps, i32> + 'static>(
//     self: Rc<Self>,
//     _: Rc<ProcessContext<Self::Input, ProcessSystems>>,
//   ) -> Rc<Self> {
//     self
//   }
// }
