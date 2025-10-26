use std::{fs, rc::Rc};

use rapier2d::{na::Vector2, prelude::*};
use serde::Deserialize;

use crate::{
  entity::{EnemySpawn, PlayerSpawn, Wall, collider_from_enemy_name},
  f::{Monad, MonadTranslate},
  system::System,
  units::{PhysicsScalar, PhysicsVector, ScreenVector, UnitConvert, UnitConvert2, vec_zero},
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
pub enum EnemyName {
  Defender,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapEnemySpawnClass {
  EnemySpawn,
}

#[derive(Clone, Debug, Deserialize)]
struct MapEnemySpawn {
  x: f32,
  y: f32,
  name: EnemyName,
  #[serde(rename = "type")]
  _class: MapEnemySpawnClass,
}

#[derive(Clone, Debug, Deserialize)]
pub enum MapPlayerSpawnClass {
  PlayerSpawn,
}

#[derive(Clone, Debug, Deserialize)]
struct MapPlayerSpawn {
  x: f32,
  y: f32,
  #[serde(rename = "type")]
  _class: MapPlayerSpawnClass,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum Object {
  EnemySpawn(MapEnemySpawn),
  PlayerSpawn(MapPlayerSpawn),
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
  return serde_json::from_str(raw).expect("JSON was not well-formatted");
}

pub const COLLISION_GROUP_WALL: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;
pub const COLLISION_GROUP_PLAYER_PROJECTILE: Group = Group::GROUP_3;
pub const COLLISION_GROUP_ENEMY: Group = Group::GROUP_4;
pub const COLLISION_GROUP_ENEMY_PROJECTILE: Group = Group::GROUP_5;

pub enum MapComponent {
  Player(PlayerSpawn),
  Enemy(EnemySpawn),
}

//    (index % map_dimensions.x) as f32 * TILE_DIMENSION_PHYSICS,
//    (map_dimensions.y - (index / map_dimensions.x)) as f32 * TILE_DIMENSION_PHYSICS

impl Object {
  pub fn into(&self, map_height: f32) -> MapComponent {
    match self {
      Object::EnemySpawn(enemy_spawn) => {
        let translation = PhysicsVector::from_vec(vector![
          enemy_spawn.x * 0.125 * TILE_DIMENSION_PHYSICS,
          (map_height - enemy_spawn.y) * 0.125 * TILE_DIMENSION_PHYSICS
        ]);
        let collider = collider_from_enemy_name(enemy_spawn.name.clone());
        let rigid_body = RigidBodyBuilder::fixed()
          .translation(translation.into_vec())
          .build();
        MapComponent::Enemy(EnemySpawn {
          name: enemy_spawn.name.clone(),
          collider,
          rigid_body,
        })
      }
      Object::PlayerSpawn(player_spawn) => MapComponent::Player(PlayerSpawn {
        translation: PhysicsVector::from_vec(vector![
          player_spawn.x * 0.125 * TILE_DIMENSION_PHYSICS,
          (map_height - player_spawn.y) * 0.125 * TILE_DIMENSION_PHYSICS
        ]),
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
  return vector![
    (index % map_dimensions.x) as f32 * TILE_DIMENSION_PHYSICS,
    (map_dimensions.y - (index / map_dimensions.x)) as f32 * TILE_DIMENSION_PHYSICS
  ];
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
      .filter(Option::is_some)
      .map(Option::unwrap)
      .collect();
  }
}

pub struct Map {
  pub colliders: Vec<MapTile>,
  pub player_spawn: PlayerSpawn,
  pub enemy_spawns: Vec<EnemySpawn>,
}

impl RawMap {
  pub fn into(&self) -> Option<Map> {
    let tile_layer = self
      .layers
      .iter()
      .find(|&layer| match layer {
        Layer::ObjectLayer(_) => false,
        Layer::TileLayer(tile_layer) => match tile_layer.name {
          TileLayerName::Colliders => true,
        },
      })
      .bind(|&found_layer| match found_layer {
        Layer::ObjectLayer(_) => None,
        Layer::TileLayer(tile_layer) => Some(tile_layer),
      })
      .flatten();

    let colliders = tile_layer.bind(|&tile_layer| tile_layer.into());

    let entities_layer = self
      .layers
      .iter()
      .find(|&layer| match layer {
        Layer::ObjectLayer(object_layer) => match object_layer.name {
          ObjectLayerName::Entities => true,
        },
        Layer::TileLayer(_) => false,
      })
      .bind(|found_layer| match found_layer {
        Layer::ObjectLayer(object_layer) => Some(object_layer),
        Layer::TileLayer(_) => None,
      })
      .flatten();

    let map_height = tile_layer.bind(|&tile_layer| tile_layer.height as f32 * 8.0);

    let enemy_spawns: Option<Vec<EnemySpawn>> = entities_layer
      .bind(|&layer| {
        map_height.bind(|map_height| {
          layer
            .into(*map_height)
            .iter()
            .flat_map(|object| match object {
              MapComponent::Enemy(enemy_spawn) => Vec::from([enemy_spawn.clone()]),
              MapComponent::Player(_) => Vec::new(),
            })
            .collect()
        })
      })
      .flatten();

    let player_spawn = entities_layer
      .bind(|&layer| {
        map_height.bind(|map_height| {
          layer
            .into(*map_height)
            .iter()
            .flat_map(|object| match object {
              MapComponent::Enemy(_) => Vec::new(),
              MapComponent::Player(player_spawn) => Vec::from([player_spawn.clone()]),
            })
            .collect::<Vec<_>>()[0]
            .clone()
        })
      })
      .flatten();

    enemy_spawns
      .bind(|enemy_spawns| {
        player_spawn.clone().bind(|player_spawn| {
          colliders.clone().bind(|colliders| Map {
            colliders: colliders.clone(),
            enemy_spawns: enemy_spawns.clone(),
            player_spawn: player_spawn.clone(),
          })
        })
      })
      .flatten()
      .flatten()
  }
}

pub fn load(file_path: &str) -> Option<Map> {
  fs::read_to_string(file_path)
    .translate()
    .bind(|raw_file| (&deser_map(&raw_file)).into())
    .flatten()
}

pub struct MapSystem {
  pub map: Rc<Map>,
}

const MAP_READ_PATH: &str = "./assets/maps/map1.json";

impl System for MapSystem {
  fn start(_: crate::system::Context) -> std::rc::Rc<dyn System>
  where
    Self: Sized,
  {
    let map = load(MAP_READ_PATH).unwrap();
    return Rc::new(Self { map: Rc::new(map) });
  }

  fn run(&self, _: &crate::system::Context) -> std::rc::Rc<dyn System> {
    return Rc::new(Self {
      map: Rc::clone(&self.map),
    });
  }
}
