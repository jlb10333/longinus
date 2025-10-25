use std::ops::Deref;

use derive_more::{Add, Mul, Sub};
use macroquad::window::screen_height;
use rapier2d::{na::Vector2, prelude::*};

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

#[derive(Sub, Add, Mul, Clone, Copy)]
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

/* MapVector */

#[derive(Sub, Add, Mul, Clone, Copy)]
pub struct MapScalar(pub f32);

impl Deref for MapScalar {
  type Target = f32;
  fn deref(&self) -> &Self::Target {
    return &self.0;
  }
}

impl UnitConvert<PhysicsScalar> for MapScalar {
  fn zero() -> Self {
    return MapScalar(0.0);
  }
  fn convert(self) -> PhysicsScalar {
    return PhysicsScalar(*self * 0.125 * 0.02);
  }
}

pub type MapVector = Vector2<MapScalar>;

impl UnitConvert<PhysicsVector> for MapVector {
  fn zero() -> Self {
    return vector![MapScalar::zero(), MapScalar::zero()];
  }

  fn convert(self) -> PhysicsVector {
    let x = self.data.0[0][0];
    let y = self.data.0[0][1];
    return vector![x.convert(), y.convert()];
  }
}

impl UnitConvert2<PhysicsVector> for MapVector {
  fn into_vec(self) -> Vector2<f32> {
    let mapped: Vec<f32> = self.iter().map(MapScalar::deref).cloned().collect();
    return vector![mapped[0], mapped[1]];
  }
  fn from_vec(vector: Vector2<f32>) -> Self {
    return vector![MapScalar(vector.x), MapScalar(vector.y)];
  }
  fn into_pos(self, _: Vector2<f32>) -> PhysicsVector {
    return self.convert();
  }
}
