use std::{f32::consts::PI, rc::Rc};

use macroquad::{prelude::rand, rand::RandGenerator};
use rapier2d::{na::Vector2, prelude::*};
use rpds::{List, list};

use crate::{
  balance::BALANCING,
  combat::{Beam, Projectile, WeaponOutput, WeaponOutputKind, distance_projection_physics},
  controls::angle_from_vec,
  easing,
  ecs::{Damageable, Enemy, Entity, EntityHandle, StatusEffect},
  load_map::{
    ENEMY_PROJECTILE_INTERACTION_GROUPS, EnemySpawn, EnemySpawnEnemy, RAYCAST_INTERACTION_GROUPS,
  },
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert2, vec_zero},
};

#[derive(Clone)]
pub struct EnemyDecisionEnemySpawn {
  pub enemy_spawn: EnemySpawn,
  pub initial_force: Vector2<f32>,
}

#[derive(Clone)]
pub struct EnemyDecision {
  pub handle: RigidBodyHandle,
  pub weapon_outputs: Vec<WeaponOutput>,
  pub movement_force: Vector2<f32>,
  pub angvel: Option<f32>,
  pub enemy: Enemy,
  pub enemies_to_spawn: Vec<EnemyDecisionEnemySpawn>,
}

impl EnemyDecision {
  pub fn default(handle: RigidBodyHandle, enemy: Enemy) -> Self {
    Self {
      handle,
      enemy,
      angvel: None,
      enemies_to_spawn: vec![],
      movement_force: vec_zero(),
      weapon_outputs: vec![],
    }
  }
}

#[derive(Clone)]
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

  fn update(
    &self,
    _: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>> {
    Rc::new(self.clone())
  }

  fn fixed_update(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    let rng = rand::RandGenerator::new();
    rng.srand(physics_system.frame_count as u64);

    let query_pipeline = physics_system.broad_phase.as_query_pipeline(
      physics_system.narrow_phase.query_dispatcher(),
      &physics_system.rigid_body_set,
      &physics_system.collider_set,
      QueryFilter::default().groups(RAYCAST_INTERACTION_GROUPS),
    );

    let enemy_behavior = enemy_behavior_generator(
      physics_system.player_handle,
      &physics_system.rigid_body_set,
      &physics_system.collider_set,
      &physics_system.narrow_phase,
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
  rigid_body_set: &RigidBodySet,
  collider_set: &ColliderSet,
  narrow_phase: &NarrowPhase,
  query_pipeline: &QueryPipeline,
  rng: &RandGenerator,
) -> impl Fn((&EntityHandle, &Rc<Entity>)) -> Option<EnemyDecision> {
  let player_translation = rigid_body_set[player_handle].translation();

  move |(&handle, entity)| {
    if let EntityHandle::RigidBody(handle) = handle {
      entity
        .components
        .get::<Enemy>()
        .map(|enemy| match enemy.as_ref() {
          Enemy::Goblin(goblin) => goblin.behavior(
            handle,
            player_handle,
            rigid_body_set,
            collider_set,
            query_pipeline,
          ),
          Enemy::Imp(imp) => imp.behavior(
            handle,
            player_handle,
            rigid_body_set,
            collider_set,
            query_pipeline,
            rng,
          ),
          Enemy::Aranea(aranea) => {
            let damageable = entity.components.get::<Damageable>().unwrap();
            aranea.behavior(
              handle,
              damageable.as_ref(),
              player_translation,
              collider_set,
              rigid_body_set,
              narrow_phase,
            )
          }
          Enemy::Defender(defender) => defender.behavior(
            handle,
            player_handle,
            player_translation,
            collider_set,
            rigid_body_set,
            query_pipeline,
          ),
          Enemy::Seeker(seeker) => seeker.behavior(handle, player_translation, rigid_body_set),
          Enemy::SeekerGenerator(seeker_generator) => {
            seeker_generator.behavior(handle, player_translation, rigid_body_set)
          }
          Enemy::Sniper(sniper) => sniper.behavior(
            handle,
            player_handle,
            collider_set,
            rigid_body_set,
            query_pipeline,
          ),
          Enemy::SniperGenerator(sniper_generator) => sniper_generator.behavior(
            handle,
            player_handle,
            collider_set,
            rigid_body_set,
            query_pipeline,
            rng,
          ),
          Enemy::LaserGate(laser_gate) => laser_gate.behavior(handle, rigid_body_set),
        })
    } else {
      None
    }
  }
}

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
          BALANCING.enemies.goblin.aggro_range,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          let self_translation = self_rigid_body.translation();
          let vector_to_player = player_translation - self_translation;

          let movement_force = vector_to_player.normalize() * BALANCING.enemies.goblin.lunge_force;

          let lunge_distance = vector_to_player
            .magnitude()
            .min(BALANCING.enemies.goblin.max_lunge_distance);

          let lunge_frames = (lunge_distance
            / (BALANCING.enemies.goblin.lunge_force / self_rigid_body.mass())
            * 60.0) as i32;

          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Lunging(lunge_frames),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Idle,
              }),
            )
          }
        }
      }
      EnemyGoblinState::Lunging(remaining_frames) => {
        if remaining_frames > 0 {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Lunging(remaining_frames - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Slowing(BALANCING.enemies.goblin.slowing_frames),
              }),
            )
          }
        }
      }
      EnemyGoblinState::Slowing(remaining_frames) => {
        let linvel = rigid_body_set[handle].linvel();

        if remaining_frames > 0 && linvel.magnitude() > 0.0 {
          EnemyDecision {
            movement_force: -linvel.normalize() * BALANCING.enemies.goblin.slowing_force(),
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Slowing(remaining_frames - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Recovering(BALANCING.enemies.goblin.recovering_frames),
              }),
            )
          }
        }
      }
      EnemyGoblinState::Recovering(remaining_frames) => {
        if remaining_frames > 0 {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Recovering(remaining_frames - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Goblin(Self {
                state: EnemyGoblinState::Idle,
              }),
            )
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
  Moving(i32, Vector2<f32>),
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
    let self_rigid_body = &rigid_body_set[handle];
    let movement_force = stop_linvel(BALANCING.enemies.imp.move_force, self_rigid_body);
    match self.state {
      EnemyImpState::Idle => {
        let self_translation = self_rigid_body.translation();

        let direction_to_player = player_translation - self_translation;

        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          BALANCING.enemies.imp.aggro_range,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Imp(Self {
                state: EnemyImpState::Shooting,
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Imp(Self {
                state: EnemyImpState::Idle,
              }),
            )
          }
        }
      }
      EnemyImpState::Shooting => EnemyDecision {
        weapon_outputs: {
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
            .map(|&angle| WeaponOutput {
              damage: BALANCING.enemies.imp.projectile_damage,
              ..WeaponOutput::default(WeaponOutputKind::Projectile(Projectile {
                initial_impulse: distance_projection_physics(
                  angle,
                  BALANCING.enemies.imp.projectile_speed,
                ),
                ..Projectile::default(
                  ColliderBuilder::ball(0.2)
                    .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
                    .build(),
                )
              }))
            })
            .collect()
        },
        ..EnemyDecision::default(
          handle,
          Enemy::Imp(Self {
            state: EnemyImpState::ShootingCooldown(
              BALANCING.enemies.imp.shooting_cooldown_initial_frames,
            ),
          }),
        )
      },
      EnemyImpState::ShootingCooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Imp(Self {
                state: EnemyImpState::ShootingCooldown(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Imp(Self {
                state: EnemyImpState::Moving(
                  BALANCING.enemies.imp.moving_initial_frames,
                  vector![rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)].normalize()
                    * BALANCING.enemies.imp.move_distance,
                ),
              }),
            )
          }
        }
      }
      EnemyImpState::Moving(frames_left, direction) => {
        if frames_left > 0 {
          let ease = easing::ease_in_out_sine_ddt2()
            * (direction / BALANCING.enemies.imp.moving_initial_frames as f32);
          let x = 1.0 - frames_left as f32 / BALANCING.enemies.imp.moving_initial_frames as f32;
          let movement_force = ease.at(x);

          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Imp(Self {
                state: EnemyImpState::Moving(frames_left - 1, direction),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Imp(Self {
                state: EnemyImpState::Idle,
              }),
            )
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub enum EnemyAraneaState {
  Idle,
  Launching(i32),
  Stopping(i32),
  Shooting,
  Cooldown(i32),
}

#[derive(Clone)]
pub struct EnemyAranea {
  state: EnemyAraneaState,
  egg_handle: ColliderHandle,
}

impl EnemyAranea {
  pub fn new(egg_handle: ColliderHandle) -> Self {
    Self {
      state: EnemyAraneaState::Idle,
      egg_handle,
    }
  }

  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    damageable: &Damageable,
    player_translation: &Vector2<f32>,
    collider_set: &ColliderSet,
    rigid_body_set: &RigidBodySet,
    narrow_phase: &NarrowPhase,
  ) -> EnemyDecision {
    let self_rigid_body = &rigid_body_set[handle];
    let movement_force = stop_linvel(BALANCING.enemies.aranea.hold_force, self_rigid_body);
    match self.state {
      EnemyAraneaState::Idle => {
        if narrow_phase
          .intersection_pairs_with(self.egg_handle)
          .any(|(_, _, colliding)| colliding)
          || damageable.health < damageable.max_health
        {
          let self_translation = self_rigid_body.translation();

          let egg_translation = collider_set[self.egg_handle].translation();
          let vector_to_egg = egg_translation - self_translation;

          let movement_force = vector_to_egg.normalize() * BALANCING.enemies.aranea.launch_force;

          let launch_frames = (vector_to_egg.magnitude()
            / (movement_force.magnitude() / self_rigid_body.mass())
            * 60.0) as i32;

          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Launching(launch_frames),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Idle,
              }),
            )
          }
        }
      }
      EnemyAraneaState::Launching(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Launching(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Stopping(BALANCING.enemies.aranea.stopping_frames),
              }),
            )
          }
        }
      }
      EnemyAraneaState::Stopping(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            movement_force: self_rigid_body.linvel()
              * -1.0
              * BALANCING.enemies.aranea.stopping_force(),
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Stopping(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force: self_rigid_body.linvel()
              * -1.0
              * BALANCING.enemies.aranea.stopping_force(),
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Cooldown(BALANCING.enemies.aranea.cooldown_initial_frames),
              }),
            )
          }
        }
      }
      EnemyAraneaState::Cooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Cooldown(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Aranea(Self {
                egg_handle: self.egg_handle,
                state: EnemyAraneaState::Shooting,
              }),
            )
          }
        }
      }
      EnemyAraneaState::Shooting => {
        let self_translation = self_rigid_body.translation();

        let vector_to_player = player_translation - self_translation;

        let shooting_force = vector_to_player.normalize() * BALANCING.enemies.aranea.shooting_force;

        let weapon_output = WeaponOutput {
          damage: BALANCING.enemies.aranea.projectile_damage,
          ..WeaponOutput::default(WeaponOutputKind::Projectile(Projectile {
            initial_impulse: PhysicsVector::from_vec(shooting_force),
            ..Projectile::default(
              ColliderBuilder::ball(0.2)
                .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
                .build(),
            )
          }))
        };

        EnemyDecision {
          weapon_outputs: vec![weapon_output],
          ..EnemyDecision::default(
            handle,
            Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Cooldown(BALANCING.enemies.aranea.cooldown_initial_frames),
            }),
          )
        }
      }
    }
  }
}

pub mod aranea_queen {
  #[derive(Clone)]
  pub enum FirstLaunchSubstate {
    Launching(i32),
    Stopping(i32),
    Spraying(i32),
  }

  #[derive(Clone)]
  pub enum Phase1Substate {}

  #[derive(Clone)]
  pub enum State {
    Idle,
    FirstLaunch(FirstLaunchSubstate),
    Phase1(Phase1Substate),
  }

  #[derive(Clone)]
  pub struct EnemyAraneaQueen {
    state: State,
  }

  impl EnemyAraneaQueen {}
}

#[derive(Clone)]
pub enum EnemyDefenderState {
  Idle,
  Shooting(i32),
  Cooldown(i32, i32),
}

#[derive(Clone)]
pub struct EnemyDefender {
  state: EnemyDefenderState,
}

impl EnemyDefender {
  pub fn new() -> EnemyDefender {
    Self {
      state: EnemyDefenderState::Idle,
    }
  }

  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    collider_set: &ColliderSet,
    rigid_body_set: &RigidBodySet,
    query_pipeline: &QueryPipeline,
  ) -> EnemyDecision {
    let self_rigid_body = &rigid_body_set[handle];

    let movement_force = stop_linvel(BALANCING.enemies.defender.hold_force, self_rigid_body);

    match self.state {
      EnemyDefenderState::Idle => {
        let self_translation = self_rigid_body.translation();

        let direction_to_player = player_translation - self_translation;

        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          BALANCING.enemies.defender.aggro_range,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Defender(Self {
                state: EnemyDefenderState::Shooting(0),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Defender(Self {
                state: EnemyDefenderState::Idle,
              }),
            )
          }
        }
      }
      EnemyDefenderState::Shooting(count) => {
        let ease = easing::ease_in_out_sine() * 2.0 * PI;

        let x = count as f32 / BALANCING.enemies.defender.ease_period;

        let angle = ease.at(x);

        let weapon_output = |offset: f32| WeaponOutput {
          damage: BALANCING.enemies.defender.damage,
          ..WeaponOutput::default(WeaponOutputKind::Projectile(Projectile {
            initial_impulse: distance_projection_physics(offset + angle, 0.7),
            ..Projectile::default(
              ColliderBuilder::ball(0.2)
                .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
                .build(),
            )
          }))
        };

        EnemyDecision {
          movement_force,
          weapon_outputs: vec![
            weapon_output(0.0),
            weapon_output(PI / 2.0),
            weapon_output(PI),
            weapon_output(PI + (PI / 2.0)),
          ],
          ..EnemyDecision::default(
            handle,
            Enemy::Defender(EnemyDefender {
              state: EnemyDefenderState::Cooldown(
                count,
                BALANCING.enemies.defender.cooldown_initial_frames,
              ),
            }),
          )
        }
      }
      EnemyDefenderState::Cooldown(count, frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Defender(EnemyDefender {
                state: EnemyDefenderState::Cooldown(count, frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Defender(EnemyDefender {
                state: EnemyDefenderState::Shooting(
                  (count + 1) % BALANCING.enemies.defender.ease_period as i32,
                ),
              }),
            )
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub struct EnemySeeker;

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

      (if velocity_towards_player.magnitude() >= BALANCING.enemies.seeker.speed_cap {
        vec_zero()
      } else {
        direction_to_player.normalize() * BALANCING.enemies.seeker.speed
      }) - velocity_away_from_player.normalize() * BALANCING.enemies.seeker.speed * 0.3
    };
    EnemyDecision {
      movement_force,
      ..EnemyDecision::default(handle, Enemy::Seeker(Self))
    }
  }
}

#[derive(Clone)]
pub struct EnemySeekerGenerator {
  pub cooldown: i32,
}

impl EnemySeekerGenerator {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    physics_rigid_bodies: &RigidBodySet,
  ) -> EnemyDecision {
    let should_spawn_enemy = self.cooldown % BALANCING.enemies.seeker_generator.spawn_cooldown == 0;
    EnemyDecision {
      enemies_to_spawn: if should_spawn_enemy {
        let self_rigid_body = &physics_rigid_bodies[handle];
        let direction_to_player = player_translation - self_rigid_body.translation();
        let initial_force =
          direction_to_player.normalize() * BALANCING.enemies.seeker_generator.initial_force;
        vec![EnemyDecisionEnemySpawn {
          initial_force,
          enemy_spawn: EnemySpawn::new(
            EnemySpawnEnemy::Seeker,
            *self_rigid_body.translation(),
            0.0,
            None,
          ),
        }]
      } else {
        vec![]
      },
      ..EnemyDecision::default(
        handle,
        Enemy::SeekerGenerator(Self {
          cooldown: self.cooldown - 1,
        }),
      )
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
  pub state: EnemySniperState,
}

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
    let movement_force = stop_linvel(BALANCING.enemies.sniper.hold_force, self_rigid_body);

    match self.state {
      EnemySniperState::Idle => {
        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          BALANCING.enemies.sniper.aggro_range,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Sniper(Self {
                state: EnemySniperState::Cooldown(BALANCING.enemies.sniper.cooldown_initial_frames),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Sniper(Self {
                state: EnemySniperState::Idle,
              }),
            )
          }
        }
      }
      EnemySniperState::Shooting => EnemyDecision {
        movement_force,
        weapon_outputs: {
          let collider = ColliderBuilder::ball(0.08)
            .mass(1.0)
            .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
            .build();

          let player_relative_velocity = *player_rigid_body.linvel() - *self_rigid_body.linvel();

          if let Some(lead_direction) = calculate_lead_direction(
            direction_to_player,
            player_relative_velocity,
            BALANCING.enemies.sniper.shooting_force / collider.mass(),
          ) {
            vec![WeaponOutput {
              damage: BALANCING.enemies.sniper.projectile_damage,
              ..WeaponOutput::default(WeaponOutputKind::Projectile(Projectile {
                initial_impulse: PhysicsVector::from_vec(
                  lead_direction * BALANCING.enemies.sniper.shooting_force,
                ),
                ..Projectile::default(collider)
              }))
            }]
          } else {
            vec![]
          }
        },
        ..EnemyDecision::default(
          handle,
          Enemy::Sniper(Self {
            state: EnemySniperState::Cooldown(BALANCING.enemies.sniper.cooldown_initial_frames),
          }),
        )
      },
      EnemySniperState::Cooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Sniper(Self {
                state: EnemySniperState::Cooldown(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            movement_force,
            ..EnemyDecision::default(
              handle,
              Enemy::Sniper(Self {
                state: EnemySniperState::Shooting,
              }),
            )
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub enum EnemySniperGeneratorState {
  Idle,
  Generating(i32),
  Cooldown(i32),
}

#[derive(Clone)]
pub struct EnemySniperGenerator {
  state: EnemySniperGeneratorState,
}

impl EnemySniperGenerator {
  pub fn new() -> Self {
    Self {
      state: EnemySniperGeneratorState::Idle,
    }
  }

  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_handle: RigidBodyHandle,
    collider_set: &ColliderSet,
    rigid_body_set: &RigidBodySet,
    query_pipeline: &QueryPipeline,
    rng: &RandGenerator,
  ) -> EnemyDecision {
    let player_rigid_body = &rigid_body_set[player_handle];
    let player_translation = player_rigid_body.translation();
    let self_rigid_body = &rigid_body_set[handle];
    let self_translation = self_rigid_body.translation();
    let direction_to_player = player_translation - self_translation;

    match self.state {
      EnemySniperGeneratorState::Idle => {
        if let Some((reached_handle, _)) = query_pipeline.cast_ray(
          &Ray::new((*self_translation).into(), direction_to_player),
          BALANCING.enemies.sniper.aggro_range,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::SniperGenerator(Self {
                state: EnemySniperGeneratorState::Generating(
                  BALANCING.enemies.sniper_generator.generating_initial_frames,
                ),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::SniperGenerator(Self {
                state: EnemySniperGeneratorState::Idle,
              }),
            )
          }
        }
      }
      EnemySniperGeneratorState::Generating(frames_left) => {
        if frames_left > 0 {
          let enemies_to_spawn =
            if frames_left % BALANCING.enemies.sniper_generator.generating_interval() == 0 {
              let rng_angle = rng.gen_range(0.0, 2.0 * PI);
              let initial_force = distance_projection_physics(
                rng_angle,
                BALANCING.enemies.sniper_generator.generating_initial_force,
              )
              .into_vec();
              vec![EnemyDecisionEnemySpawn {
                initial_force,
                enemy_spawn: EnemySpawn::new(EnemySpawnEnemy::Sniper, *self_translation, 0.0, None),
              }]
            } else {
              vec![]
            };
          EnemyDecision {
            enemies_to_spawn,
            ..EnemyDecision::default(
              handle,
              Enemy::SniperGenerator(Self {
                state: EnemySniperGeneratorState::Generating(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::SniperGenerator(Self {
                state: EnemySniperGeneratorState::Cooldown(
                  BALANCING.enemies.sniper_generator.cooldown_initial_frames,
                ),
              }),
            )
          }
        }
      }
      EnemySniperGeneratorState::Cooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::SniperGenerator(Self {
                state: EnemySniperGeneratorState::Cooldown(frames_left - 1),
              }),
            )
          }
        } else {
          EnemyDecision {
            ..EnemyDecision::default(
              handle,
              Enemy::SniperGenerator(Self {
                state: EnemySniperGeneratorState::Idle,
              }),
            )
          }
        }
      }
    }
  }
}

#[derive(Clone)]
pub struct EnemyLaserGate {
  target_rotation_angle: f32,
  parent_enemy: Option<Rc<Enemy>>,
}

impl EnemyLaserGate {
  pub fn new(rigid_body: &RigidBody) -> Self {
    Self {
      target_rotation_angle: rigid_body.rotation().angle(),
      parent_enemy: None,
    }
  }

  pub fn behavior(&self, handle: RigidBodyHandle, rigid_body_set: &RigidBodySet) -> EnemyDecision {
    let weapon_output = WeaponOutput {
      damage: BALANCING.enemies.laser_gate.damage,
      status_effects: list![(
        StatusEffect::Deteriorate,
        BALANCING.enemies.laser_gate.deteriorate_apply_amount
      )],
      ..WeaponOutput::default(WeaponOutputKind::Beam(Beam {
        angle: 0.0,
        thickness: BALANCING.enemies.laser_gate.beam_thickness,
      }))
    };

    let rigid_body = &rigid_body_set[handle];

    EnemyDecision {
      weapon_outputs: vec![weapon_output],
      movement_force: stop_linvel(100.0, rigid_body) * rigid_body.mass(),
      angvel: Some(rotate_to_target(
        1.0,
        rigid_body,
        self.target_rotation_angle,
      )),
      ..EnemyDecision::default(handle, Enemy::LaserGate(self.clone()))
    }
  }
}

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

pub fn stop_linvel(move_speed: f32, rigid_body: &RigidBody) -> Vector2<f32> {
  if rigid_body.linvel().magnitude() > move_speed {
    -rigid_body.linvel().normalize() * move_speed
  } else {
    -rigid_body.linvel()
  }
}

pub fn rotate_to_target(force_mod: f32, rigid_body: &RigidBody, target_angle: f32) -> f32 {
  let target_angle = target_angle % (2.0 * PI);
  let current_angle = rigid_body.rotation().angle() % (2.0 * PI);
  let current_angle = if target_angle - current_angle > PI {
    current_angle + (2.0 * PI)
  } else if target_angle - current_angle < -PI {
    current_angle - (2.0 * PI)
  } else {
    current_angle
  };

  let angular_difference = target_angle - current_angle;
  println!("{angular_difference}");

  if angular_difference.abs() < force_mod / 60.0 {
    0.0
  } else if angular_difference < 0.0 {
    -force_mod
  } else {
    force_mod
  }
}
