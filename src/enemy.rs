use std::{f32::consts::PI, rc::Rc};

use rapier2d::prelude::{ColliderBuilder, InteractionGroups, RigidBodyHandle};

use crate::{
  combat::{Projectile, distance_projection_physics},
  ecs::{Enemy, Entity},
  f::Monad,
  load_map::{
    COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALL, EnemyName,
  },
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert},
};

pub struct EnemyDecision {
  pub handle: RigidBodyHandle,
  pub projectiles: Vec<Projectile>,
  pub movement_force: PhysicsVector,
  pub enemy: EnemyName,
}

pub struct EnemySystem {
  pub decisions: Vec<EnemyDecision>,
}

impl System for EnemySystem {
  type Input = SaveData;
  fn start(
    _: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    Rc::new(Self {
      decisions: Vec::new(),
    })
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>> {
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();
    let decisions = physics_system
      .entities
      .iter()
      .cloned()
      .filter_map(enemy_behavior)
      .collect::<Vec<_>>();

    Rc::new(Self { decisions })
  }
}

fn enemy_behavior(entity: Entity) -> Option<EnemyDecision> {
  {
    entity
      .components
      .get::<Enemy>()
      .bind(|enemy| match enemy.name {
        crate::load_map::EnemyName::Defender(cooldown) => {
          defender_behavior(cooldown, entity.handle)
        }
      })
  }
}

fn defender_behavior(cooldown: i32, handle: RigidBodyHandle) -> EnemyDecision {
  EnemyDecision {
    handle,
    movement_force: PhysicsVector::zero(),
    projectiles: if cooldown % 50 == 0 {
      let projectile = |offset: f32| Projectile {
        collider: ColliderBuilder::ball(0.2)
          .collision_groups(InteractionGroups {
            memberships: COLLISION_GROUP_ENEMY_PROJECTILE,
            filter: COLLISION_GROUP_PLAYER.union(COLLISION_GROUP_WALL),
          })
          .build(),
        damage: 5.0,
        initial_force: distance_projection_physics(offset + cooldown as f32 / 120.0, 0.7),
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
    enemy: EnemyName::Defender(cooldown - 1),
  }
}
