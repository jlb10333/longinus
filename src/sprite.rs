use macroquad::{math::Rect, texture::Texture2D};

use crate::GameTextures;

pub fn player<'a>(game_textures: &'a GameTextures) -> (&'a Texture2D, Rect) {
  (
    &game_textures.player_texture,
    Rect {
      x: 0.0,
      y: 0.0,
      w: 8.0,
      h: 8.0,
    },
  )
}
