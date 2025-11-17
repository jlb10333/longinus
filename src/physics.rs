use rapier2d::prelude::*;
use std::{cell::RefCell, rc::Rc};

use crate::{
  combat::{CombatSystem, WeaponModuleKind},
  controls::ControlsSystem,
  ecs::{
    ComponentSet, Damageable, Damager, DestroyOnCollision, Enemy, Entity, GivesItemOnCollision,
    MapTransitionOnCollision, SaveMenuOnCollision, Sensor,
  },
  enemy::EnemySystem,
  f::Monad,
  load_map::{
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_PLAYER_INTERACTIBLE, COLLISION_GROUP_WALL, Map, MapSystem, MapTile,
  },
  menu::MenuSystem,
  save::SaveData,
  system::System,
  units::UnitConvert2,
};

const PLAYER_SPEED_LIMIT: f32 = 10.0;

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
  pub sensors: Vec<Sensor>,
  pub new_weapon_modules: Vec<(i32, WeaponModuleKind)>,
  pub frame_count: i64,
  pub load_new_map: Option<(String, i32)>,
  pub save_point_contact: Option<i32>,
  pub save_point_contact_last_frame: Option<i32>,
}

fn load_new_map(
  map: &Map,
  map_name: &str,
  acquired_modules: &Vec<(String, i32)>,
  target_player_spawn_id: i32,
  player_health: f32,
  player_max_health: f32,
) -> Rc<PhysicsSystem> {
  let mut rigid_body_set = RigidBodySet::new();
  let mut collider_set = ColliderSet::new();

  let player_spawn = map
    .player_spawns
    .iter()
    .find(|&player_spawn| player_spawn.id == target_player_spawn_id)
    .unwrap();

  /* MARK: Create the player. */
  let mut player_rigid_body = RigidBodyBuilder::dynamic()
    .translation(player_spawn.translation.into_vec())
    .build();
  player_rigid_body.wake_up(true);
  let player_collider = &ColliderBuilder::ball(0.25)
    .restitution(0.7)
    .collision_groups(InteractionGroups {
      memberships: COLLISION_GROUP_PLAYER,
      filter: COLLISION_GROUP_WALL
        .union(COLLISION_GROUP_ENEMY)
        .union(COLLISION_GROUP_ENEMY_PROJECTILE)
        .union(COLLISION_GROUP_PLAYER_INTERACTIBLE),
    })
    .build();
  let player_handle = rigid_body_set.insert(player_rigid_body);
  collider_set.insert_with_parent(player_collider.clone(), player_handle, &mut rigid_body_set);

  let player = Entity {
    handle: player_handle,
    components: ComponentSet::new().insert(Damageable {
      health: player_health,
      max_health: player_max_health,
      destroy_on_zero_health: false,
      current_hitstun: 0.0,
      max_hitstun: 30.0,
    }),
  };

  /* MARK: Spawn enemies. */
  let enemies: Vec<_> = map
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
            max_health: 100.0,
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

  /* MARK: Spawn item pickups. */
  let item_pickups = map
    .item_pickups
    .iter()
    .filter(|item_pickup| !acquired_modules.contains(&(map_name.to_string(), item_pickup.id)))
    .map(|item_pickup| {
      let handle = collider_set.insert(item_pickup.collider.clone());
      Sensor {
        handle,
        components: ComponentSet::new()
          .insert(GivesItemOnCollision {
            id: item_pickup.id,
            weapon_module_kind: item_pickup.weapon_module_kind.clone(),
          })
          .insert(DestroyOnCollision),
      }
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn map transitions. */
  let map_transitions = map
    .map_transitions
    .iter()
    .map(|map_transition| Sensor {
      handle: collider_set.insert(map_transition.collider.clone()),
      components: ComponentSet::new().insert(MapTransitionOnCollision {
        map_name: map_transition.map_name.clone(),
        target_player_spawn_id: map_transition.target_player_spawn_id,
      }),
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn save points. */
  let save_points = map
    .save_points
    .iter()
    .map(|save_point| Sensor {
      handle: collider_set.insert(save_point.collider.clone()),
      components: ComponentSet::new().insert(SaveMenuOnCollision {
        id: save_point.player_spawn_id,
      }),
    })
    .collect::<Vec<_>>();

  /* MARK: Create the map colliders. */
  map.colliders.iter().for_each(|map_tile| {
    match map_tile {
      MapTile::Wall(wall) => collider_set.insert(wall.collider.clone()),
    };
  });

  /* MARK: Create other structures necessary for the simulation. */
  let integration_parameters = IntegrationParameters::default();
  let physics_pipeline = Rc::new(RefCell::new(PhysicsPipeline::new()));
  let island_manager = IslandManager::new();
  let broad_phase = DefaultBroadPhase::new();
  let narrow_phase = NarrowPhase::new();
  let impulse_joint_set = ImpulseJointSet::new();
  let multibody_joint_set = MultibodyJointSet::new();
  let ccd_solver: CCDSolver = CCDSolver::new();
  let entities: Vec<_> = [player].iter().cloned().chain(enemies).collect();
  let sensors = item_pickups
    .iter()
    .cloned()
    .chain(map_transitions)
    .chain(save_points)
    .collect();

  return Rc::new(PhysicsSystem {
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
    sensors,
    frame_count: 0,
    new_weapon_modules: vec![],
    load_new_map: None,
    save_point_contact: None,
    save_point_contact_last_frame: None,
  });
}

impl System for PhysicsSystem {
  type Input = SaveData;
  fn start(ctx: &crate::system::GameState<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    let map_system = ctx.get::<MapSystem>().unwrap();
    let map = map_system.map.as_ref().unwrap();

    let combat_system = ctx.get::<CombatSystem>().unwrap();

    load_new_map(
      map,
      &map_system.current_map_name,
      &combat_system.acquired_items,
      map_system.target_player_spawn_id,
      ctx.input.player_health,
      ctx.input.player_max_health,
    )
  }

  fn run(
    &self,
    ctx: &crate::system::GameState<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let map_system = ctx.get::<MapSystem>().unwrap();

    let combat_system = ctx.get::<CombatSystem>().unwrap();

    if let Some(map) = map_system.map.as_ref() {
      let player_entity = self
        .entities
        .iter()
        .find(|Entity { handle, .. }| *handle == self.player_handle)
        .unwrap();
      let player_damageable = player_entity.components.get::<Damageable>().unwrap();

      // TODO: give the ability to specify which player spawn to start from
      return load_new_map(
        map,
        &map_system.current_map_name,
        &combat_system.acquired_items,
        map_system.target_player_spawn_id,
        player_damageable.health,
        player_damageable.max_health,
      );
    }

    let mut physics_pipeline = self.physics_pipeline.as_ref().borrow_mut();
    let mut island_manager = self.island_manager.clone();
    let mut broad_phase = self.broad_phase.clone();
    let mut narrow_phase = self.narrow_phase.clone();
    let mut impulse_joint_set = self.impulse_joint_set.clone();
    let mut multibody_joint_set = self.multibody_joint_set.clone();
    let mut ccd_solver = self.ccd_solver.clone();
    let mut rigid_body_set = &mut self.rigid_body_set.clone();
    let mut collider_set = self.collider_set.clone();

    let entities = self.entities.clone();
    let sensors = self.sensors.clone();

    /* MARK: Don't do physics if currently in menu */
    let menu_system = ctx.get::<MenuSystem<_>>().unwrap();

    if menu_system.active_menus.iter().count() > 0 {
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
        entities: self.entities.clone(),
        sensors: self.sensors.clone(),
        frame_count: self.frame_count + 1,
        new_weapon_modules: vec![],
        load_new_map: None,
        save_point_contact: self.save_point_contact,
        save_point_contact_last_frame: self.save_point_contact_last_frame,
      });
    }

    /* MARK: Move the player */
    let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

    let attempted_acceleration = controls_system.left_stick.into_vec();
    let player = &rigid_body_set[self.player_handle];
    let player_mass = player.mass();
    let player_velocity = player.linvel();
    let velocity_change = attempted_acceleration * player_mass;

    let safe_acceleration_x =
      if attempted_acceleration.x == 0.0 || player_velocity.x.abs() + velocity_change.x.abs() < PLAYER_SPEED_LIMIT {
        attempted_acceleration.x
      } else {
        let acceleration_x_exceeding_limit = (velocity_change.x + player_velocity.x - (player_velocity.x.signum() * PLAYER_SPEED_LIMIT)) * player_mass;

        attempted_acceleration.x - acceleration_x_exceeding_limit
      };

    let safe_acceleration_y =
      if attempted_acceleration.y == 0.0 || player_velocity.y.abs() + velocity_change.y.abs() < PLAYER_SPEED_LIMIT {
        attempted_acceleration.y
      } else {
        let acceleration_y_exceeding_limit = (velocity_change.y + player_velocity.y - (player_velocity.y.signum() * PLAYER_SPEED_LIMIT)) * player_mass;

        attempted_acceleration.y - acceleration_y_exceeding_limit
      };

    if player_velocity.magnitude() > 30.0 || safe_acceleration_x > 1.0 || safe_acceleration_y > 1.0 { panic!() }
    
    rigid_body_set[self.player_handle].apply_impulse(vector![safe_acceleration_x, safe_acceleration_y], true);

    /* MARK: Fire all weapons */
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
            .insert(Damager {
              damage: projectile.damage,
            }),
        };
      })
      .collect();

    let entities: Vec<Entity> = entities.iter().cloned().chain(new_projectiles).collect();

    /* MARK: Carry out enemy behavior */
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

    /* MARK: Damage all entities colliding with damagers */
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
              current_hitstun: damageable.current_hitstun - 1.0,
              ..*damageable
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
              current_hitstun: if damageable.current_hitstun > 0.0 {
                damageable.current_hitstun - 1.0
              } else {
                0.0
              },
              ..*damageable
            }),
          };
        }

        println!("{}", incoming_damage);

        return Entity {
          handle: entity.handle,
          components: entity.components.with(Damageable {
            health: damageable.health - incoming_damage,
            ..*damageable
          }),
        };
      })
      .collect();

    /* MARK: Destroy entities with 0 health marked as destroy on 0 health */
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

    /* MARK: Remove colliding entities marked as destroy on collision */
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

    /* MARK: Give items on collision */
    let new_weapon_modules = sensors.iter().cloned().fold(vec![], |acc, sensor| {
      if let Some(gives_item) = sensor.components.get::<GivesItemOnCollision>()
        && rigid_body_set[self.player_handle]
          .colliders()
          .iter()
          .any(|player_collider| {
            narrow_phase
              .intersection_pair(sensor.handle, *player_collider)
              .unwrap_or(false)
          })
      {
        [(gives_item.id, gives_item.weapon_module_kind.clone())]
          .iter()
          .chain(acc.iter())
          .cloned()
          .collect::<Vec<_>>()
      } else {
        acc
      }
    });

    /* MARK: Load new map */
    let load_new_map = sensors.iter().find_map(|sensor| {
      if narrow_phase
        .intersection_pairs_with(sensor.handle)
        .filter(|(_, _, colliding)| *colliding)
        .count()
        == 0
      {
        return None;
      }

      sensor
        .components
        .get::<MapTransitionOnCollision>()
        .map(|map_transition_on_collision| {
          (
            map_transition_on_collision.map_name.clone(),
            map_transition_on_collision.target_player_spawn_id,
          )
        })
    });

    /* MARK: Save point interaction */
    let save_point_contact = sensors.iter().find_map(|sensor| {
      if narrow_phase
        .intersection_pairs_with(sensor.handle)
        .filter(|(_, _, colliding)| *colliding)
        .count()
        == 0
      {
        return None;
      }

      sensor
        .components
        .get::<SaveMenuOnCollision>()
        .map(|save_menu_on_collision| save_menu_on_collision.id)
    });

    /* MARK: Remove colliding sensors marked as destroy on collision */
    let sensors = sensors
      .iter()
      .cloned()
      .filter(|sensor| {
        let entity_destroyed = sensor
          .clone()
          .components
          .get::<DestroyOnCollision>()
          .is_some()
          && narrow_phase
            .intersection_pairs_with(sensor.handle)
            .filter(|(_, _, colliding)| *colliding)
            .count()
            > 0;

        if entity_destroyed {
          collider_set.remove(
            sensor.handle,
            &mut island_manager,
            &mut rigid_body_set,
            true,
          );
        }
        return !entity_destroyed;
      })
      .collect::<Vec<_>>();

    /* MARK: Step physics */
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
      sensors,
      new_weapon_modules,
      frame_count: self.frame_count + 1,
      load_new_map,
      save_point_contact,
      save_point_contact_last_frame: self.save_point_contact,
    });
  }
}
