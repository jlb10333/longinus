use std::any::Any;

pub trait State: Any {}

#[derive(Clone)]
struct Module<'a>(pub Vec<&'a dyn State>);

impl<'a> Module<'a> {
  pub fn new(states: Vec<&'a dyn State>) -> Self {
    return Module(states);
  }

  pub fn get<Target>(&self) -> Option<&Target>
  where
    Target: State,
  {
    return match self
      .0
      .iter()
      .find(|&&state| (state as &dyn Any).downcast_ref::<Target>().is_some())
    {
      Some(&state) => (state as &dyn Any).downcast_ref::<Target>(),
      None => None,
    };
  }

  pub fn map<F>(&self, f: F) -> Self
  where
    F: Fn(&'a dyn State) -> &'a dyn State,
  {
    let new_vec = self.0.iter().map(|&state| f(state)).collect();
    return Module(new_vec);
  }
}

pub trait System: State {
  fn start(deps: Module) -> dyn State
  where
    Self: Sized;

  fn run(&self, deps: Module) -> dyn State;
}

struct Game {
  initializers: Vec<fn() -> dyn State>,
}

impl Game {
  pub fn add_state(&self, initializer: fn() -> dyn State) -> Self {
    let mut new_initializers = self.initializers.clone();
    new_initializers.push(initializer);
    return Game {
      initializers: new_initializers,
    };
  }

  pub fn run(&self) {
    let states = self.initializers.iter().map(|f| &f()).collect();
    let mut module = Module::new(states);
    loop {
      let new_states =
        module.0.iter().map(
          |&state| match (state as &dyn Any).downcast_ref::<&dyn System>() {
            Some(&system) => &system.run(module),
            None => state,
          },
        );
    }
  }
}
