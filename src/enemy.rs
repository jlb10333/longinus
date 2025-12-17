use std::{f32::consts::PI, rc::Rc};

use macroquad::{prelude::rand, rand::RandGenerator};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  combat::{Projectile, distance_projection_physics},
  ecs::{ComponentSet, Enemy, Entity, EntityHandle},
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
  pub movement_force: Vector2<f32>,
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

    let rng = rand::RandGenerator::new();
    rng.srand(physics_system.frame_count as u64);

    let player_translation =
      physics_system.rigid_body_set[physics_system.player_handle].translation();

    let enemy_behavior =
      enemy_behavior_generator(player_translation, &physics_system.rigid_body_set, &rng);

    let decisions = physics_system
      .entities
      .iter()
      .filter_map(enemy_behavior)
      .collect::<Vec<_>>();

    Rc::new(Self { decisions })
  }
}

fn enemy_behavior_generator(
  player_translation: &Vector2<f32>,
  physics_rigid_bodies: &RigidBodySet,
  rng: &RandGenerator,
) -> impl Fn((&EntityHandle, &Rc<Entity>)) -> Option<EnemyDecision> {
  |(&handle, entity)| {
    if let EntityHandle::RigidBody(rigid_body_handle) = handle {
      entity
        .components
        .get::<Enemy>()
        .map(|enemy| match enemy.as_ref() {
          Enemy::Goblin(goblin) => goblin.behavior(
            rigid_body_handle,
            player_translation,
            physics_rigid_bodies,
            rng,
          ),
          Enemy::Defender(defender) => defender.behavior(rigid_body_handle),
          Enemy::Seeker(seeker) => {
            seeker.behavior(rigid_body_handle, player_translation, physics_rigid_bodies)
          }
          Enemy::SeekerGenerator(seeker_generator) => {
            seeker_generator.behavior(rigid_body_handle, player_translation, physics_rigid_bodies)
          }
        })
    } else {
      None
    }
  }
}

const ENEMY_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_ENEMY_PROJECTILE,
  filter: COLLISION_GROUP_PLAYER.union(COLLISION_GROUP_WALL),
};

#[derive(Clone)]
pub enum EnemyGoblinState {
  Shooting(i32),
  Cruising(i32),
  Accelerating(i32, Vector2<f32>),
  Decelerating(i32),
}

impl EnemyGoblinState {
  pub fn initial() -> Self {
    Self::Shooting(GOBLIN_STATE_SHOOTING_INITIAL_FRAMES)
  }
}

#[derive(Clone)]
pub struct EnemyGoblin {
  pub state: EnemyGoblinState,
}

const GOBLIN_STATE_CRUISING_INITIAL_FRAMES: i32 = 70;
const GOBLIN_STATE_SHOOTING_INITIAL_FRAMES: i32 = 50;
const GOBLIN_STATE_ACCELERATING_INITIAL_FRAMES: i32 = 10;
const GOBLIN_STATE_DECELERATING_INITIAL_FRAMES: i32 = 10;

const GOBLIN_MOVE_FORCE: f32 = 0.2;
const GOBLIN_PROJECTILE_SPEED: f32 = 1.0;
const GOBLIN_PROJECTILE_DAMAGE: f32 = 5.0;

impl EnemyGoblin {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    rigid_body_set: &RigidBodySet,
    rng: &RandGenerator,
  ) -> EnemyDecision {
    match self.state {
      EnemyGoblinState::Shooting(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Shooting(frames_left - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Accelerating(
                GOBLIN_STATE_ACCELERATING_INITIAL_FRAMES,
                vector![rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)],
              ),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyGoblinState::Cruising(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Cruising(frames_left - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Decelerating(GOBLIN_STATE_DECELERATING_INITIAL_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyGoblinState::Accelerating(frames_left, direction) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Accelerating(frames_left - 1, direction),
            }),
            movement_force: direction.normalize() * GOBLIN_MOVE_FORCE,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Cruising(GOBLIN_STATE_CRUISING_INITIAL_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyGoblinState::Decelerating(frames_left) => {
        let linvel = rigid_body_set[handle].linvel();

        if frames_left > 0 && linvel.magnitude() > 0.0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Decelerating(frames_left - 1),
            }),
            movement_force: -linvel.normalize() * GOBLIN_MOVE_FORCE,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Shooting(GOBLIN_STATE_SHOOTING_INITIAL_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![Projectile {
              collider: ColliderBuilder::ball(0.2)
                .collision_groups(ENEMY_GROUPS)
                .build(),
              damage: GOBLIN_PROJECTILE_DAMAGE,
              initial_impulse: PhysicsVector::from_vec(
                (player_translation - rigid_body_set[handle].translation()).normalize()
                  * GOBLIN_PROJECTILE_SPEED,
              ),
              offset: PhysicsVector::zero(),
              force_mod: 0.0,
              component_set: ComponentSet::new(),
            }],
          }
        }
      }
    }
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
      movement_force: vec_zero(),
      projectiles: if should_fire_projectiles {
        let projectile = |offset: f32| Projectile {
          collider: ColliderBuilder::ball(0.2)
            .collision_groups(ENEMY_GROUPS)
            .build(),
          damage: 5.0,
          initial_impulse: distance_projection_physics(offset + self.cooldown as f32 / 120.0, 0.7),
          offset: PhysicsVector::zero(),
          component_set: ComponentSet::new(),
          force_mod: 0.0,
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

const SEEKER_SPEED_CAP: f32 = 5.0;
const SEEKER_SPEED: f32 = 0.3;

impl EnemySeeker {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    physics_rigid_bodies: &RigidBodySet,
  ) -> EnemyDecision {
    let movement_force = {
      let self_rigid_body = &physics_rigid_bodies[handle];
      let direction_to_player = player_translation - self_rigid_body.translation();
      let velocity_towards_player = (self_rigid_body.linvel().dot(&direction_to_player)
        / direction_to_player.magnitude())
        * direction_to_player.normalize();

      let velocity_away_from_player = self_rigid_body.linvel() - velocity_towards_player;

      (if velocity_towards_player.magnitude() >= SEEKER_SPEED_CAP {
        vec_zero()
      } else {
        direction_to_player.normalize() * SEEKER_SPEED
      }) - velocity_away_from_player.normalize() * SEEKER_SPEED * 0.3
    };
    EnemyDecision {
      movement_force,
      handle,
      projectiles: vec![],
      enemies_to_spawn: vec![],
      enemy: Enemy::Seeker(Self),
    }
  }
}

#[derive(Clone)]
pub struct EnemySeekerGenerator {
  pub cooldown: i32,
}

const SEEKER_GENERATOR_INITIAL_FORCE: f32 = 5.0;
const SEEKER_SPAWN_COOLDOWN: i32 = 120;

impl EnemySeekerGenerator {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    physics_rigid_bodies: &RigidBodySet,
  ) -> EnemyDecision {
    let should_spawn_enemy = self.cooldown % SEEKER_SPAWN_COOLDOWN == 0;
    EnemyDecision {
      movement_force: vec_zero(),
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
