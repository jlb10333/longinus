use crate::units::ScreenVector;
use rapier2d::prelude::*;

const RETICLE_DISTANCE: f32 = 10.0;

pub fn reticle_pos(angle: f32) -> ScreenVector {
  let reticle_x = f32::cos(angle) * RETICLE_DISTANCE;
  let reticle_y = f32::sin(angle) * RETICLE_DISTANCE;

  return ScreenVector::new(vector![reticle_x, reticle_y]);
}
