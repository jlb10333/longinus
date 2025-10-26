use rapier2d::prelude::{Collider, ColliderBuilder, InteractionGroups, RigidBody};

use crate::{
  load_map::{
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_PLAYER, COLLISION_GROUP_PLAYER_PROJECTILE,
    COLLISION_GROUP_WALL, EnemyName,
  },
  units::PhysicsVector,
};

#[derive(Clone)]
pub struct EnemySpawn {
  pub name: EnemyName,
  pub collider: Collider,
  pub rigid_body: RigidBody,
}

#[derive(Clone)]
pub struct PlayerSpawn {
  pub translation: PhysicsVector,
}

#[derive(Clone)]
pub struct Wall {
  pub collider: Collider,
}

pub fn collider_from_enemy_name(name: EnemyName) -> Collider {
  match name {
    EnemyName::Defender => ColliderBuilder::cuboid(0.5, 0.5)
      .collision_groups(InteractionGroups {
        memberships: COLLISION_GROUP_ENEMY,
        filter: COLLISION_GROUP_PLAYER
          .union(COLLISION_GROUP_PLAYER_PROJECTILE)
          .union(COLLISION_GROUP_WALL),
      })
      .build(),
  }
}
