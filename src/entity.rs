use rapier2d::prelude::{Collider, RigidBody};


pub struct Entity {
  pub collider: Collider,
  pub rigid_body: RigidBody,
}

pub struct Player {
  pub entity: Entity,
}

pub struct Enemy {
  pub entity: Entity,
}


pub struct Wall {
  pub collider: Collider,
}