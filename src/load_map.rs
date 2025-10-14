use std::fs;

use rapier2d::{na::Vector2, prelude::*};
use serde::Deserialize;

use crate::{entity::{Enemy, Entity, Player, Wall}, load_map, units::MapVector};

#[derive(Debug, Deserialize)]
enum TileLayerName {
    Colliders
}

#[derive(Debug, Deserialize)]
struct TileLayer {
    data: Vec<i32>,
    height: i32,
    width: i32,
    name: TileLayerName,
}

#[derive(Debug, Deserialize)]
enum ObjectName {
    PlayerStart,
    Enemy
}

#[derive(Debug, Deserialize)]
struct Object {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    name: ObjectName,
}

#[derive(Debug, Deserialize)]
enum ObjectLayerName {
    Entities
}

#[derive(Debug, Deserialize)]
struct ObjectLayer {
    objects: Vec<Object>,
    name: ObjectLayerName,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Layer {
    TileLayer(TileLayer),
    ObjectLayer(ObjectLayer),
}

#[derive(Debug, Deserialize)]
struct RawMap {
    layers: Vec<Layer>
}

pub fn deser_map(raw: &str) -> RawMap {
    return serde_json::from_str(raw).expect("JSON was not well-formatted");
}

pub const COLLISION_GROUP_WALL: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;

pub enum MapComponent {
    Player(Player),
    Enemy(Enemy)
}

impl Object {
    pub fn into(&self) -> MapComponent {
        let rigid_body = RigidBodyBuilder::dynamic()
            .translation(*MapVector::new(vector![self.x, self.y]).into_physics_pos())
            .build();
        let ball_collider = ColliderBuilder::ball(0.5).restitution(0.7)
            .collision_groups(InteractionGroups { memberships: COLLISION_GROUP_PLAYER, filter: COLLISION_GROUP_WALL })
            .build();
        let entity = Entity {
            rigid_body,
            collider: ball_collider,
        };

        match self.name {
            ObjectName::Enemy => {
                return MapComponent::Enemy(Enemy {
                    entity
                })
            },
            ObjectName::PlayerStart => {
                return MapComponent::Player(Player {
                    entity
                })
            }
        } 
    }
}

impl ObjectLayer {
    pub fn into(&self) -> Vec<MapComponent> {
        return self.objects.iter().map(Object::into).collect();
    }
}

const TILE_WIDTH: f32 = 0.3;
const TILE_HEIGHT: f32 = 0.3;

const EMPTY: i32 = 0;
const WALL_COLLIDER: i32 = 1;

pub enum MapTile {
    Wall(Wall),
}

pub fn translation_vector_from_index(index: i32, map_dimensions: Vector2<i32>) -> Vector<f32> {
    return vector![(index % map_dimensions.x) as f32 * TILE_WIDTH, (map_dimensions.y - (index / map_dimensions.x)) as f32 * TILE_HEIGHT];
}

impl TileLayer {
    pub fn into(&self) -> Vec<MapTile> {
        return self.data
            .iter()
            .enumerate()
            .map(|(uindex, &tile_data)| {
                let index = uindex.try_into().unwrap();
                if tile_data == WALL_COLLIDER {
                    return Some(MapTile::Wall( Wall {
                        collider: ColliderBuilder::cuboid(TILE_WIDTH / 2.0, TILE_HEIGHT / 2.0)
                            .translation(translation_vector_from_index(index, vector![self.width, self.height]))
                            .collision_groups(InteractionGroups { memberships: COLLISION_GROUP_WALL, filter: COLLISION_GROUP_PLAYER })
                            .build()
                    }))
                }
                if tile_data == EMPTY {
                    return None
                }
                todo!()
            }).filter(Option::is_some).map(Option::unwrap).collect();
    }
}

pub struct Map {
    pub colliders: Vec<MapTile>,
    pub entities: Vec<MapComponent>
}

impl RawMap {
    pub fn into(&self) -> Option<Map> {
        let entities = match self.layers.iter().find(|&layer| match layer {
                Layer::ObjectLayer(object_layer) => 
                    match object_layer.name {
                        ObjectLayerName::Entities => true,
                    }
                Layer::TileLayer(_) => false
            }) {
            Some(found_layer) => match found_layer {
                Layer::ObjectLayer(object_layer) => Some(object_layer.into()),
                Layer::TileLayer(_) => None
            },
            None => None,
        };
        let colliders = match self.layers.iter().find(|&layer| match layer {
                Layer::ObjectLayer(_) => false,
                Layer::TileLayer(tile_layer) => 
                    match tile_layer.name {
                        TileLayerName::Colliders => true
                    } 
            }) {
            Some(found_layer) => match found_layer {
                Layer::ObjectLayer(_) => None,
                Layer::TileLayer(tile_layer) => Some(tile_layer.into())
            },
            None => None,
        };
        return match entities {
            None => None,
            Some(entities) =>
                match colliders {
                    None => None,
                    Some(colliders) =>
                        Some(Map {
                            colliders,
                            entities
                        })
                }
        }
    }
}

pub fn load(file_path: &str) -> Option<Map> {
    let map_file_raw = fs::read_to_string(file_path).unwrap();

    return (&deser_map(&map_file_raw)).into();
}