use std::{any::Any, rc::Rc};

use macroquad::window::next_frame;

pub trait System: Any {
  type Input: Clone + 'static;

  fn start(_: &ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized;

  fn run(&self, _: &ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>;
}

#[derive(Clone)]
pub struct ProcessContext<Input: Clone + 'static> {
  pub systems: Vec<Rc<dyn System<Input = Input>>>,
  pub input: Input,
}

impl<Input: Clone + 'static> ProcessContext<Input> {
  pub fn get<Target>(&self) -> Option<Rc<Target>>
  where
    Target: System<Input = Input>,
  {
    return self
      .systems
      .iter()
      .find(|&system| {
        (Rc::clone(system) as Rc<dyn Any>)
          .downcast::<Target>()
          .is_ok()
      })
      .map(|system| (Rc::clone(system) as Rc<dyn Any>).downcast::<Target>().ok())
      .flatten();
  }

  pub fn downcast<Target: Clone + 'static>(&self) -> Option<&ProcessContext<Target>> {
    (self as &dyn Any).downcast_ref::<ProcessContext<Target>>()
  }

  fn with(
    self: &Rc<Self>,
    target_index: usize,
    target_system: &Rc<dyn System<Input = Input>>,
  ) -> Rc<Self> {
    Rc::new(Self {
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
    })
  }

  pub async fn run<Output, Terminator>(self: &Rc<Self>, terminator: Terminator) -> Output
  where
    Terminator: Fn(&ProcessContext<Input>) -> Option<Output>,
  {
    let mut game_state = Rc::clone(self);
    loop {
      let result = terminator(&game_state);

      if let Some(output) = result {
        return output;
      }

      game_state = game_state
        .systems
        .iter()
        .enumerate()
        .fold(Rc::clone(&game_state), |temp_state, (index, system)| {
          temp_state.with(index, &system.run(&temp_state))
        });

      next_frame().await
    }
  }

  pub async fn run_move<Output, Terminator>(self, terminator: Terminator) -> Output
  where
    Terminator: Fn(&ProcessContext<Input>) -> Option<Output>,
  {
    Rc::new(self).run(terminator).await
  }
}

type ContextInitializer<Input> = fn(&ProcessContext<Input>) -> Rc<dyn System<Input = Input>>;
pub struct Process<Input: Clone + 'static> {
  input: Input,
  ctx_initializers: Vec<ContextInitializer<Input>>,
}

impl<Input: Clone + 'static> Process<Input> {
  pub fn new(input: &Input) -> Self {
    return Process {
      input: input.clone(),
      ctx_initializers: Vec::new(),
    };
  }

  pub fn add_system(&self, system_initializer: ContextInitializer<Input>) -> Self {
    let mut new_vec = self.ctx_initializers.clone();
    new_vec.push(system_initializer);

    return Process {
      input: self.input.clone(),
      ctx_initializers: new_vec,
    };
  }

  pub fn start(&self) -> ProcessContext<Input> {
    self.ctx_initializers.iter().fold(
      ProcessContext {
        systems: vec![],
        input: self.input.clone(),
      },
      |ctx: ProcessContext<Input>, initializer| {
        let new_vec = ctx
          .systems
          .iter()
          .map(Rc::clone)
          .chain(vec![initializer(&ctx)])
          .collect();
        return ProcessContext {
          systems: new_vec,
          input: ctx.input,
        };
      },
    )
  }
}
