use macroquad::prelude::*;
use rapier2d::prelude::*;
use std::time::{Duration};
use std::thread::sleep;
use device_query::{DeviceQuery, DeviceState, Keycode};

use crate::graphics_utils::draw_tile;
use crate::load_map::{COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALL};

mod controls;
mod load_map;
mod graphics_utils;

mod assets {
    pub mod map1;
}

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

const SHOW_COLLIDERS: bool = true;

#[macroquad::main("MyGame")]
async fn main() {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();


    /* Load objects from the map */
    let map_components = load_map::map_to_components(
        &assets::map1::MAP_1,
        vector![assets::map1::MAP_1_WIDTH, assets::map1::MAP_1_HEIGHT]
    );

    map_components.for_each(|map_component| {
        match map_component {
            load_map::MapComponent::Empty => {},
            load_map::MapComponent::Wall(collider) => {
                collider_set.insert(collider);
            }
            load_map::MapComponent::Player(player) => {
                let handle = rigid_body_set.insert(player.rigid_body);
                collider_set.insert_with_parent(player.collider, handle, &mut rigid_body_set);
            }
        }
    });

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 5.0])
        .build();
    let ball_collider = ColliderBuilder::ball(0.5).restitution(0.7)
        .collision_groups(InteractionGroups { memberships: COLLISION_GROUP_PLAYER, filter: COLLISION_GROUP_WALL })
        .build();
    let ball_body_handle = rigid_body_set.insert(rigid_body);
    collider_set.insert_with_parent(ball_collider.clone(), ball_body_handle, &mut rigid_body_set);

    /* Create other structures necessary for the simulation. */
    let gravity = vector![0.0, -9.81];
    let integration_parameters = IntegrationParameters::default();
    let mut physics_pipeline = PhysicsPipeline::new();
    let mut island_manager = IslandManager::new();
    let mut broad_phase = DefaultBroadPhase::new();
    let mut narrow_phase = NarrowPhase::new();
    let mut impulse_joint_set = ImpulseJointSet::new();
    let mut multibody_joint_set = MultibodyJointSet::new();
    let mut ccd_solver = CCDSolver::new();
    let physics_hooks = ();
    let event_handler = ();

    /* Run the game loop, stepping the simulation once per frame. */
    loop {
        // physics

        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            &mut island_manager,
            &mut broad_phase,
            &mut narrow_phase,
            &mut rigid_body_set,
            &mut collider_set,
            &mut impulse_joint_set,
            &mut multibody_joint_set,
            &mut ccd_solver,
            &physics_hooks,
            &event_handler,
        );

        let ball_body = &mut rigid_body_set[ball_body_handle];

        // input

        let device_state = DeviceState::new();
        let keys: Vec<Keycode> = device_state.get_keys();

        let input_force = controls::handle_input(keys);

        ball_body.apply_impulse(input_force, true);

        // graphics

        clear_background(RED);

        draw_circle(ball_body.translation().x * 50.0, screen_height() - (ball_body.translation().y * 50.0), 10.0, GREEN);

        if SHOW_COLLIDERS {
            collider_set.iter().for_each(|(_, collider)| { draw_tile(collider.clone()) });
        }

        let frame_time = get_frame_time();

        if frame_time < MIN_FRAME_TIME {
            let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms
            sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
        }

        next_frame().await;
    }
}
