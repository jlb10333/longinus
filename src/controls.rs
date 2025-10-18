use device_query::{Keycode};
use rapier2d::{na::Vector2, prelude::*};

const INPUT_FORCE: f32 = 0.3;
const EMPTY_VECTOR: Vector2<f32> = vector![0.0, 0.0];

pub fn handle_input(keys: Vec<Keycode>) -> Vector2<f32> {    
    let component_vectors = [
        if keys.contains(&Keycode::Up) { vector![0.0, INPUT_FORCE] } else { EMPTY_VECTOR },
        if keys.contains(&Keycode::Down) { vector![0.0, -INPUT_FORCE] } else { EMPTY_VECTOR },
        if keys.contains(&Keycode::Left) { vector![-INPUT_FORCE, 0.0] } else { EMPTY_VECTOR },
        if keys.contains(&Keycode::Right) { vector![INPUT_FORCE, 0.0] } else { EMPTY_VECTOR }
    ];

    return component_vectors.iter().sum();
}
