use std::{
  any::Any,
  marker::PhantomData,
  rc::Rc,
  thread::sleep,
  time::{Duration, Instant},
};

use frunk::{
  HCons,
  hlist::*,
};
use macroquad::window::next_frame;

pub trait System<ProcessSystems: HList + Sculptor<Self::Deps, i32>>: Any {
  type Input: Clone + 'static;
  type Deps: HList;

  fn start(_: &ProcessContext<Self::Input, Self::Deps>) -> Rc<Self>
  where
    Self: Sized;

  fn update(
    self: Rc<Self>,
    _: &ProcessContext<Self::Input, ProcessSystems>,
  ) -> Rc<dyn System<ProcessSystems, Input = Self::Input, Deps = Self::Deps>>;

  fn fixed_update(
    self: Rc<Self>,
    _: &ProcessContext<Self::Input, ProcessSystems>,
  ) -> Rc<dyn System<ProcessSystems, Input = Self::Input, Deps = Self::Deps>>;
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

pub struct ProcessContext<Input: Clone + 'static, Systems: HList> {
  pub systems: Rc<Systems>,
  pub input: Input,
  options: ProcessContextOptions,
}

impl<Input: Clone + 'static, Systems: Clone + HList + 'static> ProcessContext<Input, Systems> {
  pub fn get<'a, Deps: HList, Target: System<Systems, Input = Input, Deps = Deps>>(
    &'a self,
  ) -> Rc<Target>
  where
    &'a Systems: Plucker<Rc<Target>, i32>,
    Systems: Sculptor<Deps, i32>,
  {
    self.systems.as_ref().pluck().0
  }

  pub fn downcast<Target: Clone + 'static>(&self) -> Option<&ProcessContext<Target, Systems>> {
    (self as &dyn Any).downcast_ref::<ProcessContext<Target, Systems>>()
  }

  fn with<'a, Deps: HList, NewSystem: System<Systems, Input = Input, Deps = Deps>>(
    &'a self,
    target_system: Rc<NewSystem>,
  ) -> ProcessContext<Input, HCons<Rc<NewSystem>, Systems>>
  where
    Systems: Sculptor<Deps, i32>,
  {
    ProcessContext {
      systems: h_cons(target_system, self.systems.as_ref().clone()).into(),
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
    &'a Systems: HFoldRightable<fn(&ProcessContext<Input, Systems>, ) -> &ProcessContext<Input, Systems>, Output=&ProcessContext<Input, Systems>, ProcessContext<Input, Systems>>,
  {
    let mut game_state = self;
    let mut acc_time = 0.0;
    loop {
      let result = terminator(&game_state);

      if let Some(output) = result {
        return output;
      }

      let now = Instant::now();

      game_state = game_state
        .systems
        .as_ref()
        .foldr();
        .iter()
        .enumerate()
        .fold(game_state.clone(), |temp_state, (index, system)| {
          temp_state.with(index, &system.clone().update(&temp_state))
        });

      next_frame().await;

      let skipping_fixed = game_state
        .options
        .should_freeze_fixed
        .as_ref()
        .map(|should_freeze_fixed| should_freeze_fixed(&game_state))
        .unwrap_or(false);

      if !skipping_fixed {
        let fixed_update_count =
          (acc_time / (game_state.options.fixed_time_interval * 1_000_000.0)).floor() as i32;

        acc_time -=
          game_state.options.fixed_time_interval * 1_000_000.0 * fixed_update_count as f32;

        game_state = (0..fixed_update_count).fold(game_state.clone(), |temp_state, _| {
          temp_state.systems.iter().enumerate().fold(
            temp_state.clone(),
            |temp_state, (index, system)| {
              temp_state.with(index, &system.clone().fixed_update(&temp_state))
            },
          )
        });
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
}

type ContextInitializer<Input> = fn(&ProcessContext<Input>) -> Rc<dyn System<Input = Input>>;
pub struct Process<Input: Clone + 'static> {
  input: Input,
  ctx_initializers: Vec<ContextInitializer<Input>>,
}

impl<Input: Clone + 'static> Process<Input> {
  pub fn new(input: &Input) -> Self {
    Process {
      input: input.clone(),
      ctx_initializers: Vec::new(),
    }
  }

  pub fn add_system(&self, system_initializer: ContextInitializer<Input>) -> Self {
    let mut new_vec = self.ctx_initializers.clone();
    new_vec.push(system_initializer);

    Process {
      input: self.input.clone(),
      ctx_initializers: new_vec,
    }
  }

  pub fn start(&self, options: Option<ProcessContextOptions<Input>>) -> ProcessContext<Input> {
    self.ctx_initializers.iter().fold(
      ProcessContext {
        systems: vec![],
        input: self.input.clone(),
        options: options.unwrap_or_default(),
      },
      |ctx: ProcessContext<Input>, initializer| {
        let new_vec = ctx
          .systems
          .iter()
          .map(Rc::clone)
          .chain(vec![initializer(&ctx)])
          .collect();
        ProcessContext {
          systems: new_vec,
          input: ctx.input,
          options: ctx.options,
        }
      },
    )
  }
}
