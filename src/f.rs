pub trait Monad<T> {
  type SelfType<A>: Monad<A>;

  fn bind<B, F>(self, transform: F) -> Self::SelfType<B>
  where
    F: Fn(&T) -> B;
}

impl<T> Monad<T> for Option<T> {
  type SelfType<A> = Option<A>;

  fn bind<B, F>(self, transform: F) -> Self::SelfType<B>
  where
    F: Fn(&T) -> B,
  {
    match self {
      Some(some) => Some(transform(&some)),
      None => None,
    }
  }
}

impl<T, E> Monad<T> for Result<T, E> {
  type SelfType<A> = Result<A, E>;

  fn bind<B, F>(self, transform: F) -> Self::SelfType<B>
  where
    F: Fn(&T) -> B,
  {
    match self {
      Ok(ok) => Ok(transform(&ok)),
      Err(err) => Err(err),
    }
  }
}

pub trait MonadTranslate<A, Target>: Monad<A>
where
  Target: Monad<A>,
{
  fn translate(self) -> Target;
}

impl<T, E> MonadTranslate<T, Option<T>> for Result<T, E> {
  fn translate(self) -> Option<T> {
    return match self {
      Ok(ok) => Some(ok),
      Err(_) => None,
    };
  }
}
