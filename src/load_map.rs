use std::{fs, rc::Rc};

use rapier2d::{na::Vector2, prelude::*};
use serde::Deserialize;

use crate::{
  combat::WeaponModuleKind,
  ecs::{ComponentSet, Damageable, Damager, DropHealthOnDestroy, Enemy, Entity},
  f::{Monad, MonadTranslate},
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
#[serde(untagged)]
enum Object {
  EnemySpawn(MapEnemySpawn),
  PlayerSpawn(MapPlayerSpawn),
  ItemPickup(MapItemPickup),
  MapTransition(MapMapTransition),
  SavePoint(MapSavePoint),
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
pub struct Wall {
  pub collider: Collider,
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
        collider: ColliderBuilder::cuboid(
          *map_scalar_to_physics(map_transition.width / 2.0),
          *map_scalar_to_physics(map_transition.height / 2.0),
        )
        .translation(vector![
          *map_scalar_to_physics(map_transition.x + map_transition.width / 2.0),
          *map_scalar_to_physics(map_height - map_transition.y - map_transition.height / 2.0)
        ])
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
    }
  }
}

impl ObjectLayer {
  pub fn into(&self, map_height: f32) -> Vec<MapComponent> {
    return self
      .objects
      .iter()
      .map(|object| object.into(map_height))
      .collect();
  }
}

pub const TILE_DIMENSION_PHYSICS: f32 = 0.8;

const EMPTY: i32 = 0;
const WALL_COLLIDER: i32 = 1;

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

impl TileLayer {
  pub fn into(&self) -> Vec<MapTile> {
    return self
      .data
      .iter()
      .enumerate()
      .map(|(uindex, &tile_data)| {
        let index = uindex.try_into().unwrap();
        if tile_data == WALL_COLLIDER {
          return Some(MapTile::Wall(Wall {
            collider: ColliderBuilder::cuboid(
              TILE_DIMENSION_PHYSICS / 2.0,
              TILE_DIMENSION_PHYSICS / 2.0,
            )
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
            .build(),
          }));
        }
        if tile_data == EMPTY {
          return None;
        }
        todo!()
      })
      .flatten()
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
}

impl RawMap {
  pub fn into(&self) -> Option<Map> {
    let tile_layer = self.layers.iter().find_map(|layer| {
      if let Layer::TileLayer(tile_layer) = layer
        && let TileLayerName::Colliders = tile_layer.name
      {
        Some(tile_layer)
      } else {
        None
      }
    });

    let colliders: Option<Vec<MapTile>> = tile_layer.map(|tile_layer| tile_layer.into());

    let entities_layer = self.layers.iter().find_map(|layer| match layer {
      Layer::ObjectLayer(object_layer) => match object_layer.name {
        ObjectLayerName::Entities => Some(object_layer),
      },
      Layer::TileLayer(_) => None,
    });

    let map_height = tile_layer.bind(|&tile_layer| tile_layer.height as f32 * 8.0);

    let enemy_spawns: Option<Vec<EnemySpawn>> = entities_layer
      .map(|layer| {
        map_height.map(|map_height| {
          layer
            .into(map_height)
            .iter()
            .flat_map(|object| {
              if let MapComponent::Enemy(enemy_spawn) = object {
                vec![enemy_spawn.clone()]
              } else {
                vec![]
              }
            })
            .collect()
        })
      })
      .flatten();

    let player_spawns = entities_layer
      .map(|layer| {
        map_height.map(|map_height| {
          layer
            .into(map_height)
            .iter()
            .flat_map(|object| {
              if let MapComponent::Player(player_spawn) = object {
                vec![player_spawn.clone()]
              } else {
                vec![]
              }
            })
            .collect::<Vec<_>>()
            .clone()
        })
      })
      .flatten();

    let item_pickups = entities_layer
      .map(|layer| {
        map_height.map(|map_height| {
          layer
            .into(map_height)
            .iter()
            .flat_map(|object| {
              if let MapComponent::ItemPickup(item_pickup) = object {
                vec![item_pickup.clone()]
              } else {
                vec![]
              }
            })
            .collect::<Vec<_>>()
        })
      })
      .flatten();

    let map_transitions = entities_layer
      .map(|layer| {
        map_height.map(|map_height| {
          layer
            .into(map_height)
            .iter()
            .flat_map(|object| {
              if let MapComponent::MapTransition(map_transition) = object {
                vec![map_transition.clone()]
              } else {
                vec![]
              }
            })
            .collect::<Vec<_>>()
        })
      })
      .flatten();

    let save_points = entities_layer
      .map(|layer| {
        map_height.map(|map_height| {
          layer
            .into(map_height)
            .iter()
            .flat_map(|object| {
              if let MapComponent::SavePoint(save_point) = object {
                vec![save_point.clone()]
              } else {
                vec![]
              }
            })
            .collect::<Vec<_>>()
        })
      })
      .flatten();

    if let Some(enemy_spawns) = enemy_spawns
      && let Some(player_spawns) = player_spawns
      && let Some(item_pickups) = item_pickups
      && let Some(colliders) = colliders
      && let Some(map_transitions) = map_transitions
      && let Some(save_points) = save_points
    {
      println!("{}", map_transitions[0].collider.translation());
      println!(
        "{}",
        map_transitions[0]
          .collider
          .shape()
          .as_cuboid()
          .unwrap()
          .half_extents
      );
      Some(Map {
        colliders,
        enemy_spawns,
        player_spawns,
        item_pickups,
        map_transitions,
        save_points,
      })
    } else {
      None
    }
  }
}

pub fn load(file_path: &str) -> Option<Map> {
  fs::read_to_string(file_path)
    .translate()
    .bind(|raw_file| (&deser_map(raw_file)).into())
    .flatten()
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
