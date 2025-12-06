use std::{fs, rc::Rc};

use rapier2d::{na::Vector2, prelude::*};
use serde::Deserialize;

use crate::{
  combat::WeaponModuleKind,
  ecs::{ComponentSet, Damageable, Damager, DropHealthOnDestroy, Enemy},
  f::MonadTranslate,
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsScalar, PhysicsVector, UnitConvert2},
};

#[derive(Clone, Debug, Deserialize)]
enum TileLayerName {
  Colliders,
}

#[derive(Clone, Debug, Deserialize)]
struct TileLayer {
  data: Vec<i32>,
  height: i32,
  width: i32,
  name: TileLayerName,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapEnemySpawnClass {
  EnemySpawn,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapEnemyName {
  Defender,
  Seeker,
  SeekerGenerator,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemySpawn {
  x: f32,
  y: f32,
  name: MapEnemyName,
  #[serde(rename = "type")]
  _class: MapEnemySpawnClass,
}

impl MapEnemySpawn {
  pub fn into(&self, map_height: f32) -> EnemySpawn {
    let translation = PhysicsVector::from_vec(vector![
      self.x * 0.125 * TILE_DIMENSION_PHYSICS,
      (map_height - self.y) * 0.125 * TILE_DIMENSION_PHYSICS
    ]);
    EnemySpawn::new(&self.name, translation.into_vec())
  }
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapPlayerSpawnClass {
  PlayerSpawn,
}

#[derive(Clone, Debug, Deserialize)]
struct MapPlayerSpawn {
  id: i32,
  x: f32,
  y: f32,
  #[serde(rename = "type")]
  _class: MapPlayerSpawnClass,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapItemPickupClass {
  ItemPickup,
}

#[derive(Clone, Debug, Deserialize)]
struct MapItemPickup {
  id: i32,
  x: f32,
  y: f32,
  name: WeaponModuleKind,
  #[serde(rename = "type")]
  _class: MapItemPickupClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapMapTransitionTargetClass {
  TargetPlayerSpawn,
}

#[derive(Clone, Debug, Deserialize)]
struct MapMapTransitionTarget {
  name: MapMapTransitionTargetClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapMapTransitionClass {
  MapTransition,
}

#[derive(Clone, Debug, Deserialize)]
struct MapMapTransition {
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  name: String,
  properties: (MapMapTransitionTarget,),
  #[serde(rename = "type")]
  _class: MapMapTransitionClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapSpawnPointTargetClass {
  PlayerSpawnId,
}

#[derive(Clone, Debug, Deserialize)]
struct MapSpawnPointTarget {
  #[serde(rename = "name")]
  _name: MapSpawnPointTargetClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapSavePointClass {
  SavePoint,
}

#[derive(Clone, Debug, Deserialize)]
struct MapSavePoint {
  x: f32,
  y: f32,
  properties: (MapSpawnPointTarget,),
  #[serde(rename = "type")]
  _class: MapSavePointClass,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapGateState {
  Open,
  Close,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGateInitialStateClass {
  InitialState,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGateInitialState {
  #[serde(rename = "name")]
  _name: MapGateInitialStateClass,
  value: MapGateState,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGateClass {
  Gate,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGate {
  id: i32,
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  properties: (MapGateInitialState,),
  #[serde(rename = "type")]
  _class: MapGateClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGateTriggerGateActionClass {
  GateAction,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGateTriggerGateAction {
  #[serde(rename = "name")]
  _name: MapGateTriggerGateActionClass,
  value: MapGateState,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGateTriggerTargetGateClass {
  TargetGate,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGateTriggerTargetGate {
  #[serde(rename = "name")]
  _name: MapGateTriggerTargetGateClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGateTrigger {
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  properties: (MapGateTriggerGateAction, MapGateTriggerTargetGate),
}

#[derive(Clone, Debug, Deserialize)]
enum MapGravitySourceRadiusClass {
  Radius,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGravitySourceRadius {
  #[serde(rename = "name")]
  _name: MapGravitySourceRadiusClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGravitySourceStrengthClass {
  Strength,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGravitySourceStrength {
  #[serde(rename = "name")]
  _name: MapGravitySourceStrengthClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGravitySourceClass {
  GravitySource,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGravitySource {
  x: f32,
  y: f32,
  properties: (MapGravitySourceRadius, MapGravitySourceStrength),
  #[serde(rename = "type")]
  _class: MapGravitySourceClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapAbilityPickupClass {
  AbilityPickup,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum MapAbilityType {
  Boost,
}

#[derive(Clone, Debug, Deserialize)]
struct MapAbilityPickup {
  x: f32,
  y: f32,
  name: MapAbilityType,
  #[serde(rename = "type")]
  _class: MapAbilityPickupClass,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Object {
  EnemySpawn(MapEnemySpawn),
  PlayerSpawn(MapPlayerSpawn),
  ItemPickup(MapItemPickup),
  MapTransition(MapMapTransition),
  SavePoint(MapSavePoint),
  Gate(MapGate),
  GateTrigger(MapGateTrigger),
  GravitySource(MapGravitySource),
  AbilityPickup(MapAbilityPickup),
}

#[derive(Clone, Debug, Deserialize)]
enum ObjectLayerName {
  Entities,
}

#[derive(Clone, Debug, Deserialize)]
struct ObjectLayer {
  objects: Vec<Object>,
  name: ObjectLayerName,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Layer {
  TileLayer(TileLayer),
  ObjectLayer(ObjectLayer),
}

#[derive(Clone, Debug, Deserialize)]
struct RawMap {
  layers: Vec<Layer>,
}

fn deser_map(raw: &str) -> RawMap {
  serde_json::from_str(raw).expect("JSON was not well-formatted")
}

pub const COLLISION_GROUP_WALL: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_PLAYER_PROJECTILE: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ENEMY: Group = Group::GROUP_4;
pub const COLLISION_GROUP_ENEMY_PROJECTILE: Group = Group::GROUP_5;
pub const COLLISION_GROUP_PLAYER_INTERACTIBLE: Group = Group::GROUP_6;

#[derive(Clone)]
pub struct EnemySpawn {
  pub name: Enemy,
  pub collider: Collider,
  pub rigid_body: RigidBody,
}

impl EnemySpawn {
  pub fn new(name: &MapEnemyName, translation: Vector2<f32>) -> Self {
    let collider = collider_from_enemy_name(name.clone());
    let rigid_body_builder = match name {
      MapEnemyName::Defender => RigidBodyBuilder::fixed(),
      MapEnemyName::Seeker => RigidBodyBuilder::dynamic(),
      MapEnemyName::SeekerGenerator => RigidBodyBuilder::fixed(),
    };
    let mut rigid_body = rigid_body_builder.translation(translation).build();
    rigid_body.wake_up(true);
    EnemySpawn {
      name: Enemy::default_from_map(name.clone()),
      collider,
      rigid_body,
    }
  }

  pub fn into_entity_components(&self) -> ComponentSet {
    match self.name {
      Enemy::Defender(_) => ComponentSet::new()
        .insert(Damageable {
          health: 100.0,
          max_health: 100.0,
          destroy_on_zero_health: true,
          current_hitstun: 0.0,
          max_hitstun: 0.0,
        })
        .insert(Damager { damage: 10.0 })
        .insert(DropHealthOnDestroy {
          amount: 20.0,
          chance: 0.4,
        }),
      Enemy::Seeker(_) => ComponentSet::new()
        .insert(Damageable {
          health: 30.0,
          max_health: 30.0,
          destroy_on_zero_health: true,
          current_hitstun: 0.0,
          max_hitstun: 0.0,
        })
        .insert(Damager { damage: 25.0 })
        .insert(DropHealthOnDestroy {
          amount: 10.0,
          chance: 0.5,
        }),
      Enemy::SeekerGenerator(_) => ComponentSet::new()
        .insert(Damageable {
          health: 120.0,
          max_health: 120.0,
          destroy_on_zero_health: true,
          current_hitstun: 0.0,
          max_hitstun: 0.0,
        })
        .insert(Damager { damage: 10.0 })
        .insert(DropHealthOnDestroy {
          amount: 35.0,
          chance: 0.7,
        }),
    }
    .insert(self.name.clone())
  }
}

#[derive(Clone)]
pub struct PlayerSpawn {
  pub id: i32,
  pub translation: PhysicsVector,
}

#[derive(Clone)]
pub struct ItemPickup {
  pub id: i32,
  pub weapon_module_kind: WeaponModuleKind,
  pub collider: Collider,
}

#[derive(Clone)]
pub struct MapTransition {
  pub map_name: String,
  pub collider: Collider,
  pub target_player_spawn_id: i32,
}

#[derive(Clone)]
pub struct SavePoint {
  pub player_spawn_id: i32,
  pub collider: Collider,
}

#[derive(Clone)]
pub struct Gate {
  pub id: i32,
  pub collider: Collider,
}

#[derive(Clone)]
pub struct GateTrigger {
  pub gate_id: i32,
  pub collider: Collider,
  pub action: MapGateState,
}

#[derive(Clone)]
pub struct GravitySource {
  pub collider: Collider,
  pub strength: f32,
}

#[derive(Clone)]
pub struct AbilityPickup {
  pub collider: Collider,
  pub ability_type: MapAbilityType,
}

#[derive(Clone)]
pub struct Wall {
  pub collider: Collider,
  pub damaging: Option<f32>,
  pub damageable: Option<f32>,
}

fn collider_from_enemy_name(name: MapEnemyName) -> Collider {
  let collider_builder = match name {
    MapEnemyName::Defender => ColliderBuilder::cuboid(0.5, 0.5),
    MapEnemyName::Seeker => ColliderBuilder::cuboid(0.2, 0.2).mass(1.0),
    MapEnemyName::SeekerGenerator => ColliderBuilder::cuboid(0.7, 0.7),
  };

  let collision_groups = InteractionGroups {
    memberships: COLLISION_GROUP_ENEMY,
    filter: COLLISION_GROUP_PLAYER
      .union(COLLISION_GROUP_PLAYER_PROJECTILE)
      .union(COLLISION_GROUP_WALL),
  };

  collider_builder.collision_groups(collision_groups).build()
}

#[derive(Clone)]
pub enum MapComponent {
  Player(PlayerSpawn),
  Enemy(EnemySpawn),
  ItemPickup(ItemPickup),
  MapTransition(MapTransition),
  SavePoint(SavePoint),
  Gate(Gate),
  GateTrigger(GateTrigger),
  GravitySource(GravitySource),
  AbilityPickup(AbilityPickup),
}

fn map_scalar_to_physics(scalar: f32) -> PhysicsScalar {
  PhysicsScalar(scalar * 0.125 * TILE_DIMENSION_PHYSICS)
}

impl Object {
  pub fn into(&self, map_height: f32) -> MapComponent {
    match self {
      Object::EnemySpawn(enemy_spawn) => MapComponent::Enemy(enemy_spawn.into(map_height)),

      Object::PlayerSpawn(player_spawn) => MapComponent::Player(PlayerSpawn {
        id: player_spawn.id,
        translation: PhysicsVector::from_vec(vector![
          player_spawn.x * 0.125 * TILE_DIMENSION_PHYSICS,
          (map_height - player_spawn.y) * 0.125 * TILE_DIMENSION_PHYSICS
        ]),
      }),

      Object::ItemPickup(item_pickup) => MapComponent::ItemPickup(ItemPickup {
        id: item_pickup.id,
        weapon_module_kind: item_pickup.name.clone(),
        collider: ColliderBuilder::ball(1.0)
          .translation(
            PhysicsVector::from_vec(vector![
              item_pickup.x * 0.125 * TILE_DIMENSION_PHYSICS,
              (map_height - item_pickup.y) * 0.125 * TILE_DIMENSION_PHYSICS
            ])
            .into_vec(),
          )
          .sensor(true)
          .collision_groups(InteractionGroups {
            memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
            filter: COLLISION_GROUP_PLAYER,
          })
          .build(),
      }),

      Object::MapTransition(map_transition) => MapComponent::MapTransition(MapTransition {
        target_player_spawn_id: map_transition.properties.0.value,
        map_name: map_transition.name.clone(),
        collider: cuboid_collider_from_map(
          map_transition.x,
          map_transition.y,
          map_transition.width,
          map_transition.height,
          map_height,
        )
        .sensor(true)
        .collision_groups(InteractionGroups {
          memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
          filter: COLLISION_GROUP_PLAYER,
        })
        .build(),
      }),

      Object::SavePoint(save_point) => MapComponent::SavePoint(SavePoint {
        player_spawn_id: save_point.properties.0.value,
        collider: ColliderBuilder::ball(1.0)
          .translation(vector![
            *map_scalar_to_physics(save_point.x),
            *map_scalar_to_physics(map_height - save_point.y)
          ])
          .sensor(true)
          .collision_groups(InteractionGroups {
            memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
            filter: COLLISION_GROUP_PLAYER,
          })
          .build(),
      }),

      Object::Gate(gate) => MapComponent::Gate(Gate {
        id: gate.id,
        collider: cuboid_collider_from_map(gate.x, gate.y, gate.width, gate.height, map_height)
          .enabled(matches!(gate.properties.0.value, MapGateState::Close))
          .build(),
      }),

      Object::GateTrigger(gate_trigger) => MapComponent::GateTrigger(GateTrigger {
        gate_id: gate_trigger.properties.1.value,
        collider: cuboid_collider_from_map(
          gate_trigger.x,
          gate_trigger.y,
          gate_trigger.width,
          gate_trigger.height,
          map_height,
        )
        .sensor(true)
        .collision_groups(InteractionGroups {
          memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
          filter: COLLISION_GROUP_PLAYER,
        })
        .build(),
        action: gate_trigger.properties.0.value.clone(),
      }),

      Object::GravitySource(gravity_source) => MapComponent::GravitySource(GravitySource {
        collider: ColliderBuilder::ball(gravity_source.properties.0.value)
          .translation(physics_translation_from_map(
            gravity_source.x,
            gravity_source.y,
            0.0,
            0.0,
            map_height,
          ))
          .sensor(true)
          .build(),
        strength: gravity_source.properties.1.value,
      }),

      Object::AbilityPickup(ability_pickup) => MapComponent::AbilityPickup(AbilityPickup {
        ability_type: ability_pickup.name,
        collider: ColliderBuilder::ball(1.0)
          .translation(physics_translation_from_map(
            ability_pickup.x,
            ability_pickup.y,
            0.0,
            0.0,
            map_height,
          ))
          .sensor(true)
          .collision_groups(InteractionGroups {
            memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
            filter: COLLISION_GROUP_PLAYER,
          })
          .build(),
      }),
    }
  }
}

fn physics_translation_from_map(
  translation_map_x: f32,
  translation_map_y: f32,
  translation_map_width: f32,
  translation_map_height: f32,
  map_height: f32,
) -> Vector2<f32> {
  vector![
    *map_scalar_to_physics(translation_map_x + translation_map_width / 2.0),
    *map_scalar_to_physics(map_height - translation_map_y - translation_map_height / 2.0)
  ]
}

fn cuboid_collider_from_map(
  translation_map_x: f32,
  translation_map_y: f32,
  translation_map_width: f32,
  translation_map_height: f32,
  map_height: f32,
) -> ColliderBuilder {
  ColliderBuilder::cuboid(
    *map_scalar_to_physics(translation_map_width / 2.0),
    *map_scalar_to_physics(translation_map_height / 2.0),
  )
  .translation(physics_translation_from_map(
    translation_map_x,
    translation_map_y,
    translation_map_width,
    translation_map_height,
    map_height,
  ))
}

impl ObjectLayer {
  pub fn into(&self, map_height: f32) -> Vec<MapComponent> {
    self
      .objects
      .iter()
      .map(|object| object.into(map_height))
      .collect()
  }
}

pub const TILE_DIMENSION_PHYSICS: f32 = 0.8;

const EMPTY: i32 = 0;
const WALL_COLLIDER: i32 = 1;
const WALL_DESTRUCTIBLE: i32 = 2;
const WALL_DAMAGING: i32 = 3;
const WALL: [i32; 3] = [WALL_COLLIDER, WALL_DESTRUCTIBLE, WALL_DAMAGING];

#[derive(Clone)]
pub enum MapTile {
  Wall(Wall),
}

pub fn translation_vector_from_index(index: i32, map_dimensions: Vector2<i32>) -> Vector<f32> {
  vector![
    ((index % map_dimensions.x) as f32 + 0.5) * TILE_DIMENSION_PHYSICS,
    ((map_dimensions.y - (index / map_dimensions.x)) as f32 - 0.5) * TILE_DIMENSION_PHYSICS
  ]
}

const DESTRUCTIBLE_WALL_HEALTH: f32 = 1.0;
const DAMAGING_WALL_DAMAGE: f32 = 10.0;

impl TileLayer {
  pub fn into(&self) -> Vec<MapTile> {
    return self
      .data
      .iter()
      .enumerate()
      .filter_map(|(uindex, tile_data)| {
        let index = uindex.try_into().unwrap();
        if WALL.contains(tile_data) {
          let collider =
            ColliderBuilder::cuboid(TILE_DIMENSION_PHYSICS / 2.0, TILE_DIMENSION_PHYSICS / 2.0)
              .translation(translation_vector_from_index(
                index,
                vector![self.width, self.height],
              ))
              .collision_groups(InteractionGroups {
                memberships: COLLISION_GROUP_WALL,
                filter: COLLISION_GROUP_PLAYER
                  .union(COLLISION_GROUP_PLAYER_PROJECTILE)
                  .union(COLLISION_GROUP_ENEMY)
                  .union(COLLISION_GROUP_ENEMY_PROJECTILE),
              })
              .build();

          let damageable = if *tile_data == WALL_DESTRUCTIBLE {
            Some(DESTRUCTIBLE_WALL_HEALTH)
          } else {
            None
          };

          let damaging = if *tile_data == WALL_DAMAGING {
            Some(DAMAGING_WALL_DAMAGE)
          } else {
            None
          };

          return Some(MapTile::Wall(Wall {
            collider,
            damageable,
            damaging,
          }));
        }
        if *tile_data == EMPTY {
          return None;
        }
        todo!("unaccounted wall {}", tile_data);
      })
      .collect();
  }
}

pub struct Map {
  pub colliders: Vec<MapTile>,
  pub player_spawns: Vec<PlayerSpawn>,
  pub enemy_spawns: Vec<EnemySpawn>,
  pub item_pickups: Vec<ItemPickup>,
  pub map_transitions: Vec<MapTransition>,
  pub save_points: Vec<SavePoint>,
  pub gates: Vec<Gate>,
  pub gate_triggers: Vec<GateTrigger>,
  pub gravity_sources: Vec<GravitySource>,
  pub ability_pickups: Vec<AbilityPickup>,
}

impl RawMap {
  pub fn into(&self) -> Map {
    let tile_layer = self
      .layers
      .iter()
      .find_map(|layer| {
        if let Layer::TileLayer(tile_layer) = layer
          && let TileLayerName::Colliders = tile_layer.name
        {
          Some(tile_layer)
        } else {
          None
        }
      })
      .unwrap();

    let colliders = tile_layer.into();

    let entities_layer = self
      .layers
      .iter()
      .find_map(|layer| match layer {
        Layer::ObjectLayer(object_layer) => match object_layer.name {
          ObjectLayerName::Entities => Some(object_layer),
        },
        Layer::TileLayer(_) => None,
      })
      .unwrap();

    let map_height = tile_layer.height as f32 * 8.0;

    let converted_entities = entities_layer.into(map_height);

    let enemy_spawns: Vec<EnemySpawn> = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Enemy(enemy_spawn) = object {
          vec![enemy_spawn.clone()]
        } else {
          vec![]
        }
      })
      .collect();

    let player_spawns = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Player(player_spawn) = object {
          vec![player_spawn.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>()
      .clone();

    let item_pickups = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::ItemPickup(item_pickup) = object {
          vec![item_pickup.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let map_transitions = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::MapTransition(map_transition) = object {
          vec![map_transition.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let save_points = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::SavePoint(save_point) = object {
          vec![save_point.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let gates = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Gate(gate) = object {
          vec![gate.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let gate_triggers = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::GateTrigger(gate_trigger) = object {
          vec![gate_trigger.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let gravity_sources = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::GravitySource(gravity_source) = object {
          vec![gravity_source.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let ability_pickups = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::AbilityPickup(gravity_source) = object {
          vec![gravity_source.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    Map {
      colliders,
      enemy_spawns,
      player_spawns,
      item_pickups,
      map_transitions,
      save_points,
      gates,
      gate_triggers,
      gravity_sources,
      ability_pickups,
    }
  }
}

pub fn load(file_path: &str) -> Option<Map> {
  fs::read_to_string(file_path)
    .translate()
    .as_ref()
    .map(|raw_file| (&deser_map(raw_file)).into())
}

pub struct MapSystem {
  pub map: Option<Map>,
  pub current_map_name: String,
  pub target_player_spawn_id: i32,
}

fn map_read_path(map_name: &String) -> String {
  format!("./assets/maps/{map_name}.json")
}

impl System for MapSystem {
  type Input = SaveData;
  fn start(
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    let save_data = &ctx.input;

    let map = load(&map_read_path(&save_data.map_name));
    Rc::new(Self {
      map,
      current_map_name: save_data.map_name.clone(),
      target_player_spawn_id: save_data.player_spawn_id,
    })
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>> {
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    let load_new_map = physics_system.load_new_map.as_ref();

    Rc::new(Self {
      map: physics_system
        .load_new_map
        .as_ref()
        .and_then(|(new_map_name, _)| load(&map_read_path(&new_map_name.to_string()))),
      current_map_name: load_new_map
        .map(|(map_name, _)| map_name)
        .unwrap_or(&self.current_map_name)
        .clone(),
      target_player_spawn_id: *load_new_map
        .map(|(_, id)| id)
        .unwrap_or(&self.target_player_spawn_id),
    })
  }
}
