use std::{any::Any, rc::Rc};

use macroquad::window::next_frame;

pub trait System: Any {
  type Input: Clone + 'static;

  fn start(_: &GameState<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized;

  fn run(&self, _: &GameState<Self::Input>) -> Rc<dyn System<Input = Self::Input>>;
}

#[derive(Clone)]
pub struct GameState<Input: Clone + 'static> {
  pub systems: Vec<Rc<dyn System<Input = Input>>>,
  pub input: Input,
}

impl<Input: Clone + 'static> GameState<Input> {
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

  pub fn downcast<Target: Clone + 'static>(&self) -> Option<&GameState<Target>> {
    (self as &dyn Any).downcast_ref::<GameState<Target>>()
  }

  pub async fn run<Output, Terminator>(self: &Rc<Self>, terminator: Terminator) -> Output
  where
    Terminator: Fn(&GameState<Input>) -> Option<Output>,
  {
    let mut game_state = Rc::clone(self);
    loop {
      let result = terminator(&game_state);

      if let Some(output) = result {
        return output;
      }

      game_state = Rc::new(GameState {
        systems: game_state
          .systems
          .iter()
          .map(|system| system.run(&game_state))
          .collect(),
        input: game_state.input.clone(),
      });

      next_frame().await
    }
  }

  pub async fn run_move<Output, Terminator>(self, terminator: Terminator) -> Output
  where
    Terminator: Fn(&GameState<Input>) -> Option<Output>,
  {
    Rc::new(self).run(terminator).await
  }
}

type ContextInitializer<Input> = fn(&GameState<Input>) -> Rc<dyn System<Input = Input>>;
pub struct Game<Input: Clone + 'static> {
  input: Input,
  ctx_initializers: Vec<ContextInitializer<Input>>,
}

impl<Input: Clone + 'static> Game<Input> {
  pub fn new(input: &Input) -> Self {
    return Game {
      input: input.clone(),
      ctx_initializers: Vec::new(),
    };
  }

  pub fn add_system(&self, system_initializer: ContextInitializer<Input>) -> Self {
    let mut new_vec = self.ctx_initializers.clone();
    new_vec.push(system_initializer);

    return Game {
      input: self.input.clone(),
      ctx_initializers: new_vec,
    };
  }

  pub fn start(&self) -> GameState<Input> {
    self.ctx_initializers.iter().fold(
      GameState {
        systems: vec![],
        input: self.input.clone(),
      },
      |ctx: GameState<Input>, initializer| {
        let new_vec = ctx
          .systems
          .iter()
          .map(Rc::clone)
          .chain(vec![initializer(&ctx)])
          .collect();
        return GameState {
          systems: new_vec,
          input: ctx.input,
        };
      },
    )
  }
}
