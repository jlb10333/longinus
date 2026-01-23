use std::{f32::consts::PI, rc::Rc};

use macroquad::{prelude::rand, rand::RandGenerator};
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  combat::{Projectile, distance_projection_physics},
  controls::angle_from_vec,
  ecs::{ComponentSet, Enemy, Entity, EntityHandle},
  load_map::{
    COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALL,
    ENEMY_INTERACTION_GROUPS, EnemySpawn, MapEnemyName,
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

    let query_pipeline = physics_system.broad_phase.as_query_pipeline(
      physics_system.narrow_phase.query_dispatcher(),
      &physics_system.rigid_body_set,
      &physics_system.collider_set,
      QueryFilter::default().groups(ENEMY_INTERACTION_GROUPS),
    );

    let enemy_behavior = enemy_behavior_generator(
      physics_system.player_handle,
      &physics_system.rigid_body_set,
      &physics_system.collider_set,
      &query_pipeline,
      &rng,
    );

    let decisions = physics_system
      .entities
      .iter()
      .filter_map(enemy_behavior)
      .collect::<Vec<_>>();

    Rc::new(Self { decisions })
  }
}

fn enemy_behavior_generator(
  player_handle: RigidBodyHandle,
  physics_rigid_bodies: &RigidBodySet,
  physics_colliders: &ColliderSet,
  query_pipeline: &QueryPipeline,
  rng: &RandGenerator,
) -> impl Fn((&EntityHandle, &Rc<Entity>)) -> Option<EnemyDecision> {
  let player_translation = physics_rigid_bodies[player_handle].translation();

  move |(&handle, entity)| {
    if let EntityHandle::RigidBody(rigid_body_handle) = handle {
      entity
        .components
        .get::<Enemy>()
        .map(|enemy| match enemy.as_ref() {
          Enemy::Goblin(goblin) => goblin.behavior(
            rigid_body_handle,
            player_handle,
            physics_rigid_bodies,
            physics_colliders,
            query_pipeline,
          ),
          Enemy::Imp(imp) => imp.behavior(
            rigid_body_handle,
            player_handle,
            physics_rigid_bodies,
            physics_colliders,
            query_pipeline,
            rng,
          ),
          Enemy::Defender(defender) => defender.behavior(rigid_body_handle),
          Enemy::Seeker(seeker) => {
            seeker.behavior(rigid_body_handle, player_translation, physics_rigid_bodies)
          }
          Enemy::SeekerGenerator(seeker_generator) => {
            seeker_generator.behavior(rigid_body_handle, player_translation, physics_rigid_bodies)
          }
          Enemy::Sniper(sniper) => sniper.behavior(
            rigid_body_handle,
            player_handle,
            physics_colliders,
            physics_rigid_bodies,
            query_pipeline,
          ),
          Enemy::SniperGenerator(_) => todo!(),
        })
    } else {
      None
    }
  }
}

const ENEMY_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_ENEMY_PROJECTILE,
  filter: COLLISION_GROUP_PLAYER.union(COLLISION_GROUP_WALL),
  test_mode: InteractionTestMode::And,
};

#[derive(Clone, Copy)]
pub enum EnemyGoblinState {
  Idle,
  Lunging(i32),
  Slowing(i32),
  Recovering(i32),
}

impl EnemyGoblinState {
  pub fn initial() -> Self {
    Self::Idle
  }
}

#[derive(Clone, Copy)]
pub struct EnemyGoblin {
  pub state: EnemyGoblinState,
}

const GOBLIN_AGGRO_RANGE: f32 = 20.0;
const GOBLIN_LUNGE_FORCE: f32 = 9.0;
const GOBLIN_SLOWING_FRAMES: i32 = 70;
const GOBLIN_SLOWING_FORCE: f32 = GOBLIN_LUNGE_FORCE / GOBLIN_SLOWING_FRAMES as f32;
const GOBLIN_RECOVERING_FRAMES: i32 = 50;

impl EnemyGoblin {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_handle: RigidBodyHandle,
    rigid_body_set: &RigidBodySet,
    collider_set: &ColliderSet,
    query_pipeline: &QueryPipeline,
  ) -> EnemyDecision {
    match self.state {
      EnemyGoblinState::Idle => {
        let player_translation = rigid_body_set[player_handle].translation();
        let self_rigid_body = &rigid_body_set[handle];

        let self_translation = self_rigid_body.translation();

        let direction_to_player = player_translation - self_translation;

        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          GOBLIN_AGGRO_RANGE,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          let self_translation = self_rigid_body.translation();
          let vector_to_player = player_translation - self_translation;

          let movement_force = vector_to_player.normalize() * GOBLIN_LUNGE_FORCE;

          let lunge_frames = (vector_to_player.magnitude()
            / (movement_force.magnitude() / self_rigid_body.mass())
            * 60.0) as i32;

          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Lunging(lunge_frames),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Idle,
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyGoblinState::Lunging(remaining_frames) => {
        if remaining_frames > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Lunging(remaining_frames - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Slowing(GOBLIN_SLOWING_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyGoblinState::Slowing(remaining_frames) => {
        let linvel = rigid_body_set[handle].linvel();

        if remaining_frames > 0 && linvel.magnitude() > 0.0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Slowing(remaining_frames - 1),
            }),
            movement_force: -linvel.normalize() * GOBLIN_SLOWING_FORCE,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Recovering(GOBLIN_RECOVERING_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyGoblinState::Recovering(remaining_frames) => {
        if remaining_frames > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Recovering(remaining_frames - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Idle,
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub enum EnemyImpState {
  Idle,
  Shooting,
  ShootingCooldown(i32),
  Cruising(i32),
  Accelerating(i32, Vector2<f32>),
  Decelerating(i32),
}

impl EnemyImpState {
  pub fn initial() -> Self {
    Self::Idle
  }
}

#[derive(Clone)]
pub struct EnemyImp {
  pub state: EnemyImpState,
}

const IMP_AGGRO_RANGE: f32 = 20.0;
const IMP_STATE_CRUISING_INITIAL_FRAMES: i32 = 70;
const IMP_STATE_SHOOTING_COOLDOWN_INITIAL_FRAMES: i32 = 50;
const IMP_STATE_ACCELERATING_INITIAL_FRAMES: i32 = 10;
const IMP_STATE_DECELERATING_INITIAL_FRAMES: i32 = 10;

const IMP_MOVE_FORCE: f32 = 0.2;
const IMP_PROJECTILE_SPEED: f32 = 0.7;
const IMP_PROJECTILE_DAMAGE: f32 = 5.0;

impl EnemyImp {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_handle: RigidBodyHandle,
    rigid_body_set: &RigidBodySet,
    collider_set: &ColliderSet,
    query_pipeline: &QueryPipeline,
    rng: &RandGenerator,
  ) -> EnemyDecision {
    let player_translation = rigid_body_set[player_handle].translation();
    match self.state {
      EnemyImpState::Idle => {
        let self_rigid_body = &rigid_body_set[handle];

        let self_translation = self_rigid_body.translation();

        let direction_to_player = player_translation - self_translation;

        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          IMP_AGGRO_RANGE,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Shooting,
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Idle,
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyImpState::Shooting => EnemyDecision {
        handle,
        enemy: Enemy::Imp(Self {
          state: EnemyImpState::ShootingCooldown(IMP_STATE_SHOOTING_COOLDOWN_INITIAL_FRAMES),
        }),
        movement_force: vec_zero(),
        enemies_to_spawn: vec![],
        projectiles: {
          let base_projectile = Projectile {
            collider: ColliderBuilder::ball(0.2)
              .collision_groups(ENEMY_GROUPS)
              .build(),
            damage: IMP_PROJECTILE_DAMAGE,
            initial_impulse: PhysicsVector::zero(),
            offset: PhysicsVector::zero(),
            force_mod: 0.0,
            component_set: ComponentSet::new(),
          };

          let base_impulse_angle = angle_from_vec(PhysicsVector::from_vec(
            player_translation - rigid_body_set[handle].translation(),
          ));

          let impulses = [
            base_impulse_angle,
            base_impulse_angle + PI / 6.0,
            base_impulse_angle - PI / 6.0,
          ];

          impulses
            .iter()
            .map(|&angle| Projectile {
              initial_impulse: distance_projection_physics(angle, IMP_PROJECTILE_SPEED),
              ..base_projectile.clone()
            })
            .collect()
        },
      },
      EnemyImpState::ShootingCooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::ShootingCooldown(frames_left - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Accelerating(
                IMP_STATE_ACCELERATING_INITIAL_FRAMES,
                vector![rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)],
              ),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyImpState::Cruising(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Cruising(frames_left - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Decelerating(IMP_STATE_DECELERATING_INITIAL_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyImpState::Accelerating(frames_left, direction) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Accelerating(frames_left - 1, direction),
            }),
            movement_force: direction.normalize() * IMP_MOVE_FORCE,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Cruising(IMP_STATE_CRUISING_INITIAL_FRAMES),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyImpState::Decelerating(frames_left) => {
        let linvel = rigid_body_set[handle].linvel();

        if frames_left > 0 && linvel.magnitude() > 0.0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Decelerating(frames_left - 1),
            }),
            movement_force: -linvel.normalize() * IMP_MOVE_FORCE,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Shooting,
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
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
          enemy_spawn: EnemySpawn::new(MapEnemyName::Seeker, *self_rigid_body.translation()),
        }]
      } else {
        vec![]
      },
    }
  }
}

#[derive(Clone)]
pub enum EnemySniperState {
  Idle,
  Shooting,
  Cooldown(i32),
}

#[derive(Clone)]
pub struct EnemySniper {
  state: EnemySniperState,
}

const SNIPER_AGGRO_RANGE: f32 = 40.0;
const SNIPER_COOLDOWN_INITIAL_FRAMES: i32 = 200;
const SNIPER_PROJECTILE_DAMAGE: f32 = 15.0;
const SNIPER_SHOOTING_FORCE: f32 = 0.2;

impl EnemySniper {
  pub fn new() -> Self {
    Self {
      state: EnemySniperState::Idle,
    }
  }

  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_handle: RigidBodyHandle,
    collider_set: &ColliderSet,
    rigid_body_set: &RigidBodySet,
    query_pipeline: &QueryPipeline,
  ) -> EnemyDecision {
    let player_rigid_body = &rigid_body_set[player_handle];
    let player_translation = player_rigid_body.translation();
    let self_rigid_body = &rigid_body_set[handle];
    let self_translation = self_rigid_body.translation();
    let direction_to_player = player_translation - self_translation;

    match self.state {
      EnemySniperState::Idle => {
        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          SNIPER_AGGRO_RANGE,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            handle,
            enemy: Enemy::Sniper(Self {
              state: EnemySniperState::Shooting,
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::Sniper(Self {
              state: EnemySniperState::Idle,
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemySniperState::Shooting => EnemyDecision {
        handle,
        enemy: Enemy::Sniper(Self {
          state: EnemySniperState::Cooldown(SNIPER_COOLDOWN_INITIAL_FRAMES),
        }),
        movement_force: vec_zero(),
        enemies_to_spawn: vec![],
        projectiles: {
          let collider = ColliderBuilder::ball(0.08)
            .collision_groups(ENEMY_GROUPS)
            .build();

          let player_relative_velocity = *player_rigid_body.linvel() - *self_rigid_body.linvel();

          if let Some(lead_direction) = calculate_lead_direction(
            direction_to_player,
            player_relative_velocity,
            SNIPER_SHOOTING_FORCE / collider.mass(),
          ) {
            vec![Projectile {
              collider,
              damage: SNIPER_PROJECTILE_DAMAGE,
              initial_impulse: PhysicsVector::from_vec(lead_direction * SNIPER_SHOOTING_FORCE),
              offset: PhysicsVector::zero(),
              force_mod: 0.0,
              component_set: ComponentSet::new(),
            }]
          } else {
            vec![]
          }
        },
      },
      EnemySniperState::Cooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Sniper(Self {
              state: EnemySniperState::Cooldown(frames_left - 1),
            }),
            enemies_to_spawn: vec![],
            movement_force: vec_zero(),
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Sniper(Self {
              state: EnemySniperState::Idle,
            }),
            enemies_to_spawn: vec![],
            movement_force: vec_zero(),
            projectiles: vec![],
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub struct EnemySniperGenerator;

pub fn calculate_lead_direction(
  target_relative_position: Vector2<f32>,
  target_relative_velocity: Vector2<f32>,
  bullet_speed: f32,
) -> Option<Vector2<f32>> {
  let a = target_relative_velocity.dot(&target_relative_velocity) - bullet_speed.powi(2);
  let b = 2.0 * target_relative_position.dot(&target_relative_velocity);
  let c = target_relative_position.dot(&target_relative_position);

  let discriminant = b * b - 4.0 * a * c;

  if discriminant < 0.0 {
    return None;
  }

  let sqrt_disc = discriminant.sqrt();
  let delta_times = [(-b + sqrt_disc) / (2.0 * a), (-b - sqrt_disc) / (2.0 * a)];

  let delta_time = delta_times
    .iter()
    .filter(|&&dt| dt > 0.0)
    .reduce(|dt1, dt2| if dt1 < dt2 { dt1 } else { dt2 });

  delta_time.map(|&delta_time| {
    (target_relative_position + (target_relative_velocity * delta_time))
      / (bullet_speed * delta_time)
  })
}
