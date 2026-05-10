use std::rc::Rc;

use macroquad::{
  math::{Rect, Vec2},
  prelude::Material,
  texture::Texture2D,
};

use crate::{
  GameTextures,
  units::{PhysicsVector, UnitConvert2},
};

pub struct SpriteToDraw {
  pub texture: Rc<Texture2D>,
  pub source: Rect,
  pub offset: Option<Vec2>,
  pub material: Option<Material>,
}

impl<'a> SpriteToDraw {
  pub fn with_material(&self, material: Option<Material>) -> Self {
    Self {
      material,
      offset: self.offset,
      source: self.source,
      texture: Rc::clone(&self.texture),
    }
  }
}

#[derive(Clone)]
pub enum SimpleSpriteTextureKind {
  Player,
  PlasmaProjectile,
  ImpProjectile,
  Beam(i32, PhysicsVector),
  HealthTankPickup,
  WeaponModulePickup,
  BreakableTile,
  Block(PhysicsVector),
  HealthPickup,
  ManaPickup,
  LaserGate,
  AraneaEgg,
}

pub use SimpleSpriteTextureKind::*;

pub fn get_sprites_to_draw(
  kind: &SimpleSpriteTextureKind,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  match kind {
    Player => player(game_textures),
    PlasmaProjectile => plasma_projectile(game_textures),
    Beam(index, dimension) => beam(*index, dimension, game_textures),
    ImpProjectile => imp_projectile(game_textures),
    HealthTankPickup => health_tank_pickup(game_textures),
    WeaponModulePickup => weapon_module_pickup(game_textures),
    BreakableTile => breakable_tile(game_textures),
    Block(dimensions) => block(dimensions, game_textures),
    HealthPickup => health_pickup(game_textures),
    ManaPickup => mana_pickup(game_textures),
    LaserGate => laser_gate(game_textures),
    AraneaEgg => aranea_egg(game_textures),
  }
}

pub fn player(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.player_texture.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn plasma_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.projectile_textures.plasma.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn beam(
  index: i32,
  dimensions: &PhysicsVector,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  let offset = Vec2 {
    x: 0.0,
    y: index as f32 * 24.0,
  };
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.projectile_textures.beam,
    Some(offset),
  )
}

pub fn imp_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.projectile_textures.imp.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn health_tank_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.pickup_textures.health_tank.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn weapon_module_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.pickup_textures.weapon_module.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn health_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.pickup_textures.health.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn mana_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.pickup_textures.mana.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn breakable_tile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.breakable_tile_texture.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn block(dimensions: &PhysicsVector, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(dimensions, &game_textures.block_textures.block, None)
}

pub fn touch_sensor_activated(
  dimensions: &PhysicsVector,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.activator_textures.touch_sensor_activated,
    None,
  )
}

pub fn touch_sensor_deactivated(
  dimensions: &PhysicsVector,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.activator_textures.touch_sensor_deactivated,
    None,
  )
}

pub fn goblin(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
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

pub fn imp(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
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

pub fn aranea(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 7,
      num_columns: 3,
      width: 32,
      height: 32,
      offset: None,
    },
    &game_textures.enemy_textures.aranea,
  )
}

pub fn aranea_egg(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.enemy_textures.aranea_egg.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
    offset: None,
    material: None,
  }]
}

pub fn laser_gate(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw {
    texture: Rc::new(game_textures.enemy_textures.laser_gate.weak_clone()),
    source: Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
    offset: None,
    material: None,
  }]
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
) -> Vec<SpriteToDraw> {
  let index = index % args.num_sprites;

  let target_column = index % args.num_columns;
  let target_row = index / args.num_columns;

  let x = (target_column * args.width) as f32;
  let y = (target_row * args.height) as f32;

  vec![SpriteToDraw {
    texture: Rc::new(texture.weak_clone()),
    source: Rect {
      x,
      y,
      w: args.width as f32,
      h: args.height as f32,
    },
    offset: args.offset,
    material: None,
  }]
}

fn tiled_sprites_to_draw(
  dimensions: &PhysicsVector,
  texture: &Texture2D,
  source_offset: Option<Vec2>,
) -> Vec<SpriteToDraw> {
  let texture = &Rc::new(texture.weak_clone());

  let map_dimensions = dimensions.into_vec() * 8.0;

  let num_full_tiles_x = map_dimensions.x as i32 / 8;
  let num_full_tiles_y = map_dimensions.y as i32 / 8;

  let source_offset_x = source_offset
    .map(|source_offset| source_offset.x)
    .unwrap_or(0.0);
  let source_offset_y = source_offset
    .map(|source_offset| source_offset.y)
    .unwrap_or(0.0);

  let full_tiles = (0..num_full_tiles_x).flat_map(move |x| {
    (0..num_full_tiles_y).map(move |y| SpriteToDraw {
      texture: Rc::clone(texture),
      source: Rect {
        x: 8.0 + source_offset_x,
        y: 8.0 + source_offset_y,
        w: 8.0,
        h: 8.0,
      },
      offset: Some(Vec2 {
        x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
        y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
      }),
      material: None,
    })
  });

  let partial_tile_x = (map_dimensions.x as i32 % 8) as f32;
  let partial_tile_y = (map_dimensions.y as i32 % 8) as f32;

  let bottom_row = (0..num_full_tiles_x).map(move |x| SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
    material: None,
  });

  let right_column = (0..num_full_tiles_y).map(move |y| SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: partial_tile_x,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
    }),
    material: None,
  });

  let bottom_right_tile = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: partial_tile_x,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
    material: None,
  };

  let left_edge = (0..num_full_tiles_y).map(move |y| SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 0.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
    }),
    material: None,
  });

  let right_edge = (0..num_full_tiles_y).map(move |y| SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 16.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: (map_dimensions.x / 2.0) + 4.0,
      y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
    }),
    material: None,
  });

  let left_edge_remainder = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 0.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
    material: None,
  };

  let right_edge_remainder = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 16.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: partial_tile_y,
    },
    offset: Some(Vec2 {
      x: (map_dimensions.x / 2.0) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    }),
    material: None,
  };

  let top_edge = (0..num_full_tiles_x).map(move |x| SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: -(map_dimensions.y / 2.0) - 4.0,
    }),
    material: None,
  });

  let bottom_edge = (0..num_full_tiles_x).map(move |x| SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
      y: (map_dimensions.y / 2.0) + 4.0,
    }),
    material: None,
  });

  let top_edge_remainder = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: partial_tile_x,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: -(map_dimensions.y / 2.0) - 4.0,
    }),
    material: None,
  };

  let bottom_edge_remainder = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 8.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: partial_tile_x,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: (map_dimensions.y / 2.0) + 4.0,
    }),
    material: None,
  };

  let top_left_corner = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 0.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: -(map_dimensions.y / 2.0) - 4.0,
    }),
    material: None,
  };

  let bottom_right_corner = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 16.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y) + 4.0,
    }),
    material: None,
  };

  let top_right_corner = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 16.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x) + 4.0,
      y: -(map_dimensions.y / 2.0) - 4.0,
    }),
    material: None,
  };

  let bottom_left_corner = SpriteToDraw {
    texture: Rc::clone(texture),
    source: Rect {
      x: 0.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    offset: Some(Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y) + 4.0,
    }),
    material: None,
  };

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
    .chain([top_left_corner])
    .chain([bottom_right_corner])
    .chain([top_right_corner])
    .chain([bottom_left_corner])
    .collect()
}
