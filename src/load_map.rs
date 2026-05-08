use std::{env::current_dir, f32::consts::PI, fs, path::Path, rc::Rc};

use rapier2d::{
  na::{Unit, Vector2},
  prelude::*,
};
use rpds::HashTrieMap;
use serde::Deserialize;
use serde_literals::lit_str;

use crate::{
  GameInput,
  balance::BALANCING,
  combat::{WeaponModuleKind, distance_projection_physics},
  ecs::{ComponentSet, Damageable, Damager, DropOnDestroy, Id, SimpleSprite},
  physics::PhysicsSystem,
  sprite,
  system::System,
  units::{PhysicsScalar, PhysicsVector, UnitConvert2, vec_zero},
};

#[derive(Clone, Debug, Deserialize)]
pub struct ColliderLayer {
  pub data: Vec<i32>,
  pub height: i32,
  pub width: i32,
  pub name: String,
}

lit_str!(EnemySpawnTemplatePath, "templates/EnemySpawn.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum EnemySpawnTemplate {
  #[serde(with = "EnemySpawnTemplatePath")]
  Path,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum MapEnemyName {
  /* Dragonspawn */
  Goblin,
  Imp,
  Aranea,
  AraneaQueen,
  /* Angelic Constructs */
  Defender,
  Seeker,
  SeekerGenerator,
  Sniper,
  SniperGenerator,
  LaserGate,
}

#[derive(Clone, Debug, Deserialize)]
enum MapEnemySpawnAraneaEggIdClass {
  EggId,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemySpawnAraneaEggId {
  #[serde(rename = "name")]
  _name: MapEnemySpawnAraneaEggIdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapEnemySpawnPersistDestructionClass {
  PersistDestruction,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemySpawnPersistDestruction {
  #[serde(rename = "name")]
  _name: MapEnemySpawnPersistDestructionClass,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapEnemySpawnProperties {
  AraneaEggId(MapEnemySpawnAraneaEggId),
  PersistDestruction(MapEnemySpawnPersistDestruction),
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemySpawn {
  id: i32,
  x: f32,
  y: f32,
  rotation: Option<f32>,
  name: MapEnemyName,
  #[serde(rename = "template")]
  _template: EnemySpawnTemplate,
  properties: Option<Vec<MapEnemySpawnProperties>>,
}

impl MapEnemySpawn {
  pub fn into(&self, map_height: f32) -> EnemySpawn {
    let translation = PhysicsVector::from_vec(vector![
      self.x * 0.125 * TILE_DIMENSION_PHYSICS,
      (map_height - self.y) * 0.125 * TILE_DIMENSION_PHYSICS
    ]);
    EnemySpawn::new(
      match self.name {
        MapEnemyName::Goblin => EnemySpawnEnemy::Goblin,
        MapEnemyName::Imp => EnemySpawnEnemy::Imp,
        MapEnemyName::Aranea => EnemySpawnEnemy::Aranea(Id {
          id: self
            .properties
            .as_ref()
            .and_then(|properties| {
              properties.iter().find_map(|property| {
                if let MapEnemySpawnProperties::AraneaEggId(egg_id) = property {
                  Some(egg_id.value)
                } else {
                  None
                }
              })
            })
            .unwrap(),
        }),
        MapEnemyName::AraneaQueen => EnemySpawnEnemy::AraneaQueen(Id {
          id: self
            .properties
            .as_ref()
            .and_then(|properties| {
              properties.iter().find_map(|property| {
                if let MapEnemySpawnProperties::AraneaEggId(egg_id) = property {
                  Some(egg_id.value)
                } else {
                  None
                }
              })
            })
            .unwrap(),
        }),
        MapEnemyName::Defender => EnemySpawnEnemy::Defender,
        MapEnemyName::Seeker => EnemySpawnEnemy::Seeker,
        MapEnemyName::SeekerGenerator => EnemySpawnEnemy::SeekerGenerator,
        MapEnemyName::Sniper => EnemySpawnEnemy::Sniper,
        MapEnemyName::SniperGenerator => EnemySpawnEnemy::SniperGenerator,
        MapEnemyName::LaserGate => EnemySpawnEnemy::LaserGate,
      },
      translation.into_vec(),
      -self.rotation.unwrap_or(0.0) * PI / 180.0,
      Some(EnemySpawnPersist {
        id: Id { id: self.id },
        persist_destruction: self
          .properties
          .as_ref()
          .map(|properties| {
            properties.iter().any(|property| {
              if let MapEnemySpawnProperties::PersistDestruction(_) = property {
                true
              } else {
                false
              }
            })
          })
          .unwrap_or(false),
      }),
    )
  }
}

lit_str!(MapAraneaEggTemplatePath, "templates/Aranea Egg.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapAraneaEggTemplate {
  #[serde(with = "MapAraneaEggTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
struct MapAraneaEgg {
  id: i32,
  x: f32,
  y: f32,
  #[serde(rename = "template")]
  _template: MapAraneaEggTemplate,
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

lit_str!(ItemPickupTemplatePath, "templates/ItemPickup.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapItemPickupTemplate {
  #[serde(with = "ItemPickupTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
struct MapItemPickup {
  id: i32,
  x: f32,
  y: f32,
  name: WeaponModuleKind,
  template: MapItemPickupTemplate,
}

lit_str!(MapHealthTankTemplatePath, "templates/Health Tank.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapHealthTankTemplate {
  #[serde(with = "MapHealthTankTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
enum MapHealthTankCapacityClass {
  Capacity,
}

#[derive(Clone, Debug, Deserialize)]
struct MapHealthTankCapacity {
  #[serde(rename = "name")]
  _name: MapHealthTankCapacityClass,
  value: f32,
}

#[derive(Clone, Debug, Deserialize)]
struct MapHealthTank {
  #[serde(rename = "template")]
  _template: MapHealthTankTemplate,
  properties: (MapHealthTankCapacity,),
  id: i32,
  x: f32,
  y: f32,
}

lit_str!(MapManaTankTemplatePath, "templates/Mana Tank.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapManaTankTemplate {
  #[serde(with = "MapManaTankTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
enum MapManaTankRechargeableClass {
  Rechargeable,
}

#[derive(Clone, Debug, Deserialize)]
struct MapManaTankCapacity {
  #[serde(rename = "name")]
  _name: MapManaTankRechargeableClass,
  value: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct MapManaTank {
  #[serde(rename = "template")]
  _template: MapManaTankTemplate,
  properties: (MapManaTankCapacity,),
  id: i32,
  x: f32,
  y: f32,
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
enum MapAllowRotationClass {
  AllowRotation,
}

#[derive(Clone, Debug, Deserialize)]
struct MapAllowRotation {
  name: MapAllowRotationClass,
  value: bool,
}

#[derive(Clone, Debug, Deserialize)]
enum MapGlueClass {
  Glue,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum MapGlueMapProperties {
  MapObject1Id(MapObject1Id),
  MapObject1LocalX(MapObject1LocalX),
  MapObject1LocalY(MapObject1LocalY),
  MapObject2Id(MapObject2Id),
  MapObject2LocalX(MapObject2LocalX),
  MapObject2LocalY(MapObject2LocalY),
  MapAllowRotation(MapAllowRotation),
}

#[derive(Clone, Debug, Deserialize)]
struct MapGlue {
  properties: Vec<MapGlueMapProperties>,
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
#[serde(untagged)]
enum MapGravitySourceProperty {
  ActivatorId(MapActivatorId),
  Radius(MapGravitySourceRadius),
  Strength(MapGravitySourceStrength),
}

#[derive(Clone, Debug, Deserialize)]
enum MapGravitySourceClass {
  GravitySource,
}

#[derive(Clone, Debug, Deserialize)]
struct MapGravitySource {
  id: i32,
  x: f32,
  y: f32,
  properties: Vec<MapGravitySourceProperty>,
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
enum MapEnemyIdClass {
  EnemyId,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemyId {
  #[serde(rename = "name")]
  _name: MapEnemyIdClass,
  value: i32,
}

#[derive(Clone, Debug, Deserialize)]
enum MapEnemyGateClass {
  EnemyGate,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemyGate {
  id: i32,
  x: f32,
  y: f32,
  properties: (MapEnemyId,),
  #[serde(rename = "type")]
  _class: MapEnemyGateClass,
}

#[derive(Clone, Debug, Deserialize)]
enum MapNotClass {
  Not,
}

#[derive(Clone, Debug, Deserialize)]
struct MapNot {
  id: i32,
  x: f32,
  y: f32,
  properties: (MapActivatorId,),
  #[serde(rename = "type")]
  _class: MapNotClass,
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
enum MapContentClass {
  Content,
}

#[derive(Clone, Debug, Deserialize)]
struct MapContent {
  #[serde(rename = "name")]
  _name: MapContentClass,
  value: String,
}

#[derive(Clone, Debug, Deserialize)]
enum MapCreatedAtClass {
  CreatedAt,
}

#[derive(Clone, Debug, Deserialize)]
struct MapCreatedAt {
  #[serde(rename = "name")]
  _name: MapCreatedAtClass,
  value: String,
}

lit_str!(TerminalTemplatePath, "templates/Terminal.tx");

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum TerminalTemplate {
  #[serde(with = "TerminalTemplatePath")]
  Path,
}

#[derive(Clone, Debug, Deserialize)]
struct MapTerminal {
  id: i32,
  x: f32,
  y: f32,
  template: TerminalTemplate,
  properties: (MapContent, MapCreatedAt),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Object {
  EnemySpawn(MapEnemySpawn),
  AraneaEgg(MapAraneaEgg),
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
  EnemyGate(MapEnemyGate),
  Not(MapNot),
  Locomotor(MapLocomotor),
  Glue(MapGlue),
  Engine(MapEngine),
  Terminal(MapTerminal),
  HealthTank(MapHealthTank),
  ManaTank(MapManaTank),
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
pub enum Layer {
  ColliderLayer(ColliderLayer),
  ObjectLayer(ObjectLayer),
}

#[derive(Clone, Debug, Deserialize)]
pub struct RawMap {
  layers: Vec<Layer>,
}

impl RawMap {
  pub fn collider_layer(&self) -> &ColliderLayer {
    self
      .layers
      .iter()
      .find_map(|layer| {
        if let Layer::ColliderLayer(collider_layer) = layer
          && collider_layer.name == "Colliders"
        {
          Some(collider_layer)
        } else {
          None
        }
      })
      .unwrap()
  }

  pub fn tile_bg_layer(&self) -> &ColliderLayer {
    self
      .layers
      .iter()
      .find_map(|layer| {
        if let Layer::ColliderLayer(collider_layer) = layer
          && collider_layer.name == "TilesBG"
        {
          Some(collider_layer)
        } else {
          None
        }
      })
      .unwrap()
  }

  pub fn tile_layer(&self) -> &ColliderLayer {
    self
      .layers
      .iter()
      .find_map(|layer| {
        if let Layer::ColliderLayer(collider_layer) = layer
          && collider_layer.name == "Tiles"
        {
          Some(collider_layer)
        } else {
          None
        }
      })
      .unwrap()
  }

  pub fn object_layer(&self) -> &ObjectLayer {
    self
      .layers
      .iter()
      .find_map(|layer| {
        if let Layer::ObjectLayer(object_layer) = layer {
          Some(object_layer)
        } else {
          None
        }
      })
      .unwrap()
  }
}

#[derive(Deserialize)]
pub struct WorldMap {
  #[serde(rename = "fileName")]
  file_name: String,
  height: f32,
  width: f32,
  x: f32,
  y: f32,
}

impl WorldMap {
  pub fn with_tiles(&self, tiles: Vec<i32>) -> WorldMapWithTiles {
    WorldMapWithTiles {
      tiles,
      width: self.width,
      height: self.height,
      x: self.x,
      y: self.y,
    }
  }
}

#[derive(Deserialize)]
pub struct World {
  maps: Vec<WorldMap>,
}

pub struct WorldMapWithTiles {
  pub tiles: Vec<i32>,
  pub width: f32,
  pub height: f32,
  pub x: f32,
  pub y: f32,
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
pub const COLLISION_GROUP_GRAVITY: Group = Group::GROUP_8;

pub const GRAVITY_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_GRAVITY,
  filter: Group::all(),
  test_mode: InteractionTestMode::And,
};

pub const PLAYER_INTERACTIBLE_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
  filter: COLLISION_GROUP_PLAYER,
  test_mode: InteractionTestMode::And,
};

pub const PLAYER_PROJECTILE_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_PLAYER_PROJECTILE,
  filter: COLLISION_GROUP_ENEMY
    .union(COLLISION_GROUP_WALL)
    .union(COLLISION_GROUP_GRAVITY),
  test_mode: InteractionTestMode::And,
};

pub const ENEMY_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_ENEMY,
  filter: COLLISION_GROUP_PLAYER
    .union(COLLISION_GROUP_PLAYER_PROJECTILE)
    .union(COLLISION_GROUP_WALL)
    .union(COLLISION_GROUP_GRAVITY),
  test_mode: InteractionTestMode::And,
};

pub const ENEMY_PROJECTILE_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_ENEMY_PROJECTILE,
  filter: COLLISION_GROUP_PLAYER
    .union(COLLISION_GROUP_WALL)
    .union(COLLISION_GROUP_GRAVITY),
  test_mode: InteractionTestMode::And,
};

pub const RAYCAST_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_ENEMY,
  filter: COLLISION_GROUP_PLAYER.union(COLLISION_GROUP_WALL),
  test_mode: InteractionTestMode::And,
};

pub const PLAYER_INTERACTION_GROUPS: InteractionGroups = InteractionGroups {
  memberships: COLLISION_GROUP_PLAYER,
  filter: COLLISION_GROUP_WALL
    .union(COLLISION_GROUP_ENEMY)
    .union(COLLISION_GROUP_ENEMY_PROJECTILE)
    .union(COLLISION_GROUP_PLAYER_INTERACTIBLE)
    .union(COLLISION_GROUP_GRAVITY),
  test_mode: InteractionTestMode::And,
};

pub struct EnemySpawnColliderHandles {
  pub hitboxes: Vec<ColliderHandle>,
  pub hurtboxes: Vec<ColliderHandle>,
}

#[derive(Clone)]
pub enum EnemySpawnEnemy {
  /* Dragonspawn */
  Goblin,
  Imp,
  Aranea(Id),
  AraneaQueen(Id),
  /* Angelic Constructs */
  Defender,
  Seeker,
  SeekerGenerator,
  Sniper,
  SniperGenerator,
  LaserGate,
}

#[derive(Clone)]
pub struct EnemySpawnPersist {
  pub id: Id,
  pub persist_destruction: bool,
}

#[derive(Clone)]
pub struct EnemySpawn {
  pub name: EnemySpawnEnemy,
  pub hitboxes: Vec<Collider>,
  pub hurtboxes: Vec<Collider>,
  pub rigid_body: RigidBody,
  pub persist: Option<EnemySpawnPersist>,
}

impl EnemySpawn {
  pub fn new(
    name: EnemySpawnEnemy,
    translation: Vector2<f32>,
    rotation: f32,
    persist: Option<EnemySpawnPersist>,
  ) -> Self {
    let hitboxes = hitboxes_from_enemy_name(&name);
    let hurtboxes = hurtboxes_from_enemy_name(&name);
    let mut rigid_body = RigidBodyBuilder::dynamic()
      .translation(translation)
      .rotation(rotation)
      .build();
    rigid_body.wake_up(true);
    EnemySpawn {
      name,
      hitboxes,
      hurtboxes,
      rigid_body,
      persist,
    }
  }

  pub fn into_entity_components(
    &self,
    collider_handles: EnemySpawnColliderHandles,
  ) -> ComponentSet {
    let EnemySpawnColliderHandles {
      hitboxes,
      hurtboxes,
    } = collider_handles;
    match self.name {
      EnemySpawnEnemy::Goblin => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          health: 60.0,
          max_health: 60.0,
          destroy_on_zero_health: true,
          hurtboxes,
          ..Default::default()
        })
        .insert(Damager {
          damage: BALANCING.enemies.goblin.damage,
          hitboxes,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 20.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::Imp => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 50.0,
          max_health: 50.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 10.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 15.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::Aranea(_) => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 80.0,
          max_health: 80.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 10.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 15.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::AraneaQueen(_) => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: BALANCING.enemies.aranea_queen.status_effect_threshold,
          hurtboxes,
          health: BALANCING.enemies.aranea_queen.max_health,
          max_health: BALANCING.enemies.aranea_queen.max_health,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: BALANCING.enemies.aranea_queen.contact_damage,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 40.0,
          chance_health: 1.0,
          mana_amount: 0.0,
          chance_mana: 0.0,
        }),
      EnemySpawnEnemy::Defender => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 100.0,
          max_health: 100.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 10.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 20.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::Seeker => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 30.0,
          max_health: 30.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 25.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 10.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::SeekerGenerator => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 120.0,
          max_health: 120.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 10.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 35.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::Sniper => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 60.0,
          max_health: 60.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 10.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 35.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::SniperGenerator => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 20.0,
          hurtboxes,
          health: 140.0,
          max_health: 140.0,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(Damager {
          hitboxes,
          damage: 10.0,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 35.0,
          chance_health: 0.3,
          mana_amount: 2.0,
          chance_mana: 0.2,
        }),
      EnemySpawnEnemy::LaserGate => ComponentSet::new()
        .insert(Damageable {
          status_effect_threshold: 10.0,
          hurtboxes,
          health: BALANCING.enemies.laser_gate.health,
          max_health: BALANCING.enemies.laser_gate.health,
          destroy_on_zero_health: true,
          ..Default::default()
        })
        .insert(DropOnDestroy {
          health_amount: 5.0,
          chance_health: 0.3,
          mana_amount: 1.0,
          chance_mana: 0.2,
        })
        .insert(SimpleSprite {
          kind: sprite::LaserGate,
        }),
    }
  }
}

#[derive(Clone)]
pub struct AraneaEgg {
  pub id: i32,
  pub collider: Collider,
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
  pub enemy_block_collider: Collider,
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
pub struct EnemyGate {
  pub rigid_body: RigidBody,
  pub id: Id,
  pub enemy_id: Id,
}

#[derive(Clone)]
pub struct Not {
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
  pub allow_rotation: bool,
}

#[derive(Clone)]
pub struct Engine {
  pub id: i32,
  pub activator_id: Option<i32>,
  pub rigid_body: RigidBody,
}

#[derive(Clone)]
pub struct Terminal {
  pub id: i32,
  pub collider: Collider,
  pub content: String,
  pub created_at: String,
}

#[derive(Clone)]
pub struct HealthTank {
  pub id: i32,
  pub collider: Collider,
  pub capacity: f32,
}

#[derive(Clone)]
pub struct ManaTank {
  pub id: i32,
  pub collider: Collider,
  pub rechargeable: bool,
}

#[derive(Clone)]
pub struct Wall {
  pub collider: Collider,
  pub damaging: Option<f32>,
  pub damageable: Option<f32>,
}

fn hurtboxes_from_enemy_name(name: &EnemySpawnEnemy) -> Vec<Collider> {
  let collider_builders = match name {
    EnemySpawnEnemy::Goblin => vec![ColliderBuilder::cuboid(0.6, 0.6)],
    EnemySpawnEnemy::Imp => vec![ColliderBuilder::cuboid(
      BALANCING.enemies.imp.width,
      BALANCING.enemies.imp.height,
    )],
    EnemySpawnEnemy::Aranea(_) => vec![ColliderBuilder::cuboid(0.3, 0.3)],
    EnemySpawnEnemy::AraneaQueen(_) => vec![ColliderBuilder::cuboid(
      BALANCING.enemies.aranea_queen.colliders_side_length,
      BALANCING.enemies.aranea_queen.colliders_side_length,
    )],
    EnemySpawnEnemy::Defender => vec![ColliderBuilder::cuboid(0.5, 0.5)],
    EnemySpawnEnemy::Seeker => vec![ColliderBuilder::cuboid(0.2, 0.2).mass(1.0)],
    EnemySpawnEnemy::SeekerGenerator => vec![ColliderBuilder::cuboid(0.7, 0.7)],
    EnemySpawnEnemy::Sniper => vec![ColliderBuilder::cuboid(0.2, 0.2).mass(1.0)],
    EnemySpawnEnemy::SniperGenerator => vec![ColliderBuilder::cuboid(0.7, 0.7).mass(50.0)],
    EnemySpawnEnemy::LaserGate => vec![ColliderBuilder::ball(0.1).mass(1.0)],
  };

  collider_builders
    .into_iter()
    .map(|collider_builder| {
      collider_builder
        .collision_groups(ENEMY_INTERACTION_GROUPS)
        .build()
    })
    .collect()
}

fn hitboxes_from_enemy_name(name: &EnemySpawnEnemy) -> Vec<Collider> {
  let collider_builders = match name {
    EnemySpawnEnemy::Goblin => vec![ColliderBuilder::cuboid(0.6, 0.6)],
    EnemySpawnEnemy::Imp => vec![ColliderBuilder::cuboid(
      BALANCING.enemies.imp.width,
      BALANCING.enemies.imp.height,
    )],
    EnemySpawnEnemy::Aranea(_) => vec![ColliderBuilder::cuboid(0.3, 0.3)],
    EnemySpawnEnemy::AraneaQueen(_) => vec![ColliderBuilder::cuboid(
      BALANCING.enemies.aranea_queen.colliders_side_length,
      BALANCING.enemies.aranea_queen.colliders_side_length,
    )],
    EnemySpawnEnemy::Defender => vec![ColliderBuilder::cuboid(0.5, 0.5)],
    EnemySpawnEnemy::Seeker => vec![ColliderBuilder::cuboid(0.2, 0.2).mass(1.0)],
    EnemySpawnEnemy::SeekerGenerator => vec![ColliderBuilder::cuboid(0.7, 0.7)],
    EnemySpawnEnemy::Sniper => vec![ColliderBuilder::cuboid(0.2, 0.2).mass(1.0)],
    EnemySpawnEnemy::SniperGenerator => vec![ColliderBuilder::cuboid(0.7, 0.7)],
    EnemySpawnEnemy::LaserGate => vec![],
  };

  collider_builders
    .into_iter()
    .map(|collider_builder| {
      collider_builder
        .collision_groups(ENEMY_INTERACTION_GROUPS)
        .build()
    })
    .collect()
}

#[derive(Clone)]
pub enum MapComponent {
  Player(PlayerSpawn),
  Enemy(EnemySpawn),
  AraneaEgg(AraneaEgg),
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
  EnemyGate(EnemyGate),
  Not(Not),
  Locomotor(Locomotor),
  Glue(Glue),
  Engine(Engine),
  Terminal(Terminal),
  HealthTank(HealthTank),
  ManaTank(ManaTank),
}

fn map_scalar_to_physics(scalar: f32) -> PhysicsScalar {
  PhysicsScalar(scalar * 0.125 * TILE_DIMENSION_PHYSICS)
}

pub fn physics_scalar_to_map(scalar: PhysicsScalar) -> f32 {
  *scalar * 8.0 / TILE_DIMENSION_PHYSICS
}

impl Object {
  pub fn into(&self, map_height: f32) -> MapComponent {
    match self {
      Object::EnemySpawn(enemy_spawn) => MapComponent::Enemy(enemy_spawn.into(map_height)),

      Object::AraneaEgg(aranea_egg) => MapComponent::AraneaEgg(AraneaEgg {
        id: aranea_egg.id,
        collider: ColliderBuilder::ball(0.5)
          .translation(physics_translation_from_map(
            aranea_egg.x,
            aranea_egg.y,
            0.0,
            0.0,
            map_height,
          ))
          .sensor(true)
          .build(),
      }),

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

      Object::MapTransition(map_transition) => {
        let collider_base = cuboid_collider_from_map(
          map_transition.x,
          map_transition.y,
          map_transition.width,
          map_transition.height,
          map_height,
        );

        MapComponent::MapTransition(MapTransition {
          target_player_spawn_id: map_transition.properties.0.value,
          map_name: map_transition.name.clone(),
          collider: collider_base
            .clone()
            .sensor(true)
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_PLAYER_INTERACTIBLE,
              filter: COLLISION_GROUP_PLAYER,
              ..Default::default()
            })
            .build(),
          enemy_block_collider: collider_base
            .clone()
            .collision_groups(InteractionGroups {
              memberships: COLLISION_GROUP_WALL,
              filter: !COLLISION_GROUP_PLAYER,
              ..Default::default()
            })
            .build(),
        })
      }

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
            block.x,
            block.y,
            block.width,
            block.height,
            map_height,
          ))
          .build(),
        collider: ColliderBuilder::cuboid(
          *map_scalar_to_physics(block.width / 2.0),
          *map_scalar_to_physics(block.height / 2.0),
        )
        .collision_groups(InteractionGroups {
          memberships: COLLISION_GROUP_WALL,
          filter: Group::ALL,
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

      Object::GravitySource(gravity_source) => {
        let radius = gravity_source
          .properties
          .iter()
          .find_map(|property| {
            if let MapGravitySourceProperty::Radius(radius) = property {
              Some(radius.value)
            } else {
              None
            }
          })
          .unwrap_or_else(|| {
            panic!(
              "Gravity source {} provided with no radius",
              gravity_source.id
            )
          });

        let strength = gravity_source
          .properties
          .iter()
          .find_map(|property| {
            if let MapGravitySourceProperty::Strength(strength) = property {
              Some(strength.value)
            } else {
              None
            }
          })
          .unwrap_or_else(|| {
            panic!(
              "Gravity source {} strength provided with no strength",
              gravity_source.id
            )
          });

        let maybe_activator_id = gravity_source.properties.iter().find_map(|property| {
          if let MapGravitySourceProperty::ActivatorId(activator_id) = property {
            Some(activator_id.value)
          } else {
            None
          }
        });

        MapComponent::GravitySource(GravitySource {
          collider: ColliderBuilder::ball(radius)
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
          strength,
          activator_id: maybe_activator_id,
        })
      }

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

      Object::EnemyGate(enemy_gate) => MapComponent::EnemyGate(EnemyGate {
        id: Id { id: enemy_gate.id },
        enemy_id: Id {
          id: enemy_gate.properties.0.value,
        },
        rigid_body: RigidBodyBuilder::dynamic()
          .translation(physics_translation_from_map(
            enemy_gate.x,
            enemy_gate.y,
            0.0,
            0.0,
            map_height,
          ))
          .build(),
      }),

      Object::Not(gate) => MapComponent::Not(Not {
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
            knob_base.translation(bottom_right).build()
          } else {
            knob_base.translation(top_left).build()
          },
          reverse_direction,
          activator_id: locomotor.properties.0.value,
        })
      }

      Object::Glue(glue) => {
        let object_1_id = glue
          .properties
          .iter()
          .find_map(|property| {
            if let MapGlueMapProperties::MapObject1Id(x) = property {
              Some(x)
            } else {
              None
            }
          })
          .unwrap();

        let object_1_x = glue
          .properties
          .iter()
          .find_map(|property| {
            if let MapGlueMapProperties::MapObject1LocalX(x) = property {
              Some(x)
            } else {
              None
            }
          })
          .unwrap();

        let object_1_y = glue
          .properties
          .iter()
          .find_map(|property| {
            if let MapGlueMapProperties::MapObject1LocalY(x) = property {
              Some(x)
            } else {
              None
            }
          })
          .unwrap();

        let object_2_id = glue.properties.iter().find_map(|property| {
          if let MapGlueMapProperties::MapObject2Id(x) = property {
            Some(x)
          } else {
            None
          }
        });

        let object_2_x = glue.properties.iter().find_map(|property| {
          if let MapGlueMapProperties::MapObject2LocalX(x) = property {
            Some(x)
          } else {
            None
          }
        });

        let object_2_y = glue.properties.iter().find_map(|property| {
          if let MapGlueMapProperties::MapObject2LocalY(x) = property {
            Some(x)
          } else {
            None
          }
        });

        let allow_rotation = glue
          .properties
          .iter()
          .find_map(|property| {
            if let MapGlueMapProperties::MapAllowRotation(x) = property {
              Some(x)
            } else {
              None
            }
          })
          .map(|allow_rotation| allow_rotation.value)
          .unwrap_or(false);

        let attachments = (
          (
            object_1_id.value,
            vector![
              *map_scalar_to_physics(object_1_x.value),
              *map_scalar_to_physics(object_1_y.value),
            ],
          ),
          if let Some(object_2_id) = object_2_id
            && let Some(object_2_x) = object_2_x
            && let Some(object_2_y) = object_2_y
          {
            (
              Some(object_2_id.value),
              vector![
                *map_scalar_to_physics(object_2_x.value),
                *map_scalar_to_physics(object_2_y.value),
              ],
            )
          } else {
            (
              None,
              vector![
                *map_scalar_to_physics(object_1_x.value),
                *map_scalar_to_physics(object_1_y.value),
              ],
            )
          },
        );

        MapComponent::Glue(Glue {
          attachments,
          allow_rotation,
        })
      }
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
      Object::Terminal(terminal) => MapComponent::Terminal(Terminal {
        id: terminal.id,
        collider: ColliderBuilder::ball(0.5)
          .sensor(true)
          .collision_groups(PLAYER_INTERACTIBLE_GROUPS)
          .translation(physics_translation_from_map(
            terminal.x, terminal.y, 0.0, 0.0, map_height,
          ))
          .build(),
        content: terminal.properties.0.value.clone(),
        created_at: terminal.properties.1.value.clone(),
      }),
      Object::HealthTank(health_tank) => MapComponent::HealthTank(HealthTank {
        id: health_tank.id,
        collider: ColliderBuilder::ball(1.0)
          .sensor(true)
          .collision_groups(PLAYER_INTERACTIBLE_GROUPS)
          .translation(physics_translation_from_map(
            health_tank.x,
            health_tank.y,
            0.0,
            0.0,
            map_height,
          ))
          .build(),
        capacity: health_tank.properties.0.value,
      }),
      Object::ManaTank(mana_tank) => MapComponent::ManaTank(ManaTank {
        id: mana_tank.id,
        collider: ColliderBuilder::ball(1.0)
          .sensor(true)
          .collision_groups(PLAYER_INTERACTIBLE_GROUPS)
          .translation(physics_translation_from_map(
            mana_tank.x,
            mana_tank.y,
            0.0,
            0.0,
            map_height,
          ))
          .build(),
        rechargeable: mana_tank.properties.0.value,
      }),
    }
  }
}

pub fn physics_translation_from_map(
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

pub const TILE_DIMENSION_PHYSICS: f32 = 1.0;

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

impl ColliderLayer {
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
                filter: Group::ALL,
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

#[derive(Clone)]
pub struct Map {
  pub top_left: Vector2<f32>,
  pub bottom_right: Vector2<f32>,
  pub colliders: Vec<MapTile>,
  pub player_spawns: Vec<PlayerSpawn>,
  pub enemy_spawns: Vec<EnemySpawn>,
  pub aranea_eggs: HashTrieMap<Id, AraneaEgg>,
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
  pub enemy_gates: Vec<EnemyGate>,
  pub nots: Vec<Not>,
  pub locomotors: Vec<Locomotor>,
  pub glues: Vec<Glue>,
  pub engines: Vec<Engine>,
  pub terminals: Vec<Terminal>,
  pub health_tanks: Vec<HealthTank>,
  pub mana_tanks: Vec<ManaTank>,
}

impl RawMap {
  pub fn as_map(&self) -> Map {
    let collider_layer = self.collider_layer();

    let colliders = collider_layer.into();

    let entities_layer = self.object_layer();

    let map_height = collider_layer.height as f32 * 8.0;
    let map_width = collider_layer.width as f32 * 8.0;

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

    let aranea_eggs = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::AraneaEgg(aranea_egg) = object {
          vec![(Id { id: aranea_egg.id }, aranea_egg.clone())]
        } else {
          vec![]
        }
      })
      .collect::<HashTrieMap<_, _>>();

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

    let enemy_gates = converted_entities
      .iter()
      .filter_map(|object| {
        if let MapComponent::EnemyGate(enemy_gate) = object {
          Some(enemy_gate)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let nots = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Not(not) = object {
          Some(not)
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

    let terminals = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::Terminal(terminal) = object {
          Some(terminal)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let health_tanks = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::HealthTank(health_tank) = object {
          Some(health_tank)
        } else {
          None
        }
      })
      .cloned()
      .collect::<Vec<_>>();

    let mana_tanks = converted_entities
      .iter()
      .flat_map(|object| {
        if let MapComponent::ManaTank(mana_tank) = object {
          Some(mana_tank)
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
      aranea_eggs,
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
      enemy_gates,
      nots,
      locomotors,
      glues,
      engines,
      terminals,
      health_tanks,
      mana_tanks,
    }
  }
}

pub fn load(data: &str) -> Map {
  deser_map(data).as_map()
}

pub fn load_world() -> Option<World> {
  let world_path = Path::new(&current_dir().unwrap())
    .join("assets/maps/generated/world.world")
    .to_str()
    .unwrap()
    .to_string();

  fs::read_to_string(world_path)
    .ok()
    .as_ref()
    .map(|raw_file| serde_json::from_str(raw_file).expect("JSON was not well-formatted"))
}

#[derive(Clone)]
pub struct MapSystem {
  pub map: Map,
  pub raw_map: Rc<RawMap>,
  pub world: Rc<World>,
  pub current_map_name: String,
  pub target_player_spawn_id: i32,
  pub map_registry: Rc<HashTrieMap<String, WorldMapWithTiles>>,
  pub new_map: bool,
}

fn map_read_path(map_name: &String) -> String {
  Path::new(&current_dir().unwrap())
    .join(format!("assets/maps/generated/{map_name}.json"))
    .to_str()
    .unwrap()
    .to_string()
}

impl System for MapSystem {
  type Input = GameInput;
  fn start(
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    let save_data = &ctx.input.save_data;

    let world = Rc::new(load_world().unwrap());

    let map_registry = Rc::new(
      save_data
        .visited_maps
        .iter()
        .map(|map_name| {
          let map_data = fs::read_to_string(map_read_path(map_name)).unwrap();

          let map_raw = deser_map(&map_data);

          let tiles = map_raw.collider_layer().data.clone();

          let world_map = world
            .maps
            .iter()
            .find(|&world_map| world_map.file_name == format!("{}.json", map_name))
            .unwrap();

          (map_name.to_string(), world_map.with_tiles(tiles))
        })
        .collect::<HashTrieMap<_, _>>(),
    );

    let map_data = fs::read_to_string(map_read_path(&save_data.map_name)).unwrap();

    let raw_map = deser_map(&map_data);
    let map = raw_map.as_map();

    Rc::new(Self {
      world,
      map,
      raw_map: Rc::new(raw_map),
      new_map: true,
      map_registry,
      current_map_name: save_data.map_name.clone(),
      target_player_spawn_id: save_data.player_spawn_id,
    })
  }

  fn update(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>> {
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    if let Some((map_name, id)) = physics_system.load_new_map.as_ref() {
      let map_data = fs::read_to_string(map_read_path(map_name)).unwrap();

      let raw_map = deser_map(&map_data);
      let tiles = raw_map.collider_layer().data.clone();

      let world_map = self
        .world
        .maps
        .iter()
        .find(|&world_map| world_map.file_name == format!("{}.json", map_name))
        .unwrap();

      let map_registry = Rc::new(
        self
          .map_registry
          .insert(map_name.to_string(), world_map.with_tiles(tiles)),
      );

      Rc::new(Self {
        map: raw_map.as_map(),
        raw_map: Rc::new(raw_map),
        new_map: true,
        map_registry,
        current_map_name: map_name.clone(),
        target_player_spawn_id: *id,
        world: Rc::clone(&self.world),
      })
    } else {
      Rc::new(Self {
        current_map_name: self.current_map_name.clone(),
        map: self.map.clone(),
        raw_map: Rc::clone(&self.raw_map),
        new_map: false,
        map_registry: Rc::clone(&self.map_registry),
        target_player_spawn_id: self.target_player_spawn_id,
        world: Rc::clone(&self.world),
      })
    }
  }

  fn fixed_update(
    &self,
    _: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    Rc::new(self.clone())
  }
}
