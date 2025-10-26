use rapier2d::prelude::Collider;

use crate::{load_map::EnemyName, units::PhysicsVector};

#[derive(Clone)]
pub struct EnemySpawn {
  pub name: EnemyName,
  pub translation: PhysicsVector,
}

#[derive(Clone)]
pub struct PlayerSpawn {
  pub translation: PhysicsVector,
}

#[derive(Clone)]
pub struct Wall {
  pub collider: Collider,
}
