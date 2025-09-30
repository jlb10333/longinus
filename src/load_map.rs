use rapier2d::{na::Vector2, prelude::*};

// 0x00 Wall
// 0x01 Player
// 0x02 Enemy
// 0x05 Empty space

pub struct Player {
    pub collider: Collider,
    pub rigid_body: RigidBody,
}

pub enum MapComponent {
    Player(Player),
    Wall(Collider),
    Empty,
}

const TILE_WIDTH: f32 = 0.02;
const TILE_HEIGHT: f32 = 0.02;

pub const COLLISION_GROUP_WALL: Group = Group::GROUP_1;
pub const COLLISION_GROUP_PLAYER: Group = Group::GROUP_2;

pub fn translation_vector_from_index(index: i32, map_dimensions: Vector2<i32>) -> Vector<f32> {
    return vector![(index % map_dimensions.x) as f32 * TILE_WIDTH, (map_dimensions.y - (index / map_dimensions.x)) as f32 * TILE_HEIGHT];
}

pub fn map_to_components(map_data: &[i32], map_dimensions: Vector2<i32>) -> impl Iterator<Item = MapComponent> {
    return map_data.iter().enumerate().map(move |(uindex, &tile_data)| {
        let index = uindex.try_into().unwrap();
        if tile_data == 0x00 {
            return MapComponent::Wall(
                ColliderBuilder::cuboid(TILE_WIDTH / 2.0, TILE_HEIGHT / 2.0)
                    .translation(translation_vector_from_index(index, map_dimensions))
                    .collision_groups(InteractionGroups { memberships: COLLISION_GROUP_WALL, filter: COLLISION_GROUP_PLAYER })
                    .build()
            )
        }
        if tile_data == 0x01 {
            return MapComponent::Player(
                Player {
                    collider: ColliderBuilder::ball(0.5).restitution(0.7)
                        .collision_groups(InteractionGroups { memberships: COLLISION_GROUP_PLAYER, filter: COLLISION_GROUP_WALL })
                        .build(),
                    rigid_body: RigidBodyBuilder::dynamic()
                        .translation(translation_vector_from_index(index, map_dimensions))
                        .build()
                }
            )
        }
        if tile_data == 0x05 {
            return MapComponent::Empty;
        }
        todo!()
    });
}