use std::{
  f32::consts::PI,
  ops::{Add, Mul},
};

pub struct Easing<'a, Unit>(Box<dyn 'a + Fn(f32) -> Unit>);

impl<'a, Unit> Easing<'a, Unit> {
  pub fn new<F>(f: F) -> Self
  where
    F: Fn(f32) -> Unit + 'a,
  {
    Self(Box::new(f))
  }

  pub fn at(&self, x: f32) -> Unit {
    self.0(x)
  }
}

impl<'a, Unit, Rhs, Output> Mul<Rhs> for &'a Easing<'a, Unit>
where
  Unit: Mul<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<'a, Output>;
  fn mul(self, rhs: Rhs) -> Self::Output {
    Easing::new(move |t| self.0(t) * rhs.to_owned())
  }
}

impl<'a, Unit, Rhs, Output> Mul<Rhs> for Easing<'a, Unit>
where
  Unit: Mul<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<'a, Output>;
  fn mul(self, rhs: Rhs) -> Self::Output {
    Easing::new(move |t| self.0(t) * rhs.to_owned())
  }
}

impl<'a, Unit, Rhs, Output> Add<&'a Easing<'a, Rhs>> for &'a Easing<'a, Unit>
where
  Unit: Add<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<'a, Output>;
  fn add(self, rhs: &'a Easing<'a, Rhs>) -> Self::Output {
    Easing::new(move |t| self.0(t) + rhs.at(t))
  }
}

impl<'a, Unit, Rhs, Output> Add<Easing<'a, Rhs>> for Easing<'a, Unit>
where
  Unit: Add<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<'a, Output>;
  fn add(self, rhs: Easing<'a, Rhs>) -> Self::Output {
    Easing::new(move |t| self.0(t) + rhs.at(t))
  }
}

pub fn ease_in_sine() -> Easing<'static, f32> {
  Easing::new(|x| 1.0 - (x * PI / 2.0).cos())
}

pub fn ease_out_sine() -> Easing<'static, f32> {
  Easing::new(|x| (x * PI / 2.0).sin())
}

pub fn ease_in_out_sine() -> Easing<'static, f32> {
  Easing::new(|x| 0.5 - 0.5 * (x * PI).cos())
}

pub fn ease_in_out_sine_ddt() -> Easing<'static, f32> {
  Easing::new(|x| (PI / 2.0) * (x * PI).sin())
}

pub fn ease_in_out_sine_ddt2() -> Easing<'static, f32> {
  Easing::new(|x| (PI * PI / 2.0) * (x * PI).cos())
}
