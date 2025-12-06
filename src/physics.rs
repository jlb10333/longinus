use macroquad::prelude::rand;
use rapier2d::{na::Isometry2, prelude::*};
use rpds::{HashTrieMap, List, list};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
  ability::AbilitySystem,
  combat::{CombatSystem, WeaponModuleKind},
  controls::ControlsSystem,
  ecs::{
    ComponentSet, Damageable, Damager, DestroyOnCollision, Destroyed, DropHealthOnDestroy, Entity,
    EntityHandle, Gate, GateTrigger, GiveAbilityOnCollision, GivesItemOnCollision, GravitySource,
    HealOnCollision, MapTransitionOnCollision, SaveMenuOnCollision,
  },
  enemy::EnemySystem,
  load_map::{
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_ENEMY_PROJECTILE, COLLISION_GROUP_PLAYER,
    COLLISION_GROUP_PLAYER_INTERACTIBLE, COLLISION_GROUP_WALL, Map, MapAbilityType, MapGateState,
    MapSystem, MapTile,
  },
  menu::MenuSystem,
  save::SaveData,
  system::System,
  units::UnitConvert2,
};

const PLAYER_SPEED_LIMIT: f32 = 5.0;
const PLAYER_ACCELERATION_MOD: f32 = 0.5;

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
  pub entities: HashTrieMap<EntityHandle, Rc<Entity>>,
  pub new_weapon_modules: List<(i32, WeaponModuleKind)>,
  pub new_abilities: List<MapAbilityType>,
  pub frame_count: i64,
  pub load_new_map: Option<(String, i32)>,
  pub save_point_contact: Option<i32>,
  pub save_point_contact_last_frame: Option<i32>,
}

const PLAYER_MAX_HITSTUN: f32 = 100.0;

fn load_new_map(
  map: &Map,
  map_name: &str,
  acquired_modules: &[(String, i32)],
  target_player_spawn_id: i32,
  player_health: f32,
  player_max_health: f32,
  boost_acquired: bool,
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
    handle: EntityHandle::RigidBody(player_handle),
    components: ComponentSet::new().insert(Damageable {
      health: player_health,
      max_health: player_max_health,
      destroy_on_zero_health: false,
      current_hitstun: 0.0,
      max_hitstun: PLAYER_MAX_HITSTUN,
    }),
    label: "player".to_string(),
  };

  // println!("spawning {} enemies", map.enemy_spawns.len());

  /* MARK: Spawn enemies. */
  let enemies = map
    .enemy_spawns
    .iter()
    .map(|enemy_spawn| {
      let handle = rigid_body_set.insert(enemy_spawn.rigid_body.clone());
      collider_set.insert_with_parent(enemy_spawn.collider.clone(), handle, &mut rigid_body_set);
      Entity {
        handle: EntityHandle::RigidBody(handle),
        components: enemy_spawn.into_entity_components(),
        label: "enemy".to_string(),
      }
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn item pickups. */
  let item_pickups = map
    .item_pickups
    .iter()
    .filter(|item_pickup| !acquired_modules.contains(&(map_name.to_string(), item_pickup.id)))
    .map(|item_pickup| {
      let handle = collider_set.insert(item_pickup.collider.clone());
      Entity {
        handle: EntityHandle::Collider(handle),
        components: ComponentSet::new()
          .insert(GivesItemOnCollision {
            id: item_pickup.id,
            weapon_module_kind: item_pickup.weapon_module_kind,
          })
          .insert(DestroyOnCollision),
        label: "item".to_string(),
      }
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn map transitions. */
  let map_transitions = map
    .map_transitions
    .iter()
    .map(|map_transition| Entity {
      handle: EntityHandle::Collider(collider_set.insert(map_transition.collider.clone())),
      components: ComponentSet::new().insert(MapTransitionOnCollision {
        map_name: map_transition.map_name.clone(),
        target_player_spawn_id: map_transition.target_player_spawn_id,
      }),
      label: map_transition.map_name.clone(),
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn save points. */
  let save_points = map
    .save_points
    .iter()
    .map(|save_point| Entity {
      handle: EntityHandle::Collider(collider_set.insert(save_point.collider.clone())),
      components: ComponentSet::new().insert(SaveMenuOnCollision {
        id: save_point.player_spawn_id,
      }),
      label: "save".to_string(),
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn gates */
  let gates = map
    .gates
    .iter()
    .map(|gate| {
      let rigid_body_handle = rigid_body_set.insert(RigidBodyBuilder::fixed());
      collider_set.insert_with_parent(
        gate.collider.clone(),
        rigid_body_handle,
        &mut rigid_body_set,
      );
      Entity {
        handle: EntityHandle::RigidBody(rigid_body_handle),
        components: ComponentSet::new().insert(Gate { id: gate.id }),
        label: format!("g{}", gate.id),
      }
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn gate triggers */
  let gate_triggers = map
    .gate_triggers
    .iter()
    .map(|gate_trigger| Entity {
      handle: EntityHandle::Collider(collider_set.insert(gate_trigger.collider.clone())),
      components: ComponentSet::new().insert(GateTrigger {
        gate_id: gate_trigger.gate_id,
        action: gate_trigger.action.clone(),
      }),
      label: format!("gt{}", gate_trigger.gate_id),
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn gravity sources */
  let gravity_sources = map
    .gravity_sources
    .iter()
    .map(|gravity_source| Entity {
      handle: EntityHandle::Collider(collider_set.insert(gravity_source.collider.clone())),
      components: ComponentSet::new().insert(GravitySource {
        strength: gravity_source.strength,
      }),
      label: "grav".to_string(),
    })
    .collect::<Vec<_>>();

  /* MARK: Spawn ability pickups */
  let ability_pickups = map
    .ability_pickups
    .iter()
    .filter_map(|ability_pickup| {
      let should_spawn_entity = match ability_pickup.ability_type {
        MapAbilityType::Boost => !boost_acquired,
      };

      if should_spawn_entity {
        Some(Entity {
          handle: EntityHandle::Collider(collider_set.insert(ability_pickup.collider.clone())),
          components: ComponentSet::new()
            .insert(GiveAbilityOnCollision {
              ability_type: ability_pickup.ability_type,
            })
            .insert(DestroyOnCollision),
          label: "ability".to_string(),
        })
      } else {
        None
      }
    })
    .collect::<Vec<_>>();

  /* MARK: Create the map colliders. */
  let map_tiles = map
    .colliders
    .iter()
    .map(|map_tile| match map_tile {
      MapTile::Wall(wall) => {
        if wall.damaging.is_none() && wall.damageable.is_none() {
          (
            Some((
              Isometry2::new(*wall.collider.translation(), 0.0),
              SharedShape::new(*wall.collider.shape().as_cuboid().unwrap()),
            )),
            None,
          )
        } else {
          let damager = wall.damaging.map(|damaging| Damager { damage: damaging });
          let damageable = wall.damageable.map(|damageable| Damageable {
            health: damageable,
            max_health: damageable,
            destroy_on_zero_health: true,
            current_hitstun: 0.0,
            max_hitstun: 0.0,
          });
          let rigid_body_handle = rigid_body_set.insert(RigidBodyBuilder::fixed());
          collider_set.insert_with_parent(
            wall.collider.clone(),
            rigid_body_handle,
            &mut rigid_body_set,
          );

          let label = format!(
            "{}{}",
            if damageable.is_some() { "D" } else { "" },
            if damager.is_some() { "H" } else { "" },
          );

          let component_set = ComponentSet::new();
          let component_set = if let Some(damager) = damager {
            component_set.insert(damager)
          } else {
            component_set
          };
          let component_set = if let Some(damageable) = damageable {
            component_set.insert(damageable)
          } else {
            component_set
          };

          let entity = Entity {
            handle: EntityHandle::RigidBody(rigid_body_handle),
            components: component_set,
            label,
          };
          (None, Some(entity))
        }
      }
    })
    .collect::<Vec<_>>();

  let static_walls = map_tiles
    .iter()
    .cloned()
    .filter_map(|(static_wall, _)| static_wall)
    .collect::<Vec<_>>();
  collider_set.insert(ColliderBuilder::compound(static_walls).build());

  let interactive_walls = map_tiles
    .iter()
    .cloned()
    .filter_map(|(_, interactive_wall)| interactive_wall)
    .collect::<Vec<_>>();

  /* MARK: Create other structures necessary for the simulation. */
  let integration_parameters = IntegrationParameters::default();
  let physics_pipeline = Rc::new(RefCell::new(PhysicsPipeline::new()));
  let island_manager = IslandManager::new();
  let broad_phase = DefaultBroadPhase::new();
  let narrow_phase = NarrowPhase::new();
  let impulse_joint_set = ImpulseJointSet::new();
  let multibody_joint_set = MultibodyJointSet::new();
  let ccd_solver: CCDSolver = CCDSolver::new();
  let entities = [player]
    .iter()
    .cloned()
    .chain(enemies)
    .chain(interactive_walls)
    .chain(gates)
    .chain(item_pickups)
    .chain(ability_pickups)
    .chain(map_transitions)
    .chain(save_points)
    .chain(gate_triggers)
    .chain(gravity_sources)
    .map(|entity| (entity.handle, Rc::new(entity)))
    .collect::<HashTrieMap<_, _>>();

  Rc::new(PhysicsSystem {
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
    new_weapon_modules: list![],
    new_abilities: list![],
    load_new_map: None,
    save_point_contact: None,
    save_point_contact_last_frame: None,
  })
}

impl System for PhysicsSystem {
  type Input = SaveData;
  fn start(ctx: &crate::system::ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
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
      ctx.input.acquired_boost,
    )
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let map_system = ctx.get::<MapSystem>().unwrap();

    let combat_system = ctx.get::<CombatSystem>().unwrap();
    let ability_system = ctx.get::<AbilitySystem>().unwrap();

    if let Some(map) = map_system.map.as_ref() {
      let player_entity = self
        .entities
        .get(&EntityHandle::RigidBody(self.player_handle))
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
        ability_system.acquired_boost,
      );
    }

    let mut physics_pipeline = self.physics_pipeline.as_ref().borrow_mut();
    let mut island_manager = self.island_manager.clone();
    let mut broad_phase = self.broad_phase.clone();
    let mut narrow_phase = self.narrow_phase.clone();
    let mut impulse_joint_set = self.impulse_joint_set.clone();
    let mut multibody_joint_set = self.multibody_joint_set.clone();
    let mut ccd_solver = self.ccd_solver.clone();
    let rigid_body_set = &mut self.rigid_body_set.clone();
    let mut collider_set = self.collider_set.clone();

    let entities = self.entities.clone();

    /* MARK: Don't do physics if currently in menu */
    let menu_system = ctx.get::<MenuSystem<_>>().unwrap();

    if !menu_system.active_menus.is_empty() {
      return Rc::new(Self {
        rigid_body_set: rigid_body_set.clone(),
        collider_set,
        integration_parameters: self.integration_parameters,
        physics_pipeline: Rc::clone(&self.physics_pipeline),
        island_manager,
        broad_phase,
        narrow_phase,
        impulse_joint_set,
        multibody_joint_set,
        ccd_solver,
        player_handle: self.player_handle,
        entities: self.entities.clone(),
        frame_count: self.frame_count + 1,
        new_weapon_modules: list![],
        new_abilities: list![],
        load_new_map: None,
        save_point_contact: self.save_point_contact,
        save_point_contact_last_frame: self.save_point_contact_last_frame,
      });
    }

    /* MARK: Move the player */
    let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

    let next_player_impulse =
      player_movement_impulse(controls_system, &rigid_body_set[self.player_handle]);

    rigid_body_set[self.player_handle].apply_impulse(next_player_impulse, true);

    /* MARK: Perform boost */
    let player_mass = rigid_body_set[self.player_handle].mass();

    if let Some(boost_force) = ability_system.boost_force {
      rigid_body_set[self.player_handle].apply_impulse(boost_force * player_mass, true);
    }

    /* MARK: Gravity source behavior */
    entities.iter().for_each(|(handle, entity)| {
      if let Some(gravity_source) = entity.components.get::<GravitySource>()
        && let EntityHandle::Collider(collider_handle) = handle
      {
        narrow_phase
          .intersection_pairs_with(*collider_handle)
          .filter_map(|(collider1, collider2, colliding)| {
            if colliding {
              [collider1, collider2]
                .iter()
                .find(|other_handle| **other_handle != *collider_handle)
                .cloned()
            } else {
              None
            }
          })
          .for_each(|other_handle| {
            let distance_vec = collider_set[*collider_handle].translation()
              - collider_set[other_handle].translation();

            let distance_squared = distance_vec.magnitude_squared();
            let gravity_intensity = gravity_source.strength / distance_squared;

            if let Some(rigid_body_handle) = collider_set[other_handle].parent() {
              rigid_body_set[rigid_body_handle]
                .apply_impulse(distance_vec * gravity_intensity, true);
            }
          });
      }
    });

    /* MARK: Fire all weapons */
    let new_projectiles = combat_system
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

        let handle = EntityHandle::RigidBody(handle);

        (
          handle,
          Rc::new(Entity {
            handle,
            components: ComponentSet::new()
              .insert(DestroyOnCollision)
              .insert(Damager {
                damage: projectile.damage,
              }),
            label: "projectile".to_string(),
          }),
        )
      })
      .collect::<HashTrieMap<_, _>>();

    let entities = entities.into_iter().chain(new_projectiles.iter());

    /* MARK: Carry out enemy behavior */
    let enemy_system = ctx.get::<EnemySystem>().unwrap();

    let entities = entities
      .flat_map(|(_, entity)| {
        let relevant_decision = enemy_system
          .decisions
          .iter()
          .find(|&decision| decision.handle == entity.handle);
        if relevant_decision.is_none() {
          return vec![(entity.handle, entity.clone())];
        }
        let relevant_decision = relevant_decision.unwrap();

        if let EntityHandle::RigidBody(rigid_body_handle) = entity.handle {
          rigid_body_set[rigid_body_handle]
            .apply_impulse(relevant_decision.movement_force.into_vec(), true);
        }

        let new_projectiles = if let EntityHandle::RigidBody(rigid_body_handle) = entity.handle {
          relevant_decision
            .projectiles
            .iter()
            .map(|projectile| {
              let handle = rigid_body_set.insert(RigidBodyBuilder::dynamic().translation(
                *rigid_body_set[rigid_body_handle].translation() + projectile.offset.into_vec(),
              ));
              collider_set.insert_with_parent(projectile.collider.clone(), handle, rigid_body_set);

              let rbs_clone = rigid_body_set.clone();
              let enemy_velocity = rbs_clone[rigid_body_handle].linvel();
              rigid_body_set[handle].set_linvel(*enemy_velocity, true);

              rigid_body_set[handle].apply_impulse(projectile.initial_force.into_vec(), true);

              (
                EntityHandle::RigidBody(handle),
                Rc::new(Entity {
                  handle: EntityHandle::RigidBody(handle),
                  components: ComponentSet::new()
                    .insert(DestroyOnCollision)
                    .insert(Damager {
                      damage: projectile.damage,
                    }),
                  label: "enemy projectile".to_string(),
                }),
              )
            })
            .collect::<HashMap<_, _>>()
        } else {
          HashMap::new()
        };

        [(
          entity.handle,
          Rc::new(Entity {
            components: entity.components.with(relevant_decision.enemy.clone()),
            ..entity.as_ref().clone()
          }),
        )]
        .into_iter()
        .chain(new_projectiles)
        .chain(
          relevant_decision
            .enemies_to_spawn
            .iter()
            .map(|enemy_to_spawn| {
              let handle = rigid_body_set.insert(enemy_to_spawn.enemy_spawn.rigid_body.clone());
              collider_set.insert_with_parent(
                enemy_to_spawn.enemy_spawn.collider.clone(),
                handle,
                rigid_body_set,
              );
              rigid_body_set[handle].apply_impulse(enemy_to_spawn.initial_force, true);
              (
                EntityHandle::RigidBody(handle),
                Rc::new(Entity {
                  handle: EntityHandle::RigidBody(handle),
                  components: enemy_to_spawn.enemy_spawn.into_entity_components(),
                  label: "child enemy".to_string(),
                }),
              )
            }),
        )
        .collect()
      })
      .collect::<HashTrieMap<_, _>>();

    /* MARK: Damage all entities colliding with damagers */
    let entities = entities
      .iter()
      .map(map_damageable_damage_taken(
        rigid_body_set,
        &narrow_phase,
        &collider_set,
        &entities,
      ))
      .collect::<HashTrieMap<_, _>>();

    /* MARK: Destroy all entities with 0 health marked as such */
    let entities = entities
      .iter()
      .map(|(handle, entity)| {
        if let Some(damageable) = entity.components.get::<Damageable>()
          && damageable.health <= 0.0
        {
          (
            handle,
            Rc::new(Entity {
              components: entity.components.with(Destroyed),
              ..entity.as_ref().clone()
            }),
          )
        } else {
          (handle, Rc::clone(entity))
        }
      })
      .collect::<HashTrieMap<_, _>>();

    /* MARK: Destroy colliding entities marked as destroy on collision */
    let entities = entities
      .into_iter()
      .map(|(handle, entity)| {
        let entity_destroyed = !(entity.components.get::<DestroyOnCollision>().is_none()
          || entity
            .handle
            .colliders(rigid_body_set)
            .iter()
            .flat_map(|&&collider_handle| {
              let collider = &collider_set[collider_handle];

              if collider.is_sensor() {
                narrow_phase
                  .intersection_pairs_with(collider_handle)
                  .flat_map(|(collider1, collider2, is_intersecting)| {
                    if is_intersecting {
                      vec![collider1, collider2]
                    } else {
                      vec![]
                    }
                  })
                  .collect::<Vec<_>>()
              } else {
                narrow_phase
                  .contact_pairs_with(collider_handle)
                  .flat_map(|contact_pair| {
                    if contact_pair.has_any_active_contact {
                      vec![contact_pair.collider1, contact_pair.collider2]
                    } else {
                      vec![]
                    }
                  })
                  .collect::<Vec<_>>()
              }
            })
            .filter(|collider_handle| {
              EntityHandle::Collider(*collider_handle) != entity.handle
                && !collider_set[*collider_handle].is_sensor()
            })
            .count()
            == 0);

        if entity_destroyed {
          (
            *handle,
            Rc::new(Entity {
              components: entity.components.with(Destroyed),
              ..entity.as_ref().clone()
            }),
          )
        } else {
          (*handle, Rc::clone(entity))
        }
      })
      .collect::<Vec<_>>();

    let rng = rand::RandGenerator::new();
    rng.srand(self.frame_count as u64);

    /* MARK: Drop health pickups from entities with 0 health marked as such */
    let entities = entities
      .into_iter()
      .flat_map(|(&handle, entity)| {
        if entity.components.get::<Destroyed>().is_none()
          || entity.components.get::<DropHealthOnDestroy>().is_none()
        {
          return vec![(handle, entity)];
        };
        let drop_health = entity.components.get::<DropHealthOnDestroy>().unwrap();

        let random = rng.gen_range(0.0, 1.0);
        let should_drop_health = random < drop_health.chance;

        if !should_drop_health {
          return vec![(handle, entity)];
        }

        let new_handle = collider_set.insert(
          ColliderBuilder::ball(0.31)
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
              filter: COLLISION_GROUP_PLAYER,
            })
            .sensor(true)
            .translation(*entity.handle.translation(rigid_body_set, &collider_set))
            .build(),
        );
        vec![
          (handle, entity),
          (
            EntityHandle::Collider(new_handle),
            Entity {
              handle: EntityHandle::Collider(new_handle),
              components: ComponentSet::new()
                .insert(DestroyOnCollision)
                .insert(HealOnCollision {
                  amount: drop_health.amount,
                }),
              label: "health".to_string(),
            }
            .into(),
          ),
        ]
      })
      .collect::<HashTrieMap<_, _>>();

    /* MARK: Give items on collision */
    let new_weapon_modules = entities.iter().fold(list![], |acc, (handle, entity)| {
      if let Some(gives_item) = entity.components.get::<GivesItemOnCollision>()
        && handle
          .colliders(rigid_body_set)
          .iter()
          .any(|&entity_collider_handle| {
            rigid_body_set[self.player_handle]
              .colliders()
              .iter()
              .any(|player_collider| {
                narrow_phase
                  .intersection_pair(*entity_collider_handle, *player_collider)
                  .unwrap_or(false)
              })
          })
      {
        acc.push_front((gives_item.id, gives_item.weapon_module_kind))
      } else {
        acc
      }
    });

    /* MARK: Give abilities on collision */
    let new_abilities = entities.iter().fold(list![], |acc, (handle, entity)| {
      if let Some(gives_ability) = entity.components.get::<GiveAbilityOnCollision>()
        && handle
          .colliders(rigid_body_set)
          .iter()
          .any(|&entity_collider_handle| {
            rigid_body_set[self.player_handle]
              .colliders()
              .iter()
              .any(|player_collider| {
                narrow_phase
                  .intersection_pair(*entity_collider_handle, *player_collider)
                  .unwrap_or(false)
              })
          })
      {
        acc.push_front(gives_ability.ability_type)
      } else {
        acc
      }
    });

    /* MARK: Load new map */
    let load_new_map = entities.iter().find_map(|(handle, entity)| {
      if handle
        .colliders(rigid_body_set)
        .iter()
        .all(|&collider_handle| {
          narrow_phase
            .intersection_pairs_with(*collider_handle)
            .filter(|(_, _, colliding)| *colliding)
            .count()
            == 0
        })
      {
        return None;
      }

      entity
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
    let save_point_contact = entities.iter().find_map(|(handle, entity)| {
      if handle
        .colliders(rigid_body_set)
        .iter()
        .all(|&collider_handle| {
          narrow_phase
            .intersection_pairs_with(*collider_handle)
            .filter(|(_, _, colliding)| *colliding)
            .count()
            == 0
        })
      {
        return None;
      }

      entity
        .components
        .get::<SaveMenuOnCollision>()
        .map(|save_menu_on_collision| save_menu_on_collision.id)
    });

    /* MARK: Open/close gates from triggers */
    entities.iter().for_each(|(handle, entity)| {
      if handle
        .colliders(rigid_body_set)
        .iter()
        .any(|&collider_handle| {
          narrow_phase
            .intersection_pairs_with(*collider_handle)
            .filter(|(_, _, colliding)| *colliding)
            .count()
            != 0
        })
        && let Some(gate_trigger) = entity.components.get::<GateTrigger>()
      {
        let gate_handle = entities.iter().find_map(|(handle, entity)| {
          entity.components.get::<Gate>().and_then(|gate| {
            if gate.id == gate_trigger.gate_id {
              Some(handle)
            } else {
              None
            }
          })
        });

        if let Some(gate_handle) = gate_handle {
          gate_handle
            .colliders(rigid_body_set)
            .iter()
            .for_each(|&collider_handle| {
              collider_set[*collider_handle].set_enabled(match gate_trigger.action {
                MapGateState::Close => true,
                MapGateState::Open => false,
              });
            });
        }
      }
    });

    /* MARK: Heal from sensor collision mark as such */
    let entities = entities
      .iter()
      .map(|(handle, entity)| {
        let damageable = entity.components.get::<Damageable>();

        if damageable.is_none() {
          return (handle, Rc::clone(entity));
        }
        let damageable = damageable.unwrap();

        let healing_sensors = entity
          .handle
          .colliders(rigid_body_set)
          .into_iter()
          .flat_map(|&collider_handle| {
            narrow_phase
              .intersection_pairs_with(collider_handle)
              .flat_map(|(collider1, collider2, has_active_contact)| {
                if !has_active_contact {
                  Vec::new()
                } else {
                  [collider1, collider2]
                    .iter()
                    .cloned()
                    .filter(|&handle| collider_handle != handle)
                    .collect::<Vec<_>>()
                }
              })
              .collect::<Vec<_>>()
          })
          .flat_map(|collider_handle| {
            entities
              .iter()
              .find(|(handle, _)| {
                handle
                  .colliders(rigid_body_set)
                  .iter()
                  .any(|&handle| *handle == collider_handle)
              })
              .and_then(|(_, entity)| entity.components.get::<HealOnCollision>())
          });

        let incoming_healing = healing_sensors.fold(0.0, |sum, healing| sum + healing.amount);

        (
          handle,
          Entity {
            components: entity.components.with(Damageable {
              health: (damageable.health + incoming_healing).min(damageable.max_health),
              ..*damageable
            }),
            ..entity.as_ref().clone()
          }
          .into(),
        )
      })
      .collect::<HashTrieMap<_, _>>();

    /* MARK: Remove destroyed entities */
    let entities = entities
      .into_iter()
      .filter_map(|(&&handle, entity)| {
        if entity.components.get::<Destroyed>().is_none() {
          return Some((handle, Rc::clone(entity)));
        }

        match entity.handle {
          EntityHandle::RigidBody(rigid_body_handle) => {
            rigid_body_set.remove(
              rigid_body_handle,
              &mut island_manager,
              &mut collider_set,
              &mut impulse_joint_set,
              &mut multibody_joint_set,
              true,
            );
          }
          EntityHandle::Collider(collider_handle) => {
            collider_set.remove(collider_handle, &mut island_manager, rigid_body_set, true);
          }
        }
        None
      })
      .collect::<HashTrieMap<_, _>>();

    /* MARK: Step physics */
    physics_pipeline.step(
      &vector![0.0, 0.0],
      &self.integration_parameters,
      &mut island_manager,
      &mut broad_phase,
      &mut narrow_phase,
      rigid_body_set,
      &mut collider_set,
      &mut impulse_joint_set,
      &mut multibody_joint_set,
      &mut ccd_solver,
      &(),
      &(),
    );

    Rc::new(Self {
      rigid_body_set: rigid_body_set.clone(),
      collider_set,
      integration_parameters: self.integration_parameters,
      physics_pipeline: Rc::clone(&self.physics_pipeline),
      island_manager,
      broad_phase,
      narrow_phase,
      impulse_joint_set,
      multibody_joint_set,
      ccd_solver,
      player_handle: self.player_handle,
      entities,
      new_weapon_modules,
      new_abilities,
      frame_count: self.frame_count + 1,
      load_new_map,
      save_point_contact,
      save_point_contact_last_frame: self.save_point_contact,
    })
  }
}

fn player_movement_impulse(
  controls_system: Rc<ControlsSystem<SaveData>>,
  player: &RigidBody,
) -> Vector<f32> {
  let attempted_acceleration = controls_system.left_stick.into_vec() * PLAYER_ACCELERATION_MOD;
  let player_mass = player.mass();
  let player_velocity = player.linvel();
  let velocity_change = attempted_acceleration * player_mass;

  let safe_acceleration_x = if attempted_acceleration.x == 0.0
    || velocity_change.x.signum() != player_velocity.x.signum()
    || player_velocity.x.abs() < PLAYER_SPEED_LIMIT
  {
    attempted_acceleration.x
  } else {
    0.0
  };

  let safe_acceleration_y = if attempted_acceleration.y == 0.0
    || velocity_change.y.signum() != player_velocity.y.signum()
    || player_velocity.y.abs() < PLAYER_SPEED_LIMIT
  {
    attempted_acceleration.y
  } else {
    0.0
  };

  vector![safe_acceleration_x, safe_acceleration_y]
}

fn map_damageable_damage_taken(
  rigid_body_set: &RigidBodySet,
  narrow_phase: &NarrowPhase,
  collider_set: &ColliderSet,
  entities: &HashTrieMap<EntityHandle, Rc<Entity>>,
) -> impl Fn((&EntityHandle, &Rc<Entity>)) -> (EntityHandle, Rc<Entity>) {
  |(&handle, entity)| {
    let damageable = entity.components.get::<Damageable>();

    if damageable.is_none() {
      return (handle, Rc::clone(entity));
    }
    let damageable = damageable.unwrap();

    if damageable.current_hitstun > 0.0 {
      return (
        handle,
        Rc::new(Entity {
          components: entity.components.with(Damageable {
            current_hitstun: damageable.current_hitstun - 1.0,
            ..*damageable
          }),
          ..entity.as_ref().clone()
        }),
      );
    }

    let damagers = entity
      .handle
      .colliders(rigid_body_set)
      .into_iter()
      .flat_map(|&collider_handle| {
        narrow_phase
          .contact_pairs_with(collider_handle)
          .flat_map(|contact_pairs| {
            if !contact_pairs.has_any_active_contact {
              Vec::new()
            } else {
              [contact_pairs.collider1, contact_pairs.collider2]
                .into_iter()
                .filter(|&handle| collider_handle != handle)
                .collect::<Vec<_>>()
            }
          })
          .collect::<Vec<_>>()
      })
      .flat_map(|collider_handle| {
        collider_set[collider_handle]
          .parent()
          .and_then(|rigid_body_handle| entities.get(&EntityHandle::RigidBody(rigid_body_handle)))
          .and_then(|entity| entity.components.get::<Damager>())
      });

    let incoming_damage = damagers.fold(0.0, |sum, damager| sum + damager.damage);

    if incoming_damage == 0.0 {
      if damageable.current_hitstun > 0.0 {
        return (
          handle,
          Rc::new(Entity {
            components: entity.components.with(Damageable {
              current_hitstun: damageable.current_hitstun - 1.0,
              ..*damageable
            }),
            ..entity.as_ref().clone()
          }),
        );
      }

      return (handle, Rc::clone(entity));
    }

    (
      handle,
      Rc::new(Entity {
        components: entity.components.with(Damageable {
          health: damageable.health - incoming_damage,
          current_hitstun: damageable.max_hitstun,
          ..*damageable
        }),
        ..entity.as_ref().clone()
      }),
    )
  }
}
