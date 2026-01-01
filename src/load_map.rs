use std::{
  env::{current_dir, current_exe},
  f32::consts::PI,
  fs,
  path::Path,
  rc::Rc,
};

use rapier2d::{
  na::{Unit, Vector2},
  prelude::*,
};
use serde::Deserialize;
use serde_literals::lit_str;

use crate::{
  combat::{WeaponModuleKind, distance_projection_physics},
  ecs::{ComponentSet, Damageable, Damager, DropHealthOnDestroy, Enemy},
  f::MonadTranslate,
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsScalar, PhysicsVector, UnitConvert2, vec_zero},
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

lit_str!(EnemySpawnTemplatePath, "templates/EnemySpawn.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum EnemySpawnTemplate {
  #[serde(with = "EnemySpawnTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapEnemyName {
  /* Dragonspawn */
  Goblin,
  /* Angelic Constructs */
  Defender,
  Seeker,
  SeekerGenerator,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemySpawn {
  x: f32,
  y: f32,
  name: MapEnemyName,
  template: EnemySpawnTemplate,
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

lit_str!(PlayerSpawnTemplatePath, "templates/Player Spawn.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum PlayerSpawnTemplate {
  #[serde(with = "PlayerSpawnTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
struct MapPlayerSpawn {
  id: i32,
  x: f32,
  y: f32,
  template: PlayerSpawnTemplate,
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

lit_str!(MapTransitionTemplatePath, "templates/MapTransition.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapTransitionTemplate {
  #[serde(with = "MapTransitionTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
struct MapMapTransition {
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  name: String,
  properties: (MapMapTransitionTarget,),
  template: MapTransitionTemplate,
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
enum MapBlockClass {
  Block,
}

#[derive(Clone, Debug, Deserialize)]
struct MapBlock {
  id: i32,
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  #[serde(rename = "type")]
  _class: MapBlockClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapObject1IdClass {
  Object1Id,
}

#[derive(Clone, Debug, Deserialize)]
struct MapObject1Id {
  name: MapObject1IdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapObject1LocalXClass {
  Object1LocalX,
}

#[derive(Clone, Debug, Deserialize)]
struct MapObject1LocalX {
  name: MapObject1LocalXClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapObject1LocalYClass {
  Object1LocalY,
}

#[derive(Clone, Debug, Deserialize)]
struct MapObject1LocalY {
  name: MapObject1LocalYClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapObject2IdClass {
  Object2Id,
}

#[derive(Clone, Debug, Deserialize)]
struct MapObject2Id {
  name: MapObject2IdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapObject2LocalXClass {
  Object2LocalX,
}

#[derive(Clone, Debug, Deserialize)]
struct MapObject2LocalX {
  name: MapObject2LocalXClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapObject2LocalYClass {
  Object2LocalY,
}

#[derive(Clone, Debug, Deserialize)]
struct MapObject2LocalY {
  name: MapObject2LocalYClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGlueClass {
  Glue,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapGlueMapObjects {
  MultiObject(
    (
      MapObject1Id,
      MapObject1LocalX,
      MapObject1LocalY,
      MapObject2Id,
      MapObject2LocalX,
      MapObject2LocalY,
    ),
  ),
  SingleObject((MapObject1Id, MapObject1LocalX, MapObject1LocalY)),
}

#[derive(Clone, Debug, Deserialize)]
struct MapGlue {
  properties: MapGlueMapObjects,
  #[serde(rename = "type")]
  _class: MapGlueClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapTouchSensorTargetActivationClass {
  TargetActivation,
}

#[derive(Clone, Debug, Deserialize)]
struct MapTouchSensorTargetActivation {
  #[serde(rename = "name")]
  _name: MapTouchSensorTargetActivationClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct MapTouchSensor {
  id: i32,
  x: f32,
  y: f32,
  width: f32,
  height: f32,
  properties: (MapTouchSensorTargetActivation,),
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
  properties: (
    Option<MapActivatorId>,
    MapGravitySourceRadius,
    MapGravitySourceStrength,
  ),
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
  Chain,
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
enum MapInitialActivationClass {
  InitialActivation,
}

#[derive(Clone, Debug, Deserialize)]
struct MapInitialActivation {
  #[serde(rename = "name")]
  _name: MapInitialActivationClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapChainSwitchClass {
  ChainSwitch,
}

#[derive(Clone, Debug, Deserialize)]
enum MapRotationClass {
  Rotation,
}

#[derive(Clone, Debug, Deserialize)]
enum MapActivatorIdClass {
  ActivatorId,
}

#[derive(Clone, Debug, Deserialize)]
struct MapActivatorId {
  #[serde(rename = "name")]
  _name: MapActivatorIdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapActivator1IdClass {
  Activator1Id,
}

#[derive(Clone, Debug, Deserialize)]
struct MapActivator1Id {
  #[serde(rename = "name")]
  _name: MapActivator1IdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapActivator2IdClass {
  Activator2Id,
}

#[derive(Clone, Debug, Deserialize)]
struct MapActivator2Id {
  #[serde(rename = "name")]
  _name: MapActivator2IdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct MapChainSwitch {
  id: i32,
  x: f32,
  y: f32,
  rotation: f32,
  properties: (MapInitialActivation,),
  #[serde(rename = "type")]
  _class: MapChainSwitchClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapMountPointClass {
  MountPoint,
}

#[derive(Clone, Debug, Deserialize)]
struct MapMountPoint {
  id: i32,
  x: f32,
  y: f32,
  #[serde(rename = "type")]
  _class: MapMountPointClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapOrClass {
  Or,
}

#[derive(Clone, Debug, Deserialize)]
struct MapOr {
  id: i32,
  x: f32,
  y: f32,
  properties: (MapActivator1Id, MapActivator2Id),
  #[serde(rename = "type")]
  _class: MapOrClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapAndClass {
  And,
}

#[derive(Clone, Debug, Deserialize)]
struct MapAnd {
  id: i32,
  x: f32,
  y: f32,
  properties: (MapActivator1Id, MapActivator2Id),
  #[serde(rename = "type")]
  _class: MapAndClass,
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
  properties: (MapActivatorId,),
  #[serde(rename = "type")]
  _class: MapGateClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapReverseDirectionClass {
  ReverseDirection,
}

#[derive(Clone, Debug, Deserialize)]
struct MapReverseDirection {
  #[serde(rename = "name")]
  _name: MapReverseDirectionClass,
  value: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct MapPoint {
  x: f32,
  y: f32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapLocomotorClass {
  Locomotor,
}

#[derive(Clone, Debug, Deserialize)]
struct MapLocomotor {
  id: i32,
  x: f32,
  y: f32,
  polyline: [MapPoint; 2],
  properties: (MapActivatorId, MapReverseDirection),
  #[serde(rename = "type")]
  _class: MapLocomotorClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapEngineClass {
  Engine,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEngine {
  id: i32,
  x: f32,
  y: f32,
  properties: (Option<MapActivatorId>,),
  #[serde(rename = "type")]
  _class: MapEngineClass,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Object {
  EnemySpawn(MapEnemySpawn),
  PlayerSpawn(MapPlayerSpawn),
  ItemPickup(MapItemPickup),
  MapTransition(MapMapTransition),
  SavePoint(MapSavePoint),
  Block(MapBlock),
  TouchSensor(MapTouchSensor),
  GravitySource(MapGravitySource),
  AbilityPickup(MapAbilityPickup),
  ChainSwitch(MapChainSwitch),
  MountPoint(MapMountPoint),
  Or(MapOr),
  And(MapAnd),
  Gate(MapGate),
  Locomotor(MapLocomotor),
  Glue(MapGlue),
  Engine(MapEngine),
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
struct RawMap {
  layers: (TileLayer, ObjectLayer),
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
pub const COLLISION_GROUP_CHAIN: Group = Group::GROUP_7;

pub const GRAVITY_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: Group::all(),
  filter: COLLISION_GROUP_WALL.complement(),
  test_mode: InteractionTestMode::And,
};

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
      MapEnemyName::Goblin => RigidBodyBuilder::dynamic(),
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
      Enemy::Goblin(_) => ComponentSet::new()
        .insert(Damageable {
          health: 50.0,
          max_health: 50.0,
          destroy_on_zero_health: true,
          current_hitstun: 0.0,
          max_hitstun: 0.0,
        })
        .insert(Damager { damage: 10.0 })
        .insert(DropHealthOnDestroy {
          amount: 15.0,
          chance: 0.5,
        }),
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
pub struct Block {
  pub id: i32,
  pub rigid_body: RigidBody,
  pub collider: Collider,
}

#[derive(Clone)]
pub struct TouchSensor {
  pub collider: Collider,
  pub target_activation: f32,
  pub id: i32,
}

#[derive(Clone)]
pub struct GravitySource {
  pub collider: Collider,
  pub strength: f32,
  pub activator_id: Option<i32>,
}

#[derive(Clone)]
pub struct AbilityPickup {
  pub collider: Collider,
  pub ability_type: MapAbilityType,
}

#[derive(Clone)]
pub struct ChainSwitch {
  pub id: i32,
  pub collider: Collider,
  pub switch_center: RigidBody,
  pub mount_body: RigidBody,
  pub switch_joint: PrismaticJoint,
}

#[derive(Clone)]
pub struct MountPoint {
  pub id: i32,
  pub rigid_body: RigidBody,
  pub zone: Collider,
  pub knob: Collider,
}

#[derive(Clone)]
pub struct Or {
  pub rigid_body: RigidBody,
  pub id: i32,
  pub activator_ids: (i32, i32),
}

#[derive(Clone)]
pub struct And {
  pub rigid_body: RigidBody,
  pub id: i32,
  pub activator_ids: (i32, i32),
}

#[derive(Clone)]
pub struct Gate {
  pub rigid_body: RigidBody,
  pub id: i32,
  pub activator_id: i32,
}

#[derive(Clone)]
pub struct Locomotor {
  pub id: i32,
  pub base: RigidBody,
  pub joint: PrismaticJoint,
  pub knob: RigidBody,
  pub reverse_direction: bool,
  pub activator_id: i32,
}

#[derive(Clone)]
pub struct Glue {
  pub attachments: ((i32, Vector2<f32>), (Option<i32>, Vector2<f32>)),
}

#[derive(Clone)]
pub struct Engine {
  pub id: i32,
  pub activator_id: Option<i32>,
  pub rigid_body: RigidBody,
}

#[derive(Clone)]
pub struct Wall {
  pub collider: Collider,
  pub damaging: Option<f32>,
  pub damageable: Option<f32>,
}

fn collider_from_enemy_name(name: MapEnemyName) -> Collider {
  let collider_builder = match name {
    MapEnemyName::Goblin => ColliderBuilder::cuboid(0.5, 0.3),
    MapEnemyName::Defender => ColliderBuilder::cuboid(0.5, 0.5),
    MapEnemyName::Seeker => ColliderBuilder::cuboid(0.2, 0.2).mass(1.0),
    MapEnemyName::SeekerGenerator => ColliderBuilder::cuboid(0.7, 0.7),
  };

  let collision_groups = InteractionGroups {
    memberships: COLLISION_GROUP_ENEMY,
    filter: COLLISION_GROUP_PLAYER
      .union(COLLISION_GROUP_PLAYER_PROJECTILE)
      .union(COLLISION_GROUP_WALL),
    ..Default::default()
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
  Block(Block),
  TouchSensor(TouchSensor),
  GravitySource(GravitySource),
  AbilityPickup(AbilityPickup),
  ChainSwitch(ChainSwitch),
  MountPoint(MountPoint),
  Or(Or),
  And(And),
  Gate(Gate),
  Locomotor(Locomotor),
  Glue(Glue),
  Engine(Engine),
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
        weapon_module_kind: item_pickup.name,
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
            ..Default::default()
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
          ..Default::default()
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
            ..Default::default()
          })
          .build(),
      }),

      Object::Block(block) => MapComponent::Block(Block {
        id: block.id,
        rigid_body: RigidBodyBuilder::dynamic()
          .translation(physics_translation_from_map(
            block.x, block.y, 0.0, 0.0, map_height,
          ))
          .build(),
        collider: ColliderBuilder::cuboid(
          *map_scalar_to_physics(block.width / 2.0),
          *map_scalar_to_physics(block.height / 2.0),
        )
        .collision_groups(InteractionGroups {
          memberships: COLLISION_GROUP_WALL,
          filter: !COLLISION_GROUP_WALL,
          ..Default::default()
        })
        .build(),
      }),

      Object::TouchSensor(touch_sensor) => MapComponent::TouchSensor(TouchSensor {
        collider: cuboid_collider_from_map(
          touch_sensor.x,
          touch_sensor.y,
          touch_sensor.width,
          touch_sensor.height,
          map_height,
        )
        .sensor(true)
        .collision_groups(InteractionGroups {
          memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
          filter: COLLISION_GROUP_PLAYER,
          ..Default::default()
        })
        .build(),
        target_activation: touch_sensor.properties.0.value,
        id: touch_sensor.id,
      }),

      Object::GravitySource(gravity_source) => MapComponent::GravitySource(GravitySource {
        collider: ColliderBuilder::ball(gravity_source.properties.1.value)
          .translation(physics_translation_from_map(
            gravity_source.x,
            gravity_source.y,
            0.0,
            0.0,
            map_height,
          ))
          .sensor(true)
          .collision_groups(GRAVITY_INTERACTION_GROUPS)
          .build(),
        strength: gravity_source.properties.2.value,
        activator_id: gravity_source
          .properties
          .0
          .as_ref()
          .map(|activator_id| activator_id.value),
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
            ..Default::default()
          })
          .build(),
      }),

      Object::ChainSwitch(chain_switch) => {
        let center_position =
          physics_translation_from_map(chain_switch.x, chain_switch.y, 0.0, 0.0, map_height);

        let switch_half_limits = 1.0; // TODO: load from map

        let rotation_vec =
          distance_projection_physics(chain_switch.rotation * PI / 180.0, 1.0).into_vec();

        let initial_activation = chain_switch.properties.0.value;

        let knob_position = center_position + (2.0 * initial_activation - 1.0) * rotation_vec;

        MapComponent::ChainSwitch(ChainSwitch {
          id: chain_switch.id,
          collider: ColliderBuilder::ball(10.0)
            .translation(physics_translation_from_map(
              chain_switch.x,
              chain_switch.y,
              0.0,
              0.0,
              map_height,
            ))
            .sensor(true)
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
              filter: COLLISION_GROUP_PLAYER,
              ..Default::default()
            })
            .build(),
          switch_center: RigidBodyBuilder::dynamic()
            .lock_translations()
            .translation(center_position)
            .build(),
          mount_body: RigidBodyBuilder::dynamic()
            .translation(knob_position)
            .build(),
          switch_joint: PrismaticJointBuilder::new(Unit::new_normalize(rotation_vec))
            .limits([-1.0, 1.0])
            .local_anchor1(vec_zero().into())
            .local_anchor2(vec_zero().into())
            .build(),
        })
      }

      Object::MountPoint(mount_point) => {
        let mount_point_translation =
          physics_translation_from_map(mount_point.x, mount_point.y, 0.0, 0.0, map_height);

        MapComponent::MountPoint(MountPoint {
          rigid_body: RigidBodyBuilder::dynamic()
            .translation(mount_point_translation)
            .build(),
          zone: ColliderBuilder::ball(10.0)
            .sensor(true)
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
              filter: COLLISION_GROUP_PLAYER,
              ..Default::default()
            })
            .build(),
          knob: ColliderBuilder::ball(0.1)
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_WALL,
              filter: Group::empty(),
              ..Default::default()
            })
            .build(),
          id: mount_point.id,
        })
      }

      Object::Or(or) => MapComponent::Or(Or {
        activator_ids: (or.properties.0.value, or.properties.1.value),
        rigid_body: RigidBodyBuilder::dynamic()
          .translation(physics_translation_from_map(
            or.x, or.y, 0.0, 0.0, map_height,
          ))
          .build(),
        id: or.id,
      }),

      Object::And(and) => MapComponent::And(And {
        activator_ids: (and.properties.0.value, and.properties.1.value),
        rigid_body: RigidBodyBuilder::dynamic()
          .translation(physics_translation_from_map(
            and.x, and.y, 0.0, 0.0, map_height,
          ))
          .build(),
        id: and.id,
      }),

      Object::Gate(gate) => MapComponent::Gate(Gate {
        id: gate.id,
        activator_id: gate.properties.0.value,
        rigid_body: RigidBodyBuilder::dynamic()
          .translation(physics_translation_from_map(
            gate.x, gate.y, 0.0, 0.0, map_height,
          ))
          .build(),
      }),

      Object::Locomotor(locomotor) => {
        let top_left = physics_translation_from_map(
          locomotor.x + locomotor.polyline[0].x,
          locomotor.y + locomotor.polyline[0].y,
          0.0,
          0.0,
          map_height,
        );
        let bottom_right = physics_translation_from_map(
          locomotor.x + locomotor.polyline[1].x,
          locomotor.y + locomotor.polyline[1].y,
          0.0,
          0.0,
          map_height,
        );
        let axis = top_left - bottom_right;
        let axis_len = axis.magnitude();

        let reverse_direction = locomotor.properties.1.value;

        let knob_base = RigidBodyBuilder::dynamic().lock_rotations();

        MapComponent::Locomotor(Locomotor {
          id: locomotor.id,
          base: RigidBodyBuilder::dynamic()
            .translation((top_left + bottom_right) / 2.0)
            .lock_rotations()
            .build(),
          joint: PrismaticJointBuilder::new(UnitVector::new_normalize(axis))
            .limits([-axis_len / 2.0, axis_len / 2.0])
            .contacts_enabled(false)
            .build(),
          knob: if reverse_direction {
            knob_base.translation(top_left).build()
          } else {
            knob_base.translation(bottom_right).build()
          },
          reverse_direction,
          activator_id: locomotor.properties.0.value,
        })
      }

      Object::Glue(glue) => MapComponent::Glue(Glue {
        attachments: match &glue.properties {
          MapGlueMapObjects::MultiObject((
            object_1_id,
            object_1_x,
            object_1_y,
            object_2_id,
            object_2_x,
            object_2_y,
          )) => (
            (
              object_1_id.value,
              physics_translation_from_map(
                object_1_x.value,
                object_1_y.value,
                0.0,
                0.0,
                map_height,
              ),
            ),
            (
              Some(object_2_id.value),
              physics_translation_from_map(
                object_2_x.value,
                object_2_y.value,
                0.0,
                0.0,
                map_height,
              ),
            ),
          ),
          MapGlueMapObjects::SingleObject((object_1_id, object_1_x, object_1_y)) => (
            (
              object_1_id.value,
              physics_translation_from_map(
                object_1_x.value,
                object_1_y.value,
                0.0,
                0.0,
                map_height,
              ),
            ),
            (
              None,
              physics_translation_from_map(
                object_1_x.value,
                object_1_y.value,
                0.0,
                0.0,
                map_height,
              ),
            ),
          ),
        },
      }),
      Object::Engine(engine) => MapComponent::Engine(Engine {
        id: engine.id,
        activator_id: engine
          .properties
          .0
          .as_ref()
          .map(|activator_id| activator_id.value),
        rigid_body: RigidBodyBuilder::fixed()
          .translation(physics_translation_from_map(
            engine.x, engine.y, 0.0, 0.0, map_height,
          ))
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
    self
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
                ..Default::default()
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
      .collect()
  }
}

pub struct Map {
  pub top_left: Vector2<f32>,
  pub bottom_right: Vector2<f32>,
  pub colliders: Vec<MapTile>,
  pub player_spawns: Vec<PlayerSpawn>,
  pub enemy_spawns: Vec<EnemySpawn>,
  pub item_pickups: Vec<ItemPickup>,
  pub map_transitions: Vec<MapTransition>,
  pub save_points: Vec<SavePoint>,
  pub blocks: Vec<Block>,
  pub touch_sensors: Vec<TouchSensor>,
  pub gravity_sources: Vec<GravitySource>,
  pub ability_pickups: Vec<AbilityPickup>,
  pub chain_switches: Vec<ChainSwitch>,
  pub mount_points: Vec<MountPoint>,
  pub ands: Vec<And>,
  pub ors: Vec<Or>,
  pub gates: Vec<Gate>,
  pub locomotors: Vec<Locomotor>,
  pub glues: Vec<Glue>,
  pub engines: Vec<Engine>,
}

impl RawMap {
  pub fn into(&self) -> Map {
    let tile_layer = &self.layers.0;

    let colliders = tile_layer.into();

    let entities_layer = &self.layers.1;

    let map_height = tile_layer.height as f32 * 8.0;
    let map_width = tile_layer.width as f32 * 8.0;

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

    let blocks = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Block(gate) = object {
          vec![gate.clone()]
        } else {
          vec![]
        }
      })
      .collect::<Vec<_>>();

    let touch_sensors = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::TouchSensor(touch_sensor) = object {
          vec![touch_sensor.clone()]
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

    let chain_switches = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::ChainSwitch(chain_switch) = object {
          Some(chain_switch)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let mount_points = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::MountPoint(mount_point) = object {
          Some(mount_point)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let ands = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::And(and) = object {
          Some(and)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let ors = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Or(or) = object {
          Some(or)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let gates = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Gate(gate) = object {
          Some(gate)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let locomotors = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Locomotor(locomotor) = object {
          Some(locomotor)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let glues = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Glue(glue) = object {
          Some(glue)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let engines = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Engine(engine) = object {
          Some(engine)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    Map {
      top_left: physics_translation_from_map(0.0, 0.0, 0.0, 0.0, map_height),
      bottom_right: physics_translation_from_map(map_width, map_height, 0.0, 0.0, map_height),
      colliders,
      enemy_spawns,
      player_spawns,
      item_pickups,
      map_transitions,
      save_points,
      blocks,
      touch_sensors,
      gravity_sources,
      ability_pickups,
      chain_switches,
      mount_points,
      ands,
      ors,
      gates,
      locomotors,
      glues,
      engines,
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
  Path::new(&current_dir().unwrap())
    .join(format!("assets/maps/{map_name}.json"))
    .to_str()
    .unwrap()
    .to_string()
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
