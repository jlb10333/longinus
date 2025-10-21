use std::{any::Any, rc::Rc};

use macroquad::window::next_frame;

pub trait System: Any {
  fn start(_: Context) -> Rc<dyn System>
  where
    Self: Sized;

  fn run(&self, _: &Context) -> Rc<dyn System>;
}

#[derive(Clone)]
pub struct Context(pub Vec<Rc<dyn System>>);

impl Context {
  pub fn get<Target>(&self) -> Option<Rc<Target>>
  where
    Target: System,
  {
    return match self.0.iter().find(|&state| {
      (Rc::clone(state) as Rc<dyn Any>)
        .downcast::<Target>()
        .is_ok()
    }) {
      Some(state) => match (Rc::clone(state) as Rc<dyn Any>).downcast::<Target>() {
        Ok(target) => Some(target),
        Err(_) => None,
      },

      None => None,
    };
  }
}

pub struct Game {
  ctx_initializers: Vec<fn(Context) -> Rc<dyn System>>,
}

impl Game {
  pub fn new() -> Self {
    return Game {
      ctx_initializers: Vec::new(),
    };
  }

  pub fn add_system(&self, system_initializer: fn(Context) -> Rc<dyn System>) -> Self {
    let mut new_vec = self.ctx_initializers.clone();
    new_vec.push(system_initializer);

    return Game {
      ctx_initializers: new_vec,
    };
  }

  pub async fn run(&self) {
    /* Initialize Context */
    let mut ctx =
      self
        .ctx_initializers
        .iter()
        .fold(Context(Vec::new()), |ctx: Context, initializer| {
          let mut new_vec = ctx.0.clone();
          new_vec.push(initializer(ctx));
          return Context(new_vec);
        });

    loop {
      ctx = Context(ctx.0.iter().map(|system| system.run(&ctx)).collect());

      next_frame().await;
    }
  }
}
