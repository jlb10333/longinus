pub trait System<'a> {
  type Deps;

  fn start() -> Self
  where
    Self: Sized;

  fn run(&self, deps: &'a Self::Deps) -> Self
  where
    Self: Sized;
}
