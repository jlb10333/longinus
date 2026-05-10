use std::rc::Rc;

use crate::{easing::Easing, sprite::SpriteToDraw, units::PhysicsVector};

#[derive(Clone, Copy)]
pub enum EffectKind {
  NoiseDissolve,
}

pub use EffectKind::*;

#[derive(Clone)]
pub struct Effect {
  pub kind: EffectKind,
  pub easing: Easing<f32>, // After easing.at(frame_count) >= 1.0, filter out this effect
  pub translation: PhysicsVector,
  pub rotation: f32,
  pub sprites_to_draw: Rc<Vec<SpriteToDraw>>,
}
