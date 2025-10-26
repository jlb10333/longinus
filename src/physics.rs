use rapier2d::prelude::*;
use std::{cell::RefCell, ops::Deref, rc::Rc};

use crate::{
  combat::CombatSystem,
  controls::ControlsSystem,
  ecs::{ComponentSet, Damageable, Damager, DestroyOnCollision, Enemy, Entity},
  enemy::{EnemyDecision, EnemySystem},
  f::Monad,
  load_map::{
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_WALL, MapSystem, MapTile,
  },
  system::System,
  units::UnitConvert2,
};

pub struct PhysicsSystem {
  pub rigid_body_set: RigidBodySet,
  pub collider_set: ColliderSet,
  pub integration_parameters: IntegrationParameters,
  pub physics_pipeline: Rc<RefCell<PhysicsPipeline>>,
  pub island_manager: IslandManager,
  pub broad_phase: DefaultBroadPhase,
  pub narrow_phase: NarrowPhase,
  pub impulse_joint_set: ImpulseJointSet,
  pub multibody_joint_set: MultibodyJointSet,
  pub ccd_solver: CCDSolver,
  pub player_handle: RigidBodyHandle,
  pub entities: Vec<Entity>,
  pub frame_count: i64,
}

impl System for PhysicsSystem {
  fn start(ctx: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    let map_system = ctx.get::<MapSystem>().unwrap();

    /* Create the player. */
    let player_rigid_body = RigidBodyBuilder::dynamic()
      .translation(map_system.map.player_spawn.translation.into_vec())
      .build();
    let player_collider = &ColliderBuilder::ball(0.25)
      .restitution(0.7)
      .collision_groups(InteractionGroups {
        memberships: COLLISION_GROUP_PLAYER,
        filter: COLLISION_GROUP_WALL
          .union(COLLISION_GROUP_ENEMY)
          .union(COLLISION_GROUP_ENEMY_PROJECTILE),
      })
      .build();
    let player_handle = rigid_body_set.insert(player_rigid_body);
    collider_set.insert_with_parent(player_collider.clone(), player_handle, &mut rigid_body_set);

    let player = Entity {
      handle: player_handle,
      components: ComponentSet::new().insert(Damageable {
        health: 100.0,
        destroy_on_zero_health: false,
        current_hitstun: 0.0,
        max_hitstun: 30.0,
      }),
    };

    /* Spawn enemies. */
    let enemies: Vec<_> = map_system
      .map
      .enemy_spawns
      .iter()
      .map(|enemy_spawn| {
        let handle = rigid_body_set.insert(enemy_spawn.rigid_body.clone());
        collider_set.insert_with_parent(enemy_spawn.collider.clone(), handle, &mut rigid_body_set);
        Entity {
          handle,
          components: ComponentSet::new()
            .insert(Damageable {
              health: 100.0,
              destroy_on_zero_health: true,
              current_hitstun: 0.0,
              max_hitstun: 0.0,
            })
            .insert(Damager { damage: 20.0 })
            .insert(Enemy {
              name: enemy_spawn.name.clone(),
            }),
        }
      })
      .collect();

    /* Create the map colliders. */
    map_system.map.colliders.iter().for_each(|map_tile| {
      match map_tile {
        MapTile::Wall(wall) => collider_set.insert(wall.collider.clone()),
      };
    });

    /* Create other structures necessary for the simulation. */
    let integration_parameters = IntegrationParameters::default();
    let physics_pipeline = Rc::new(RefCell::new(PhysicsPipeline::new()));
    let island_manager = IslandManager::new();
    let broad_phase = DefaultBroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let impulse_joint_set = ImpulseJointSet::new();
    let multibody_joint_set = MultibodyJointSet::new();
    let ccd_solver: CCDSolver = CCDSolver::new();
    let entities: Vec<_> = [player].iter().cloned().chain(enemies).collect();

    return Rc::new(Self {
      rigid_body_set,
      collider_set,
      integration_parameters,
      physics_pipeline,
      island_manager,
      broad_phase,
      narrow_phase,
      impulse_joint_set,
      multibody_joint_set,
      ccd_solver,
      player_handle,
      entities,
      frame_count: 0,
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let mut physics_pipeline = self.physics_pipeline.as_ref().borrow_mut();
    let mut island_manager = self.island_manager.clone();
    let mut broad_phase = self.broad_phase.clone();
    let mut narrow_phase = self.narrow_phase.clone();
    let mut impulse_joint_set = self.impulse_joint_set.clone();
    let mut multibody_joint_set = self.multibody_joint_set.clone();
    let mut ccd_solver = self.ccd_solver.clone();
    let mut rigid_body_set = &mut self.rigid_body_set.clone();
    let mut collider_set = self.collider_set.clone();

    /* Move the player */
    let controls_system = ctx.get::<ControlsSystem>().unwrap();

    rigid_body_set[self.player_handle].apply_impulse(controls_system.left_stick.into_vec(), true);

    /* Fire all weapons */
    let combat_system = ctx.get::<CombatSystem>().unwrap();

    let new_projectiles: Vec<Entity> = combat_system
      .new_projectiles
      .iter()
      .map(|projectile| {
        let handle = rigid_body_set.insert(RigidBodyBuilder::dynamic().translation(
          *rigid_body_set[self.player_handle].translation() + projectile.offset.into_vec(),
        ));
        collider_set.insert_with_parent(projectile.collider.clone(), handle, rigid_body_set);

        let rbs_clone = rigid_body_set.clone();
        let player_velocity = rbs_clone[self.player_handle].linvel();
        rigid_body_set[handle].set_linvel(*player_velocity, true);

        rigid_body_set[handle].apply_impulse(projectile.initial_force.into_vec(), true);

        return Entity {
          handle,
          components: ComponentSet::new()
            .insert(DestroyOnCollision)
            .insert(Damager { damage: 10.0 }),
        };
      })
      .collect();

    let entities: Vec<Entity> = self
      .entities
      .iter()
      .cloned()
      .chain(new_projectiles)
      .collect();

    /* Carry out enemy behavior */
    let enemy_system = ctx.get::<EnemySystem>().unwrap();

    let entities = entities
      .iter()
      .cloned()
      .flat_map(|entity: Entity| {
        let entity = &entity;
        let relevant_decision = enemy_system
          .decisions
          .iter()
          .find(|&decision| decision.handle == entity.handle)
          .bind(|&decision| decision);
        if relevant_decision.is_none() {
          return Vec::from([entity.clone()]);
        }
        let relevant_decision = relevant_decision.unwrap();

        rigid_body_set[entity.handle]
          .apply_impulse(relevant_decision.movement_force.into_vec(), true);

        [entity.clone()]
          .iter()
          .cloned()
          .chain(
            relevant_decision
              .projectiles
              .iter()
              .map(|projectile| {
                let handle = rigid_body_set.insert(RigidBodyBuilder::dynamic().translation(
                  *rigid_body_set[entity.handle].translation() + projectile.offset.into_vec(),
                ));
                collider_set.insert_with_parent(
                  projectile.collider.clone(),
                  handle,
                  rigid_body_set,
                );

                let rbs_clone = rigid_body_set.clone();
                let enemy_velocity = rbs_clone[entity.handle].linvel();
                rigid_body_set[handle].set_linvel(*enemy_velocity, true);

                rigid_body_set[handle].apply_impulse(projectile.initial_force.into_vec(), true);

                Entity {
                  handle,
                  components: ComponentSet::new()
                    .insert(DestroyOnCollision)
                    .insert(Damager {
                      damage: projectile.damage,
                    }),
                }
              })
              .collect::<Vec<_>>(),
          )
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    /* Damage all entities colliding with damagers */
    let entities: Vec<_> = entities
      .iter()
      .map(|entity| {
        let damageable = entity.components.get::<Damageable>();

        if damageable.is_none() {
          return entity.clone();
        }
        let damageable = damageable.unwrap();

        if damageable.current_hitstun > 0.0 {
          return Entity {
            handle: entity.handle,
            components: entity.components.with(Damageable {
              health: damageable.health,
              destroy_on_zero_health: damageable.destroy_on_zero_health,
              current_hitstun: damageable.current_hitstun - 1.0,
              max_hitstun: damageable.max_hitstun,
            }),
          };
        }

        let damagers = rigid_body_set[entity.handle]
          .colliders()
          .iter()
          .cloned()
          .flat_map(|collider_handle| {
            narrow_phase
              .contact_pairs_with(collider_handle)
              .flat_map(|contact_pairs| {
                if !contact_pairs.has_any_active_contact {
                  Vec::new()
                } else {
                  [contact_pairs.collider1, contact_pairs.collider2]
                    .iter()
                    .cloned()
                    .filter(|&handle| collider_handle != handle.clone())
                    .collect::<Vec<_>>()
                }
              })
              .collect::<Vec<_>>()
          })
          .map(|collider_handle| {
            collider_set[collider_handle]
              .parent()
              .bind(|rigid_body_handle| {
                entities
                  .iter()
                  .cloned()
                  .find(|entity| entity.handle == *rigid_body_handle)
              })
              .flatten()
              .bind(|entity| entity.components.get::<Damager>())
              .flatten()
          })
          .flatten();

        let incoming_damage = damagers.fold(0.0, |sum, damager| sum + damager.damage);

        if incoming_damage == 0.0 {
          return Entity {
            handle: entity.handle,
            components: entity.components.with(Damageable {
              health: damageable.health,
              destroy_on_zero_health: damageable.destroy_on_zero_health,
              current_hitstun: if damageable.current_hitstun > 0.0 {
                damageable.current_hitstun - 1.0
              } else {
                0.0
              },
              max_hitstun: damageable.max_hitstun,
            }),
          };
        }

        return Entity {
          handle: entity.handle,
          components: entity.components.with(Damageable {
            health: damageable.health - incoming_damage,
            destroy_on_zero_health: damageable.destroy_on_zero_health,
            current_hitstun: damageable.max_hitstun,
            max_hitstun: damageable.max_hitstun,
          }),
        };
      })
      .collect();

    /* Destroy entities with 0 health marked as destroy on 0 health */
    let entities = entities
      .iter()
      .cloned()
      .filter(|entity| {
        let damageable = entity.clone().components.get::<Damageable>();
        if damageable.is_none() {
          return true;
        }
        let damageable = damageable.unwrap();

        let entity_destroyed = damageable.health <= 0.0 && damageable.destroy_on_zero_health;

        if entity_destroyed {
          rigid_body_set.remove(
            entity.handle,
            &mut island_manager,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            true,
          );
        }

        return !entity_destroyed;
      })
      .collect::<Vec<_>>();

    /* Remove colliding entities marked as destroy on collision */
    let entities = entities
      .iter()
      .cloned()
      .filter(|entity| {
        let entity_destroyed = !(entity
          .clone()
          .components
          .get::<DestroyOnCollision>()
          .is_none()
          || rigid_body_set[entity.handle]
            .colliders()
            .iter()
            .cloned()
            .flat_map(|collider| narrow_phase.contact_pairs_with(collider))
            .filter(|&contact_pair| contact_pair.has_any_active_contact)
            .count()
            == 0);

        if entity_destroyed {
          rigid_body_set.remove(
            entity.handle,
            &mut island_manager,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            true,
          );
        }
        return !entity_destroyed;
      })
      .collect();

    /* Step physics */
    physics_pipeline.step(
      &vector![0.0, 0.0],
      &self.integration_parameters,
      &mut island_manager,
      &mut broad_phase,
      &mut narrow_phase,
      &mut rigid_body_set,
      &mut collider_set,
      &mut impulse_joint_set,
      &mut multibody_joint_set,
      &mut ccd_solver,
      &(),
      &(),
    );

    return Rc::new(Self {
      rigid_body_set: rigid_body_set.clone(),
      collider_set: collider_set,
      integration_parameters: self.integration_parameters,
      physics_pipeline: Rc::clone(&self.physics_pipeline),
      island_manager: island_manager,
      broad_phase: broad_phase,
      narrow_phase: narrow_phase,
      impulse_joint_set: impulse_joint_set,
      multibody_joint_set: multibody_joint_set,
      ccd_solver: ccd_solver,
      player_handle: self.player_handle,
      entities,
      frame_count: self.frame_count + 1,
    });
  }
}
