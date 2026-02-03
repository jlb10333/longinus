use std::{
  any::Any,
  rc::Rc,
  thread::sleep,
  time::{Duration, Instant},
};

use macroquad::window::next_frame;

pub trait System: Any {
  type Input: Clone + 'static;

  fn start(_: &ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized;

  fn update(&self, _: &ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>;

  fn fixed_update(&self, _: &ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>;
}

#[derive(Clone, Copy)]
pub struct ProcessContextOptions {
  /// Interval to target for fixed_update calls, in seconds
  pub fixed_time_interval: f32,
  pub target_fps: i32,
}

impl Default for ProcessContextOptions {
  fn default() -> Self {
    Self {
      fixed_time_interval: 1.0 / 60.0,
      target_fps: 24,
    }
  }
}

#[derive(Clone)]
pub struct ProcessContext<Input: Clone + 'static> {
  pub systems: Vec<Rc<dyn System<Input = Input>>>,
  pub input: Input,
  options: ProcessContextOptions,
}

impl<Input: Clone + 'static> ProcessContext<Input> {
  pub fn get<Target>(&self) -> Option<Rc<Target>>
  where
    Target: System<Input = Input>,
  {
    self
      .systems
      .iter()
      .find(|&system| {
        (Rc::clone(system) as Rc<dyn Any>)
          .downcast::<Target>()
          .is_ok()
      })
      .and_then(|system| (Rc::clone(system) as Rc<dyn Any>).downcast::<Target>().ok())
  }

  pub fn downcast<Target: Clone + 'static>(&self) -> Option<&ProcessContext<Target>> {
    (self as &dyn Any).downcast_ref::<ProcessContext<Target>>()
  }

  fn with(&self, target_index: usize, target_system: &Rc<dyn System<Input = Input>>) -> Self {
    Self {
      systems: self
        .systems
        .iter()
        .enumerate()
        .map(|(index, system)| {
          Rc::clone(if index == target_index {
            target_system
          } else {
            system
          })
        })
        .collect(),
      input: self.input.clone(),
      options: self.options,
    }
  }

  pub async fn run<Output, Terminator>(self, terminator: Terminator) -> Output
  where
    Terminator: Fn(&ProcessContext<Input>) -> Option<Output>,
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
        .iter()
        .enumerate()
        .fold(game_state.clone(), |temp_state, (index, system)| {
          temp_state.with(index, &system.update(&temp_state))
        });

      next_frame().await;

      let fixed_update_count =
        (acc_time / (game_state.options.fixed_time_interval * 1_000_000.0)).floor() as i32;

      acc_time -= game_state.options.fixed_time_interval * 1_000_000.0 * fixed_update_count as f32;

      game_state = (0..fixed_update_count).fold(game_state.clone(), |temp_state, _| {
        temp_state
          .systems
          .iter()
          .enumerate()
          .fold(temp_state.clone(), |temp_state, (index, system)| {
            temp_state.with(index, &system.fixed_update(&temp_state))
          })
      });

      let frame_time = now.elapsed().as_millis() as f32;

      let min_frame_time = 1000.0 / game_state.options.target_fps as f32;

      if frame_time < min_frame_time {
        let time_to_sleep = min_frame_time - frame_time;

        sleep(Duration::from_millis(time_to_sleep as u64));
      }

      acc_time += now.elapsed().as_micros() as f32;
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

  pub fn start(&self, options: Option<ProcessContextOptions>) -> ProcessContext<Input> {
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
