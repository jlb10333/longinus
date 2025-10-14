use std::ops::Deref;

use macroquad::window::screen_height;
use rapier2d::prelude::*;

pub struct ScreenVector(Vector<f32>);
pub struct PhysicsVector(Vector<f32>);
pub struct MapVector(Vector<f32>);


impl Deref for ScreenVector {
  type Target = Vector<f32>;
  fn deref(&self) -> &Self::Target {
    return &self.0;
  }
}

impl ScreenVector {
  /* Used for internal physics engine dimensions */
  pub fn into_physics(self) -> PhysicsVector {
    return PhysicsVector(self.scale(0.02));
  }

  /* Used for internal physics engine positions, flipping vertically */
  pub fn into_physics_pos(self) -> PhysicsVector {
    return PhysicsVector(vector![self.x, screen_height() - self.y].scale(0.02))
  }

  pub fn new(vector: Vector<f32>) -> ScreenVector {
    return ScreenVector(vector);
  }
}

impl Deref for PhysicsVector {
  type Target = Vector<f32>;
  fn deref(&self) -> &Self::Target {
    return &self.0;
  }
}

impl PhysicsVector {
  /* Used for screen (pixel) dimensions */
  pub fn into_screen(self) -> ScreenVector {
    return ScreenVector(self.scale(50.0));
  }

  /* Used for screen (pixel) positions, flipping vertically */
  pub fn into_screen_pos(self) -> ScreenVector {
    return ScreenVector(vector![self.x, (screen_height() * 0.02) - self.y].scale(50.0))
  }
  
  pub fn new(vector: Vector<f32>) -> PhysicsVector {
    return PhysicsVector(vector);
  }
}

impl Deref for MapVector {
  type Target = Vector<f32>;
  fn deref(&self) -> &Self::Target {
    return &self.0;
  }
}

impl MapVector {
  pub fn into_screen(self) -> ScreenVector {
    return ScreenVector(self.scale(0.125))
  }

  pub fn into_physics(self) -> PhysicsVector {
    return self.into_screen().into_physics();
  }

  pub fn into_physics_pos(self) -> PhysicsVector {
    return self.into_screen().into_physics_pos();
  }

  pub fn new(vector: Vector<f32>) -> MapVector {
    return MapVector(vector);
  }
}

