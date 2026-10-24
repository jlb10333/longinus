use std::path::PathBuf;

use macroquad::prelude::*;

pub async fn load_texture_with_filter(path: PathBuf) -> Texture2D {
  let texture = load_texture(path.to_str().unwrap()).await.unwrap();
  texture.set_filter(FilterMode::Nearest);
  texture
}
