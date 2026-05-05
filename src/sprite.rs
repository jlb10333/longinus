use macroquad::{
  math::{Rect, Vec2},
  texture::Texture2D,
};

use crate::{
  GameTextures,
  units::{PhysicsVector, UnitConvert2},
};

pub struct SpriteToDraw<'a> {
  pub texture: &'a Texture2D,
  pub source: Rect,
  pub offset: Option<Vec2>,
}

#[derive(Clone)]
pub enum TextureKind {
  Player,
  PlasmaProjectile,
  HealthTankPickup,
  BreakableTile,
  Block(PhysicsVector),
}

pub use TextureKind::*;

pub fn get_sprites_to_draw<'a>(
  kind: &TextureKind,
  game_textures: &'a GameTextures,
) -> Vec<SpriteToDraw<'a>> {
  match kind {
    Player => player(game_textures),
    PlasmaProjectile => plasma_projectile(game_textures),
    HealthTankPickup => health_tank_pickup(game_textures),
    BreakableTile => breakable_tile(game_textures),
    Block(dimensions) => block(dimensions, game_textures),
  }
}

pub fn player(game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  vec![SpriteToDraw {
    texture: &game_textures.player_texture,
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
  }]
}

pub fn plasma_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  vec![SpriteToDraw {
    texture: &game_textures.projectile_textures.plasma,
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
  }]
}

pub fn health_tank_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  vec![SpriteToDraw {
    texture: &game_textures.pickup_textures.health_tank,
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
    offset: None,
  }]
}

pub fn breakable_tile(game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  vec![SpriteToDraw {
    texture: &game_textures.breakable_tile_texture,
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
  }]
}

pub fn block<'a>(
  dimensions: &PhysicsVector,
  game_textures: &'a GameTextures,
) -> Vec<SpriteToDraw<'a>> {
  tiled_sprites_to_draw(dimensions, &game_textures.block_textures.block)
}

pub fn tiled_sprites_to_draw<'a>(
  dimensions: &PhysicsVector,
  texture: &'a Texture2D,
) -> Vec<SpriteToDraw<'a>> {
  let map_dimensions = dimensions.into_vec() * 8.0;

  let num_full_tiles_x = map_dimensions.x as i32 / 8;
  let num_full_tiles_y = map_dimensions.y as i32 / 8;

  let full_tiles = (0..num_full_tiles_x).flat_map(move |x| {
    (0..num_full_tiles_y).map(move |y| SpriteToDraw {
      texture,
      source: Rect {
        x: 8.0,
        y: 8.0,
        w: 8.0,
        h: 8.0,
      },
      offset: Some(Vec2 {
        x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
        y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
      }),
    })
  });

  let partial_tile_x = (map_dimensions.x as i32 % 8) as f32;
  let partial_tile_y = (map_dimensions.y as i32 % 8) as f32;

  let bottom_row = (0..num_full_tiles_x).map(move |x| SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 8.0,
      w: 8.0,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
  });

  let right_column = (0..num_full_tiles_y).map(move |y| SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 8.0,
      w: partial_tile_x,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
    }),
  });

  let bottom_right_tile = SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 8.0,
      w: partial_tile_x,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
  };

  let left_edge = (0..num_full_tiles_y).map(move |y| SpriteToDraw {
    texture,
    source: Rect {
      x: 0.0,
      y: 8.0,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
    }),
  });

  let right_edge = (0..num_full_tiles_y).map(move |y| SpriteToDraw {
    texture,
    source: Rect {
      x: 16.0,
      y: 8.0,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: (map_dimensions.x / 2.0) + 4.0,
      y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
    }),
  });

  let left_edge_remainder = SpriteToDraw {
    texture,
    source: Rect {
      x: 0.0,
      y: 8.0,
      w: 8.0,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
  };

  let right_edge_remainder = SpriteToDraw {
    texture,
    source: Rect {
      x: 16.0,
      y: 8.0,
      w: 8.0,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: (map_dimensions.x / 2.0) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
  };

  let top_edge = (0..num_full_tiles_x).map(move |x| SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: -(map_dimensions.y / 2.0) - 4.0,
    }),
  });

  let bottom_edge = (0..num_full_tiles_x).map(move |x| SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: (map_dimensions.y / 2.0) - 3.0,
    }),
  });

  let top_edge_remainder = SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 0.0,
      w: partial_tile_x,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: -(map_dimensions.y / 2.0) - 4.0,
    }),
  };

  let bottom_edge_remainder = SpriteToDraw {
    texture,
    source: Rect {
      x: 8.0,
      y: 16.0,
      w: partial_tile_x,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: (map_dimensions.y / 2.0) + 4.0,
    }),
  };

  // let top_left_corner = SpriteToDraw {
  //   texture,
  //   source: Rect {
  //     x: 0.0,
  //     y: 0.0,
  //     w: 8.0,
  //     h: 8.0,
  //   },
  //   offset: Some(Vec2 {
  //     x: -(map_dimensions.x / 2.0) - 4.0,
  //     y: -(map_dimensions.y / 2.0) - 4.0,
  //   }),
  // };

  // let bottom_right_corner = SpriteToDraw {
  //   texture,
  //   source: Rect {
  //     x: 16.0,
  //     y: 16.0,
  //     w: 8.0,
  //     h: 8.0,
  //   },
  //   offset: Some(Vec2 {
  //     x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
  //     y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
  //   }),
  // };

  full_tiles
    .chain(bottom_row)
    .chain(right_column)
    .chain([bottom_right_tile])
    .chain(left_edge)
    .chain(right_edge)
    .chain([left_edge_remainder])
    .chain([right_edge_remainder])
    .chain(top_edge)
    .chain(bottom_edge)
    .chain([top_edge_remainder])
    .chain([bottom_edge_remainder])
    .collect()
}
