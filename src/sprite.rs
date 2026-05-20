use std::{rc::Rc, u32};

use macroquad::{
  math::{Rect, Vec2},
  prelude::Material,
  texture::Texture2D,
};

use crate::{
  GameTextures,
  easing::Easing,
  units::{PhysicsVector, UnitConvert2},
};

#[derive(Clone)]
pub struct SpriteToDraw {
  pub texture: Rc<Texture2D>,
  pub source: Rect,
  pub offset: Option<Vec2>,
  pub material: Option<Material>,
  pub z_position: Option<f32>,
}

impl SpriteToDraw {
  pub fn default(texture: &Texture2D, source: Rect) -> Self {
    Self {
      texture: Rc::new(texture.weak_clone()),
      source,
      offset: None,
      material: None,
      z_position: None,
    }
  }

  pub fn with_material(&self, material: Option<Material>) -> Self {
    Self {
      material,
      ..self.clone()
    }
  }
}

#[derive(Clone)]
pub enum SimpleSpriteTextureKind {
  Player,
  PlasmaProjectile,
  MissileProjectile,
  ImpProjectile,
  AraneaQueenProjectile,
  Beam(i32, PhysicsVector),
  SniperProjectile,
  HealthTankPickup,
  ManaTankPickup,
  WeaponModulePickup,
  BreakableTile,
  Block(PhysicsVector),
  AngelicBlock(PhysicsVector),
  HealthPickup,
  ManaPickup,
  LaserGate,
  AraneaEgg,
  GravityParticle,
  Explosion(Easing<f32>),
  SavePoint,
  Chain(PhysicsVector),
}

pub use SimpleSpriteTextureKind::*;

pub fn get_sprites_to_draw(
  kind: &SimpleSpriteTextureKind,
  frame_count: i64,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  match kind {
    Player => player(game_textures),
    PlasmaProjectile => plasma_projectile(game_textures),
    MissileProjectile => missile_projectile(game_textures),
    Beam(index, dimension) => beam(*index, dimension, game_textures),
    ImpProjectile => imp_projectile(game_textures),
    AraneaQueenProjectile => aranea_queen_projectile(game_textures),
    SniperProjectile => sniper_projectile(game_textures),
    HealthTankPickup => health_tank_pickup(game_textures),
    ManaTankPickup => mana_tank_pickup(frame_count, game_textures),
    WeaponModulePickup => weapon_module_pickup(game_textures),
    BreakableTile => breakable_tile(game_textures),
    Block(dimensions) => block(dimensions, game_textures),
    AngelicBlock(dimensions) => angelic_block(dimensions, game_textures),
    HealthPickup => health_pickup(game_textures),
    ManaPickup => mana_pickup(game_textures),
    LaserGate => laser_gate(game_textures),
    AraneaEgg => aranea_egg(game_textures),
    GravityParticle => gravity_particle(game_textures),
    Explosion(easing) => explosion((easing.at(frame_count as f32) * 5.0) as i32, game_textures),
    SavePoint => save_point((frame_count as i32 / 15) % 5, game_textures),
    Chain(dimensions) => chain(dimensions, game_textures),
  }
}

pub fn player(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.player_texture,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn plasma_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.projectile_textures.plasma,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn missile_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.projectile_textures.missile,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
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
    None,
  )
}

pub fn imp_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.projectile_textures.imp,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn aranea_queen_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.projectile_textures.aranea_queen,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn sniper_projectile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.projectile_textures.sniper,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn health_tank_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.pickup_textures.health_tank,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
  )]
}

pub fn mana_tank_pickup(frame_count: i64, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    (frame_count / 30) as i32 % 3,
    SpriteSheetArgs {
      width: 16,
      height: 16,
      num_columns: 2,
      num_sprites: 3,
      offset: None,
      z_position: None,
    },
    &game_textures.pickup_textures.mana_tank,
  )
}

pub fn weapon_module_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.pickup_textures.weapon_module,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
  )]
}

pub fn health_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.pickup_textures.health,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn mana_pickup(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.pickup_textures.mana,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn breakable_tile(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.breakable_tile_texture,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn block(dimensions: &PhysicsVector, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(dimensions, &game_textures.block_textures.block, None, None)
}

pub fn angelic_block(
  dimensions: &PhysicsVector,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.block_textures.angelic_block,
    None,
    None,
  )
}

pub fn chain(dimensions: &PhysicsVector, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.ability_textures.chain,
    None,
    Some(-10.0),
  )
}

pub fn chain_mount_point_selection(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 6,
      num_columns: 2,
      width: 24,
      height: 24,
      offset: None,
      z_position: Some(-10.0),
    },
    &game_textures.ability_textures.chain_mount_point_selection,
  )
}

pub fn touch_sensor_activated(
  dimensions: &PhysicsVector,
  game_textures: &GameTextures,
) -> Vec<SpriteToDraw> {
  tiled_sprites_to_draw(
    dimensions,
    &game_textures.activator_textures.touch_sensor_activated,
    None,
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
      z_position: None,
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
      z_position: None,
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
      z_position: None,
    },
    &game_textures.enemy_textures.aranea,
  )
}

pub fn aranea_queen(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 5,
      num_columns: 2,
      width: 48,
      height: 48,
      offset: None,
      z_position: None,
    },
    &game_textures.enemy_textures.aranea_queen,
  )
}

pub fn aranea_egg(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.enemy_textures.aranea_egg,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 16.0,
      h: 16.0,
    },
  )]
}

pub fn sniper(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 5,
      num_columns: 2,
      width: 16,
      height: 16,
      offset: None,
      z_position: None,
    },
    &game_textures.enemy_textures.sniper,
  )
}

pub fn defender(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 11,
      num_columns: 3,
      width: 24,
      height: 24,
      offset: None,
      z_position: None,
    },
    &game_textures.enemy_textures.defender,
  )
}

pub fn explosion(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 5,
      num_columns: 2,
      width: 32,
      height: 32,
      offset: None,
      z_position: None,
    },
    &game_textures.effect_textures.explosion,
  )
}

pub fn save_point(index: i32, game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  draw_from_sprite_sheet(
    index,
    SpriteSheetArgs {
      num_sprites: 5,
      num_columns: 2,
      width: 24,
      height: 24,
      offset: None,
      z_position: None,
    },
    &game_textures.save_point_texture,
  )
}

pub fn gravity_particle(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.effect_textures.gravity_particle,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 5.0,
      h: 5.0,
    },
  )]
}

pub fn laser_gate(game_textures: &GameTextures) -> Vec<SpriteToDraw> {
  vec![SpriteToDraw::default(
    &game_textures.enemy_textures.laser_gate,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )]
}

pub fn text(input_text: &str, wrap: Option<u32>, text_texture: &Texture2D) -> Vec<SpriteToDraw> {
  let wrap = wrap.unwrap_or(u32::MAX);
  input_text
    .char_indices()
    .flat_map(|(char_index, input_char)| {
      let sprite_index = input_char as u32 - 32;

      let sprite_y = sprite_index / 16;
      let sprite_x = sprite_index % 16;
      let adjusted_sprite_y = 13 - sprite_y;
      let adjusted_sprite_index = sprite_x + adjusted_sprite_y * 16;

      draw_from_sprite_sheet(
        adjusted_sprite_index as i32,
        SpriteSheetArgs {
          width: 8,
          height: 8,
          num_columns: 16,
          num_sprites: 224,
          offset: Some(Vec2 {
            x: (char_index as u32 % wrap) as f32 * 8.0,
            y: (char_index as u32 / wrap) as f32 * 8.0,
          }),
          z_position: None,
        },
        text_texture,
      )
    })
    .collect()
}

struct SpriteSheetArgs {
  num_sprites: i32,
  num_columns: i32,
  width: i32,
  height: i32,
  offset: Option<Vec2>,
  z_position: Option<f32>,
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
    offset: args.offset,
    z_position: args.z_position,
    ..SpriteToDraw::default(
      texture,
      Rect {
        x,
        y,
        w: args.width as f32,
        h: args.height as f32,
      },
    )
  }]
}

fn tiled_sprites_to_draw(
  dimensions: &PhysicsVector,
  texture: &Texture2D,
  source_offset: Option<Vec2>,
  z_offset: Option<f32>,
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

  let construct_sprite = |source: Rect, offset: Vec2| SpriteToDraw {
    offset: Some(offset),
    z_position: z_offset,
    ..SpriteToDraw::default(texture, source)
  };

  let full_tiles = (0..num_full_tiles_x).flat_map(move |x| {
    (0..num_full_tiles_y).map(move |y| {
      construct_sprite(
        Rect {
          x: 8.0 + source_offset_x,
          y: 8.0 + source_offset_y,
          w: 8.0,
          h: 8.0,
        },
        Vec2 {
          x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
          y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
        },
      )
    })
  });

  let partial_tile_x = (map_dimensions.x as i32 % 8) as f32;
  let partial_tile_y = (map_dimensions.y as i32 % 8) as f32;

  let bottom_row = (0..num_full_tiles_x).map(move |x| {
    construct_sprite(
      Rect {
        x: 8.0 + source_offset_x,
        y: 8.0 + source_offset_y,
        w: 8.0,
        h: partial_tile_y,
      },
      Vec2 {
        x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
        y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
      },
    )
  });

  let right_column = (0..num_full_tiles_y).map(move |y| {
    construct_sprite(
      Rect {
        x: 8.0 + source_offset_x,
        y: 8.0 + source_offset_y,
        w: partial_tile_x,
        h: 8.0,
      },
      Vec2 {
        x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
        y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
      },
    )
  });

  let bottom_right_tile = construct_sprite(
    Rect {
      x: 8.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: partial_tile_x,
      h: partial_tile_y,
    },
    Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    },
  );

  let left_edge = (0..num_full_tiles_y).map(move |y| {
    construct_sprite(
      Rect {
        x: 0.0 + source_offset_x,
        y: 8.0 + source_offset_y,
        w: 8.0,
        h: 8.0,
      },
      Vec2 {
        x: -(map_dimensions.x / 2.0) - 4.0,
        y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
      },
    )
  });

  let right_edge = (0..num_full_tiles_y).map(move |y| {
    construct_sprite(
      Rect {
        x: 16.0 + source_offset_x,
        y: 8.0 + source_offset_y,
        w: 8.0,
        h: 8.0,
      },
      Vec2 {
        x: (map_dimensions.x / 2.0) + 4.0,
        y: y as f32 * 8.0 - (map_dimensions.y / 2.0) + 4.0,
      },
    )
  });

  let left_edge_remainder = construct_sprite(
    Rect {
      x: 0.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: partial_tile_y,
    },
    Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    },
  );

  let right_edge_remainder = construct_sprite(
    Rect {
      x: 16.0 + source_offset_x,
      y: 8.0 + source_offset_y,
      w: 8.0,
      h: partial_tile_y,
    },
    Vec2 {
      x: (map_dimensions.x / 2.0) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y / 2.0),
    },
  );

  let top_edge = (0..num_full_tiles_x).map(move |x| {
    construct_sprite(
      Rect {
        x: 8.0 + source_offset_x,
        y: 0.0 + source_offset_y,
        w: 8.0,
        h: 8.0,
      },
      Vec2 {
        x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
        y: -(map_dimensions.y / 2.0) - 4.0,
      },
    )
  });

  let bottom_edge = (0..num_full_tiles_x).map(move |x| {
    construct_sprite(
      Rect {
        x: 8.0 + source_offset_x,
        y: 16.0 + source_offset_y,
        w: 8.0,
        h: 8.0,
      },
      Vec2 {
        x: x as f32 * 8.0 - (map_dimensions.x / 2.0) + 4.0,
        y: (map_dimensions.y / 2.0) + 4.0,
      },
    )
  });

  let top_edge_remainder = construct_sprite(
    Rect {
      x: 8.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: partial_tile_x,
      h: 8.0,
    },
    Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: -(map_dimensions.y / 2.0) - 4.0,
    },
  );

  let bottom_edge_remainder = construct_sprite(
    Rect {
      x: 8.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: partial_tile_x,
      h: 8.0,
    },
    Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x / 2.0),
      y: (map_dimensions.y / 2.0) + 4.0,
    },
  );

  let top_left_corner = construct_sprite(
    Rect {
      x: 0.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: -(map_dimensions.y / 2.0) - 4.0,
    },
  );

  let bottom_right_corner = construct_sprite(
    Rect {
      x: 16.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x) + 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y) + 4.0,
    },
  );

  let top_right_corner = construct_sprite(
    Rect {
      x: 16.0 + source_offset_x,
      y: 0.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    Vec2 {
      x: num_full_tiles_x as f32 * 8.0 - (map_dimensions.x / 2.0) + (partial_tile_x) + 4.0,
      y: -(map_dimensions.y / 2.0) - 4.0,
    },
  );

  let bottom_left_corner = construct_sprite(
    Rect {
      x: 0.0 + source_offset_x,
      y: 16.0 + source_offset_y,
      w: 8.0,
      h: 8.0,
    },
    Vec2 {
      x: -(map_dimensions.x / 2.0) - 4.0,
      y: num_full_tiles_y as f32 * 8.0 - (map_dimensions.y / 2.0) + (partial_tile_y) + 4.0,
    },
  );

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
