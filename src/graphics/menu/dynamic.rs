use itertools::Itertools;
use macroquad::math::Vec2;
use rapier2d::{na::Vector2, prelude::*};

use crate::{
  SpriteTextures,
  graphics::{
    GameColor, GameMaterials, GameTextParams, consts::SCREEN_WIDTH_TILES, draw_game_text,
    draw_sprites,
  },
  sprite::{SpriteSheetArgs, draw_from_sprite_sheet},
  units::{ScreenVector, UnitConvert2},
};

use super::Tiles;

pub struct DynamicMenu {
  pub data: Vec<u8>,
  pub text: Vec<DynamicMenuText>,
  pub anchors: Vec<DynamicMenuAnchor>,
}

impl DynamicMenu {
  pub fn get_anchor(&self, slug: &str) -> &DynamicMenuAnchor {
    self
      .anchors
      .iter()
      .find(|anchor| anchor.slug == slug)
      .expect(&format!("Couldn't find anchor under slug {}", slug))
  }
}

pub struct DynamicMenuText {
  pub x: Tiles,
  pub y: Tiles,
  pub text: String,
  pub select_coords: Vec<Vector2<i32>>,
}

pub struct DynamicMenuAnchor {
  pub x: Tiles,
  pub y: Tiles,
  pub slug: String,
}

pub trait DynamicMenuItem {
  fn x(&self) -> Tiles;
  fn y(&self) -> Tiles;
  fn to_screen_vec2(&self) -> Vec2 {
    Vec2 {
      x: self.x().to_screen(),
      y: self.y().to_screen(),
    }
  }
}

impl DynamicMenuItem for DynamicMenuText {
  fn x(&self) -> Tiles {
    self.x
  }
  fn y(&self) -> Tiles {
    self.y
  }
}

impl DynamicMenuItem for DynamicMenuAnchor {
  fn x(&self) -> Tiles {
    self.x
  }
  fn y(&self) -> Tiles {
    self.y
  }
}

pub fn draw_dynamic_menu(
  menu: &DynamicMenu,
  cursor_position: Vector2<i32>,
  game_textures: &SpriteTextures,
  game_materials: &GameMaterials,
) {
  // Draw menu box
  menu.data.iter().enumerate().for_each(|(index, data)| {
    // Account for tileset offset
    if *data == 0 {
      return;
    }
    let data = data - 1;

    let sprites_to_draw = draw_from_sprite_sheet(
      data as i32,
      SpriteSheetArgs {
        width: 8,
        height: 8,
        num_columns: 3,
        num_sprites: 9,
        offset: None,
        z_position: None,
      },
      &game_textures.ui_textures.menu_texture,
    );

    let x = Tiles(index as i32 % SCREEN_WIDTH_TILES.0);
    let y = Tiles(index as i32 / SCREEN_WIDTH_TILES.0);
    draw_sprites(
      &sprites_to_draw,
      ScreenVector::from_vec(vector![x.to_screen(), y.to_screen()]),
      0.0,
      false,
    );
  });

  // Draw text
  menu.text.iter().for_each(|text| {
    let selected = text.select_coords.iter().contains(&cursor_position);

    draw_game_text(
      &text.text,
      &game_textures.ui_textures.text_font_texture,
      Vec2 {
        x: text.x.to_screen(),
        y: text.y.to_screen(),
      },
      GameTextParams {
        color: if selected {
          GameColor::Color1
        } else {
          GameColor::Color3
        },
        ..Default::default()
      },
      game_materials,
    );
  });
}
