use macroquad::prelude::*;
use rapier2d::prelude::*;
use std::time::{Duration};
use std::thread::sleep;
use device_query::{DeviceQuery, DeviceState, Keycode};

mod controls;

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

#[macroquad::main("MyGame")]
async fn main() {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create the ground. */
    let ground_collider = ColliderBuilder::cuboid(100.0, 0.1).build();
    collider_set.insert(ground_collider.clone());

    /* Create the bouncing ball. */
    let rigid_body = RigidBodyBuilder::dynamic()
        .translation(vector![0.0, 2.0])
        .build();
    let ball_collider = ColliderBuilder::ball(0.5).restitution(0.7).build();
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

        println!["{}, {}", ball_body.translation().x, ball_body.translation().y];

        // graphics

        clear_background(RED);

        draw_line(ground_collider.translation().x * 50.0, screen_height() - (ground_collider.translation().y * 50.0), (ground_collider.translation().x + 2.0) * 50.0, screen_height() - (ground_collider.translation().y * 50.0), 15.0, BLUE);
        draw_circle(ball_body.translation().x * 50.0, screen_height() - (ball_body.translation().y * 50.0), 10.0, GREEN);

        let frame_time = get_frame_time();

        if frame_time < MIN_FRAME_TIME {
            let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms
            sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
        }

        next_frame().await;
    }
}
