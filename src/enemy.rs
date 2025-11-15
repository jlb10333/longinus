use std::{f32::consts::PI, rc::Rc};

use rapier2d::prelude::{ColliderBuilder, Group, InteractionGroups, RigidBodyHandle, RigidBodySet};

use crate::{
  combat::{Projectile, distance_projection_physics},
  ecs::{Enemy, Entity},
  f::Monad,
  load_map::{
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_WALL,
  },
  physics::PhysicsSystem,
  save::SaveSystem,
  system::System,
  units::{PhysicsVector, UnitConvert},
};

pub struct EnemyDecision {
  pub handle: RigidBodyHandle,
  pub projectiles: Vec<Projectile>,
  pub movement_force: PhysicsVector,
}

pub struct EnemySystem {
  pub decisions: Vec<EnemyDecision>,
}

impl System for EnemySystem {
  fn start(_: crate::system::Context) -> std::rc::Rc<dyn System>
  where
    Self: Sized,
  {
    Rc::new(Self {
      decisions: Vec::new(),
    })
  }

  fn run(&self, ctx: &crate::system::Context) -> std::rc::Rc<dyn System> {
    let decisions = ctx
      .get::<SaveSystem>()
      .unwrap()
      .active_game_state
      .as_ref()
      .map(|active_game_state| {
        active_game_state
          .map_system
          .physics_system
          .entities
          .iter()
          .cloned()
          .map(enemy_behavior(
            &active_game_state.map_system.physics_system.frame_count,
          ))
          .flatten()
          .collect::<Vec<_>>()
      })
      .unwrap_or(vec![]);

    return Rc::new(Self { decisions });
  }
}

fn enemy_behavior(frame_count: &i64) -> impl Fn(Entity) -> Option<EnemyDecision> {
  |entity| {
    entity
      .components
      .get::<Enemy>()
      .bind(|enemy| match enemy.name {
        crate::load_map::EnemyName::Defender => {
          defender_behavior(frame_count.clone(), entity.handle)
        }
      })
  }
}

fn defender_behavior(frame_count: i64, handle: RigidBodyHandle) -> EnemyDecision {
  EnemyDecision {
    handle,
    movement_force: PhysicsVector::zero(),
    projectiles: if frame_count % 50 == 0 {
      let projectile = |offset: f32| Projectile {
        collider: ColliderBuilder::ball(0.2)
          .collision_groups(InteractionGroups {
            memberships: COLLISION_GROUP_ENEMY_PROJECTILE,
            filter: COLLISION_GROUP_PLAYER.union(COLLISION_GROUP_WALL),
          })
          .build(),
        damage: 5.0,
        initial_force: distance_projection_physics(offset + frame_count as f32 / 120.0, 0.7),
        offset: PhysicsVector::zero(),
      };
      Vec::from([
        projectile(0.0),
        projectile(PI / 2.0),
        projectile(PI),
        projectile(PI + (PI / 2.0)),
      ])
    } else {
      Vec::new()
    },
  }
}
