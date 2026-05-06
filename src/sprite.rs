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
pub enum SimpleSpriteTextureKind {
  Player,
  PlasmaProjectile,
  ImpProjectile,
  HealthTankPickup,
  WeaponModulePickup,
  BreakableTile,
  Block(PhysicsVector),
}

pub use SimpleSpriteTextureKind::*;

pub fn get_sprites_to_draw<'a>(
  kind: &SimpleSpriteTextureKind,
  game_textures: &'a GameTextures,
) -> Vec<SpriteToDraw<'a>> {
  match kind {
    Player => player(game_textures),
    PlasmaProjectile => plasma_projectile(game_textures),
    ImpProjectile => imp_projectile(game_textures),
    HealthTankPickup => health_tank_pickup(game_textures),
    WeaponModulePickup => weapon_module_pickup(game_textures),
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

pub fn imp_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  vec![SpriteToDraw {
    texture: &game_textures.projectile_textures.imp,
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

pub fn weapon_module_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  vec![SpriteToDraw {
    texture: &game_textures.pickup_textures.weapon_module,
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

pub fn touch_sensor_activated<'a>(
  dimensions: &PhysicsVector,
  game_textures: &'a GameTextures,
) -> Vec<SpriteToDraw<'a>> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.activator_textures.touch_sensor_activated,
  )
}

pub fn touch_sensor_deactivated<'a>(
  dimensions: &PhysicsVector,
  game_textures: &'a GameTextures,
) -> Vec<SpriteToDraw<'a>> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.activator_textures.touch_sensor_deactivated,
  )
}

pub fn goblin(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 2,
      num_columns: 1,
      width: 16,
      height: 16,
      offset: None,
    },
    &game_textures.enemy_textures.goblin,
  )
}

pub fn imp(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw<'_>> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 6,
      num_columns: 2,
      width: 32,
      height: 32,
      offset: None,
    },
    &game_textures.enemy_textures.imp,
  )
}

struct SpriteSheetArgs {
  num_sprites: i32,
  num_columns: i32,
  width: i32,
  height: i32,
  offset: Option<Vec2>,
}
fn draw_from_sprite_sheet(
  index: i32,
  args: SpriteSheetArgs,
  texture: &Texture2D,
) -> Vec<SpriteToDraw<'_>> {
  let index = index % args.num_sprites;

  let target_column = index % args.num_columns;
  let target_row = index / args.num_columns;

  let x = (target_column * args.width) as f32;
  let y = (target_row * args.height) as f32;

  vec![SpriteToDraw {
    texture,
    source: Rect {
      x,
      y,
      w: args.width as f32,
      h: args.height as f32,
    },
    offset: args.offset,
  }]
}

fn tiled_sprites_to_draw<'a>(
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
      y: 16.0,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: (map_dimensions.y / 2.0) + 4.0,
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
