use derive_more::{Add, Sub};
use macroquad::prelude::*;
use rapier2d::prelude::*;

#[derive(Add, Sub, Clone, Copy)]
pub struct ScreenVector(pub Vector<f32>);
#[derive(Add, Sub, Clone, Copy)]
pub struct PhysicsVector(pub Vector<f32>);

impl ScreenVector {
  /* Used for internal physics engine dimensions */
  pub fn into_physics(self) -> PhysicsVector {
    return PhysicsVector(self.0.scale(0.02));
  }

  /* Used for internal physics engine positions, flipping vertically */
  pub fn into_physics_pos(self) -> PhysicsVector {
    return PhysicsVector(vector![self.0.x, screen_height() - self.0.y].scale(0.02))
  }
}

impl PhysicsVector {
  /* Used for screen (pixel) dimensions */
  pub fn into_screen(self) -> ScreenVector {
    return ScreenVector(self.0.scale(50.0));
  }

  /* Used for screen (pixel) positions, flipping vertically */
  pub fn into_screen_pos(self) -> ScreenVector {
    return ScreenVector(vector![self.0.x, (screen_height() * 0.02) - self.0.y].scale(50.0))
  }
}

pub fn screen_bounds_adjusted() -> PhysicsVector {
  return ScreenVector(vector![screen_width(), screen_height()]).into_physics();
}

pub fn draw_cuboid_collider(collider: &Collider) {
  let translation = PhysicsVector(*collider.translation()).into_screen_pos();
 
  match collider.shape().as_cuboid() {
    Some(cuboid) => {
      let extents = PhysicsVector(cuboid.half_extents.scale(2.0)).into_screen();

      draw_rectangle(
        translation.0.x,
        translation.0.y,
        translation.0.x + extents.0.x, 
        translation.0.y + extents.0.y,
        ORANGE
      );
    },
    None => {},
  }()
}