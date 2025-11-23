use std::{f32::consts::PI, rc::Rc};

use rapier2d::{
  na::Vector2,
  prelude::{ColliderBuilder, InteractionGroups, RigidBodyHandle, RigidBodySet},
};

use crate::{
  combat::{Projectile, distance_projection_physics},
  ecs::{Enemy, Entity},
  load_map::{
    COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALL, EnemySpawn,
    MapEnemyName,
  },
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert, UnitConvert2, vec_zero},
};

pub struct EnemyDecisionEnemySpawn {
  pub enemy_spawn: EnemySpawn,
  pub initial_force: Vector2<f32>,
}

pub struct EnemyDecision {
  pub handle: RigidBodyHandle,
  pub projectiles: Vec<Projectile>,
  pub movement_force: PhysicsVector,
  pub enemy: Enemy,
  pub enemies_to_spawn: Vec<EnemyDecisionEnemySpawn>,
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

    let player_translation =
      physics_system.rigid_body_set[physics_system.player_handle].translation();

    let enemy_behavior =
      enemy_behavior_generator(player_translation, &physics_system.rigid_body_set);

    let decisions = physics_system
      .entities
      .iter()
      .cloned()
      .filter_map(enemy_behavior)
      .collect::<Vec<_>>();

    Rc::new(Self { decisions })
  }
}

fn enemy_behavior_generator(
  player_translation: &Vector2<f32>,
  physics_rigid_bodies: &RigidBodySet,
) -> impl Fn(Entity) -> Option<EnemyDecision> {
  |entity| {
    entity
      .components
      .get::<Enemy>()
      .map(|enemy| match enemy.as_ref() {
        Enemy::Defender(defender) => defender.behavior(entity.handle),
        Enemy::Seeker(seeker) => {
          seeker.behavior(entity.handle, player_translation, physics_rigid_bodies)
        }
        Enemy::SeekerGenerator(seeker_generator) => {
          seeker_generator.behavior(entity.handle, player_translation, physics_rigid_bodies)
        }
      })
  }
}

#[derive(Clone)]
pub struct EnemyDefender {
  pub cooldown: i32,
}

impl EnemyDefender {
  pub fn behavior(&self, handle: RigidBodyHandle) -> EnemyDecision {
    let should_fire_projectiles = self.cooldown % 50 == 0;
    EnemyDecision {
      handle,
      movement_force: PhysicsVector::zero(),
      projectiles: if should_fire_projectiles {
        let projectile = |offset: f32| Projectile {
          collider: ColliderBuilder::ball(0.2)
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_ENEMY_PROJECTILE,
              filter: COLLISION_GROUP_PLAYER.union(COLLISION_GROUP_WALL),
            })
            .build(),
          damage: 5.0,
          initial_force: distance_projection_physics(offset + self.cooldown as f32 / 120.0, 0.7),
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
      enemy: Enemy::Defender(EnemyDefender {
        cooldown: self.cooldown - 1,
      }),
      enemies_to_spawn: vec![],
    }
  }
}

#[derive(Clone)]
pub struct EnemySeeker;

const SEEKER_SPEED_CAP: f32 = 10.0;
const SEEKER_SPEED: f32 = 0.3;

impl EnemySeeker {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    physics_rigid_bodies: &RigidBodySet,
  ) -> EnemyDecision {
    let self_rigid_body = &physics_rigid_bodies[handle];
    let direction_to_player = player_translation - self_rigid_body.translation();
    let velocity_towards_player = (self_rigid_body.linvel().dot(&direction_to_player)
      / direction_to_player.magnitude())
      * direction_to_player.normalize();

    let velocity_away_from_player = self_rigid_body.linvel() - velocity_towards_player;
    let movement_force = PhysicsVector::from_vec(
      if velocity_towards_player.magnitude() >= SEEKER_SPEED_CAP {
        vec_zero()
      } else {
        direction_to_player.normalize() * SEEKER_SPEED
      } - velocity_away_from_player.normalize() * SEEKER_SPEED * 0.3,
    );
    EnemyDecision {
      movement_force,
      handle,
      projectiles: vec![],
      enemies_to_spawn: vec![],
      enemy: Enemy::Seeker(self.clone()),
    }
  }
}

#[derive(Clone)]
pub struct EnemySeekerGenerator {
  pub cooldown: i32,
}

const SEEKER_GENERATOR_INITIAL_FORCE: f32 = 5.0;
const SEEKER_SPAWN_COOLDOWN: i32 = 1000;

impl EnemySeekerGenerator {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    physics_rigid_bodies: &RigidBodySet,
  ) -> EnemyDecision {
    let should_spawn_enemy = self.cooldown % SEEKER_SPAWN_COOLDOWN == 0;
    EnemyDecision {
      movement_force: PhysicsVector::zero(),
      handle,
      projectiles: vec![],
      enemy: Enemy::SeekerGenerator(Self {
        cooldown: self.cooldown - 1,
      }),
      enemies_to_spawn: if should_spawn_enemy {
        let self_rigid_body = &physics_rigid_bodies[handle];
        let direction_to_player = player_translation - self_rigid_body.translation();
        let initial_force = direction_to_player.normalize() * SEEKER_GENERATOR_INITIAL_FORCE;
        vec![EnemyDecisionEnemySpawn {
          initial_force,
          enemy_spawn: EnemySpawn::new(&MapEnemyName::Seeker, *self_rigid_body.translation()),
        }]
      } else {
        vec![]
      },
    }
  }
}
