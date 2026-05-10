use std::{
  f32::consts::PI,
  ops::{Add, Mul},
  rc::Rc,
};

#[derive(Clone)]
pub struct Easing<Unit>(Rc<dyn Fn(f32) -> Unit>);

impl<Unit> Easing<Unit> {
  pub fn new(f: &Rc<dyn Fn(f32) -> Unit>) -> Self {
    Self(Rc::clone(f))
  }

  pub fn at(&self, x: f32) -> Unit {
    self.0(x)
  }
}

impl<Unit> Easing<Unit>
where
  Unit: 'static,
{
  pub fn offset(&self, o: f32) -> Self {
    let f = Rc::clone(&self.0);
    Self(Rc::new(move |t| f(t - o)) as Rc<dyn Fn(f32) -> Unit>)
  }

  pub fn scale(&self, s: f32) -> Self {
    let f = Rc::clone(&self.0);
    Self(Rc::new(move |t| f(t / s)) as Rc<dyn Fn(f32) -> Unit>)
  }
}

impl<Unit, Rhs, Output> Mul<Rhs> for &Easing<Unit>
where
  Unit: Mul<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<Output>;
  fn mul(self, rhs: Rhs) -> Self::Output {
    let f = Rc::clone(&self.0);
    Easing(Rc::new(move |t| f(t) * rhs.to_owned()) as Rc<dyn Fn(f32) -> Output>)
  }
}

impl<Unit, Rhs, Output> Mul<Rhs> for Easing<Unit>
where
  Unit: Mul<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<Output>;
  fn mul(self, rhs: Rhs) -> Self::Output {
    (&self).mul(rhs)
  }
}

impl<Unit, Rhs, Output> Add<Easing<Rhs>> for &Easing<Unit>
where
  Unit: Add<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<Output>;
  fn add(self, rhs: Easing<Rhs>) -> Self::Output {
    let f = Rc::clone(&self.0);
    Easing(Rc::new(move |t| f(t) + rhs.at(t)) as Rc<dyn Fn(f32) -> Output>)
  }
}

impl<Unit, Rhs, Output> Add<Easing<Rhs>> for Easing<Unit>
where
  Unit: Add<Rhs, Output = Output> + Clone + 'static,
  Rhs: Clone + 'static,
{
  type Output = Easing<Output>;
  fn add(self, rhs: Easing<Rhs>) -> Self::Output {
    (&self).add(rhs)
  }
}

pub fn ease_in_sine() -> Easing<f32> {
  Easing(Rc::new(|x| 1.0 - (x * PI / 2.0).cos()))
}

pub fn ease_out_sine() -> Easing<f32> {
  Easing(Rc::new(|x| (x * PI / 2.0).sin()))
}

pub fn ease_in_out_sine() -> Easing<f32> {
  Easing(Rc::new(|x| 0.5 - 0.5 * (x * PI).cos()))
}

pub fn ease_in_out_sine_ddt() -> Easing<f32> {
  Easing(Rc::new(|x| (PI / 2.0) * (x * PI).sin()))
}

pub fn ease_in_out_sine_ddt2() -> Easing<f32> {
  Easing(Rc::new(|x| (PI * PI / 2.0) * (x * PI).cos()))
}

pub fn ease_out_cubic() -> Easing<f32> {
  Easing(Rc::new(|x| 1.0 - (1.0 - x).powf(3.0)))
}

pub fn linear() -> Easing<f32> {
  Easing(Rc::new(|x| x))
}
