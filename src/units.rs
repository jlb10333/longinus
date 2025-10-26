use std::ops::Deref;

use derive_more::{Add, Div, Mul, Sub};
use macroquad::window::screen_height;
use rapier2d::{na::Vector2, prelude::*};

use crate::load_map::TILE_DIMENSION_PHYSICS;

pub fn vec_zero() -> Vector2<f32> {
  return vector![0.0, 0.0];
}

pub trait UnitConvert<Other>: Clone + Copy {
  fn zero() -> Self;
  fn convert(self) -> Other;
}

pub trait UnitConvert2<Other>: UnitConvert<Other> {
  fn into_vec(self) -> Vector2<f32>;
  fn from_vec(vector: Vector2<f32>) -> Self;
  fn into_pos(self, camera_position: Vector2<f32>) -> Other;
  fn x(self) -> f32 {
    return self.into_vec().x;
  }
  fn y(self) -> f32 {
    return self.into_vec().y;
  }
}

/* ScreenScalar */

#[derive(Sub, Add, Mul, Clone, Copy)]
pub struct ScreenScalar(pub f32);

impl Deref for ScreenScalar {
  type Target = f32;
  fn deref(&self) -> &Self::Target {
    return &self.0;
  }
}

impl UnitConvert<PhysicsScalar> for ScreenScalar {
  fn zero() -> Self {
    return Self(0.0);
  }
  fn convert(self) -> PhysicsScalar {
    return PhysicsScalar(*self * 0.2);
  }
}

/* PhysicsScalar */

#[derive(Sub, Add, Mul, Div, Clone, Copy)]
pub struct PhysicsScalar(pub f32);

impl Deref for PhysicsScalar {
  type Target = f32;
  fn deref(&self) -> &Self::Target {
    return &self.0;
  }
}

impl UnitConvert<ScreenScalar> for PhysicsScalar {
  fn zero() -> Self {
    return Self(0.0);
  }
  fn convert(self) -> ScreenScalar {
    return ScreenScalar(*self * 50.0);
  }
}

/* ScreenVector */

pub type ScreenVector = Vector2<ScreenScalar>;

impl UnitConvert<PhysicsVector> for ScreenVector {
  fn zero() -> Self {
    return vector![ScreenScalar::zero(), ScreenScalar::zero()];
  }
  fn convert(self) -> PhysicsVector {
    return PhysicsVector::from_vec(vector![self.into_vec().x, -self.into_vec().y].scale(0.02));
  }
}

impl UnitConvert2<PhysicsVector> for ScreenVector {
  fn into_vec(self) -> Vector2<f32> {
    let mapped: Vec<f32> = self.iter().map(ScreenScalar::deref).cloned().collect();
    return vector![mapped[0], mapped[1]];
  }
  fn from_vec(vector: Vector2<f32>) -> Self {
    return vector![ScreenScalar(vector.x), ScreenScalar(vector.y)];
  }
  fn into_pos(self, camera_position: Vector2<f32>) -> PhysicsVector {
    return PhysicsVector::from_vec(
      vector![self.into_vec().x, screen_height() - self.into_vec().y].scale(0.02) + camera_position,
    );
  }
}

/* PhysicsVector */

pub type PhysicsVector = Vector2<PhysicsScalar>;

impl UnitConvert<ScreenVector> for PhysicsVector {
  fn convert(self) -> ScreenVector {
    return ScreenVector::from_vec(vector![self.x(), -self.y()].scale(50.0));
  }
  fn zero() -> Self {
    return Self::from_vec(vec_zero());
  }
}

impl UnitConvert2<ScreenVector> for PhysicsVector {
  fn into_vec(self) -> Vector2<f32> {
    let mapped: Vec<f32> = self.iter().map(PhysicsScalar::deref).cloned().collect();
    return vector![mapped[0], mapped[1]];
  }
  fn from_vec(vector: Vector2<f32>) -> Self {
    return vector![PhysicsScalar(vector.x), PhysicsScalar(vector.y)];
  }
  fn into_pos(self, camera_position: Vector2<f32>) -> Vector<ScreenScalar> {
    return Vector::<ScreenScalar>::from_vec(
      vector![
        self.into_vec().x,
        (screen_height() * 0.02) - self.into_vec().y
      ]
      .scale(50.0)
        - camera_position,
    );
  }
}
