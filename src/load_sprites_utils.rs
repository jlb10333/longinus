use macroquad::prelude::*;

pub async fn load_texture_with_filter(path: &'static str) -> Texture2D {
  let texture = load_texture(path).await.unwrap();
  texture.set_filter(FilterMode::Nearest);
  texture
}
