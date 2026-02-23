use std::{f32::consts::PI, rc::Rc};

use macroquad::{prelude::rand, rand::RandGenerator};
use rapier2d::{na::Vector2, prelude::*};
use rpds::{List, list};

use crate::{
  balance::BALANCING,
  combat::{Projectile, distance_projection_physics},
  controls::angle_from_vec,
  easing,
  ecs::{ComponentSet, Damageable, Enemy, Entity, EntityHandle},
  load_map::{
    ENEMY_PROJECTILE_INTERACTION_GROUPS, EnemySpawn, EnemySpawnEnemy, RAYCAST_INTERACTION_GROUPS,
  },
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert, UnitConvert2, vec_zero},
};

#[derive(Clone)]
pub struct EnemyDecisionEnemySpawn {
  pub enemy_spawn: EnemySpawn,
  pub initial_force: Vector2<f32>,
}

#[derive(Clone)]
pub struct EnemyDecision {
  pub handle: RigidBodyHandle,
  pub projectiles: Vec<Projectile>,
  pub movement_force: Vector2<f32>,
  pub enemy: Enemy,
  pub enemies_to_spawn: Vec<EnemyDecisionEnemySpawn>,
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
              state: EnemyGoblinState::Slowing(BALANCING.enemies.goblin.slowing_frames),
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
            movement_force: -linvel.normalize() * BALANCING.enemies.goblin.slowing_force(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Goblin(Self {
              state: EnemyGoblinState::Recovering(BALANCING.enemies.goblin.recovering_frames),
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
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Shooting,
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Idle,
            }),
            handle,
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyImpState::Shooting => EnemyDecision {
        handle,
        enemy: Enemy::Imp(Self {
          state: EnemyImpState::ShootingCooldown(
            BALANCING.enemies.imp.shooting_cooldown_initial_frames,
          ),
        }),
        movement_force: vec_zero(),
        enemies_to_spawn: vec![],
        projectiles: {
          let base_projectile = Projectile {
            collider: ColliderBuilder::ball(0.2)
              .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
              .build(),
            damage: BALANCING.enemies.imp.projectile_damage,
            initial_impulse: PhysicsVector::zero(),
            offset: PhysicsVector::zero(),
            force_mod: 0.0,
            component_set: ComponentSet::new(),
            status_effects: list![],
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
              initial_impulse: distance_projection_physics(
                angle,
                BALANCING.enemies.imp.projectile_speed,
              ),
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
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Moving(
                BALANCING.enemies.imp.moving_initial_frames,
                vector![rng.gen_range(-1.0, 1.0), rng.gen_range(-1.0, 1.0)].normalize()
                  * BALANCING.enemies.imp.move_distance,
              ),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
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
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Moving(frames_left - 1, direction),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Imp(Self {
              state: EnemyImpState::Idle,
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
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
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Launching(launch_frames),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Idle,
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyAraneaState::Launching(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Launching(frames_left - 1),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Stopping(BALANCING.enemies.aranea.stopping_frames),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyAraneaState::Stopping(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Stopping(frames_left - 1),
            }),
            movement_force: self_rigid_body.linvel()
              * -1.0
              * BALANCING.enemies.aranea.stopping_force(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Cooldown(BALANCING.enemies.aranea.cooldown_initial_frames),
            }),
            movement_force: self_rigid_body.linvel()
              * -1.0
              * BALANCING.enemies.aranea.stopping_force(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyAraneaState::Cooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Cooldown(frames_left - 1),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Aranea(Self {
              egg_handle: self.egg_handle,
              state: EnemyAraneaState::Shooting,
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyAraneaState::Shooting => {
        let self_translation = self_rigid_body.translation();

        let vector_to_player = player_translation - self_translation;

        let shooting_force = vector_to_player.normalize() * BALANCING.enemies.aranea.shooting_force;

        let projectile = Projectile {
          collider: ColliderBuilder::ball(0.2)
            .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
            .build(),
          damage: BALANCING.enemies.aranea.projectile_damage,
          initial_impulse: PhysicsVector::from_vec(shooting_force),
          offset: PhysicsVector::zero(),
          force_mod: 0.0,
          component_set: ComponentSet::new(),
          status_effects: list![],
        };

        EnemyDecision {
          handle,
          enemy: Enemy::Aranea(Self {
            egg_handle: self.egg_handle,
            state: EnemyAraneaState::Cooldown(BALANCING.enemies.aranea.cooldown_initial_frames),
          }),
          projectiles: vec![projectile],
          movement_force: vec_zero(),
          enemies_to_spawn: vec![],
        }
      }
    }
  }
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
            handle,
            enemy: Enemy::Defender(Self {
              state: EnemyDefenderState::Shooting(0),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::Defender(Self {
              state: EnemyDefenderState::Idle,
            }),
            handle,
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemyDefenderState::Shooting(count) => {
        let ease = easing::ease_in_out_sine() * 2.0 * PI;

        let x = count as f32 / BALANCING.enemies.defender.ease_period;

        let angle = ease.at(x);

        let projectile = |offset: f32| Projectile {
          collider: ColliderBuilder::ball(0.2)
            .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
            .build(),
          damage: 5.0,
          initial_impulse: distance_projection_physics(offset + angle, 0.7),
          offset: PhysicsVector::zero(),
          component_set: ComponentSet::new(),
          force_mod: 0.0,
          status_effects: list![],
        };

        EnemyDecision {
          handle,
          movement_force,
          projectiles: vec![
            projectile(0.0),
            projectile(PI / 2.0),
            projectile(PI),
            projectile(PI + (PI / 2.0)),
          ],
          enemy: Enemy::Defender(EnemyDefender {
            state: EnemyDefenderState::Cooldown(
              count,
              BALANCING.enemies.defender.cooldown_initial_frames,
            ),
          }),
          enemies_to_spawn: vec![],
        }
      }
      EnemyDefenderState::Cooldown(count, frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            handle,
            enemy: Enemy::Defender(EnemyDefender {
              state: EnemyDefenderState::Cooldown(count, frames_left - 1),
            }),
            movement_force,
            projectiles: vec![],
            enemies_to_spawn: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Defender(EnemyDefender {
              state: EnemyDefenderState::Shooting(
                (count + 1) % BALANCING.enemies.defender.ease_period as i32,
              ),
            }),
            movement_force,
            projectiles: vec![],
            enemies_to_spawn: vec![],
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

impl EnemySeekerGenerator {
  pub fn behavior(
    &self,
    handle: RigidBodyHandle,
    player_translation: &Vector2<f32>,
    physics_rigid_bodies: &RigidBodySet,
  ) -> EnemyDecision {
    let should_spawn_enemy = self.cooldown % BALANCING.enemies.seeker_generator.spawn_cooldown == 0;
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
        let initial_force =
          direction_to_player.normalize() * BALANCING.enemies.seeker_generator.initial_force;
        vec![EnemyDecisionEnemySpawn {
          initial_force,
          enemy_spawn: EnemySpawn::new(EnemySpawnEnemy::Seeker, *self_rigid_body.translation()),
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
  pub state: EnemySniperState,
}

const SNIPER_AGGRO_RANGE: f32 = 40.0;
const SNIPER_COOLDOWN_INITIAL_FRAMES: i32 = 200;
const SNIPER_PROJECTILE_DAMAGE: f32 = 20.0;
const SNIPER_SHOOTING_FORCE: f32 = 15.0;
const SNIPER_HOLD_FORCE: f32 = 0.2;

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
    let movement_force = stop_linvel(SNIPER_HOLD_FORCE, self_rigid_body);

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
              state: EnemySniperState::Cooldown(SNIPER_COOLDOWN_INITIAL_FRAMES),
            }),
            movement_force,
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::Sniper(Self {
              state: EnemySniperState::Idle,
            }),
            handle,
            movement_force,
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
        movement_force,
        enemies_to_spawn: vec![],
        projectiles: {
          let collider = ColliderBuilder::ball(0.08)
            .mass(1.0)
            .collision_groups(ENEMY_PROJECTILE_INTERACTION_GROUPS)
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
              status_effects: list![],
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
            movement_force,
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            handle,
            enemy: Enemy::Sniper(Self {
              state: EnemySniperState::Shooting,
            }),
            enemies_to_spawn: vec![],
            movement_force,
            projectiles: vec![],
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

const SNIPER_GENERATOR_GENERATING_INITIAL_FRAMES: i32 = 60;
const SNIPER_GENERATOR_NUM_SNIPERS_GENERATED: i32 = 3;
const SNIPER_GENERATOR_GENERATING_INTERVAL: i32 =
  SNIPER_GENERATOR_GENERATING_INITIAL_FRAMES / SNIPER_GENERATOR_NUM_SNIPERS_GENERATED;
const SNIPER_GENERATOR_COOLDOWN_INITIAL_FRAMES: i32 = 450;
const SNIPER_GENERATOR_GENERATING_INITIAL_FORCE: f32 = 25.0;

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
          SNIPER_AGGRO_RANGE,
          true,
        ) && let Some(reached_parent_handle) = collider_set[reached_handle].parent()
          && reached_parent_handle == player_handle
        {
          EnemyDecision {
            handle,
            enemy: Enemy::SniperGenerator(Self {
              state: EnemySniperGeneratorState::Generating(
                SNIPER_GENERATOR_GENERATING_INITIAL_FRAMES,
              ),
            }),
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::SniperGenerator(Self {
              state: EnemySniperGeneratorState::Idle,
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemySniperGeneratorState::Generating(frames_left) => {
        if frames_left > 0 {
          let enemies_to_spawn = if frames_left % SNIPER_GENERATOR_GENERATING_INTERVAL == 0 {
            let rng_angle = rng.gen_range(0.0, 2.0 * PI);
            let initial_force =
              distance_projection_physics(rng_angle, SNIPER_GENERATOR_GENERATING_INITIAL_FORCE)
                .into_vec();
            vec![EnemyDecisionEnemySpawn {
              initial_force,
              enemy_spawn: EnemySpawn::new(EnemySpawnEnemy::Sniper, *self_translation),
            }]
          } else {
            vec![]
          };
          EnemyDecision {
            enemy: Enemy::SniperGenerator(Self {
              state: EnemySniperGeneratorState::Generating(frames_left - 1),
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn,
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::SniperGenerator(Self {
              state: EnemySniperGeneratorState::Cooldown(SNIPER_GENERATOR_COOLDOWN_INITIAL_FRAMES),
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
      EnemySniperGeneratorState::Cooldown(frames_left) => {
        if frames_left > 0 {
          EnemyDecision {
            enemy: Enemy::SniperGenerator(Self {
              state: EnemySniperGeneratorState::Cooldown(frames_left - 1),
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        } else {
          EnemyDecision {
            enemy: Enemy::SniperGenerator(Self {
              state: EnemySniperGeneratorState::Idle,
            }),
            handle,
            movement_force: vec_zero(),
            enemies_to_spawn: vec![],
            projectiles: vec![],
          }
        }
      }
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
