use std::{env::current_dir, fs, path::Path};

use itertools::Itertools;
use rapier2d::{na::Vector2, prelude::*};
use serde::Deserialize;

use crate::graphics::consts::{SCREEN_HEIGHT_TILES, SCREEN_WIDTH_TILES};

use super::{
  Tiles,
  dynamic::{DynamicMenu, DynamicMenuText},
};

#[derive(Deserialize)]
struct MenuMap {
  layers: (TileLayer, ObjectLayer),
}

#[derive(Deserialize)]
struct TileLayer {
  data: Vec<u8>,
}

#[derive(Deserialize)]
struct ObjectLayer {
  objects: Vec<Object>,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum Object {
  Text(Text),
}

#[derive(Deserialize)]
struct Text {
  x: i32,
  y: i32,
  name: String,
  properties: Option<Vec<TextProperty>>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TextProperty {
  SelectCoords(SelectCoordsProperty),
}

#[derive(Deserialize)]
struct SelectCoordsProperty {
  #[serde(rename = "name")]
  _name: SelectCoordsPropertyName,
  value: String,
}

#[derive(Deserialize)]
enum SelectCoordsPropertyName {
  SelectCoords,
}

impl MenuMap {
  pub fn into_dynamic(self) -> DynamicMenu {
    let text = self
      .layers
      .1
      .objects
      .iter()
      .filter_map(|object| {
        if let Object::Text(text) = object {
          let select_coords = text
            .properties
            .iter()
            .flatten()
            .filter_map(|property| {
              if let TextProperty::SelectCoords(select_coords) = property {
                let mut coords = select_coords.value.split(",");
                let x = coords
                  .next()
                  .expect("malformed coordinate, missing x")
                  .parse::<i32>()
                  .expect("malformed coordinate, non-parseable x");
                let y = coords
                  .next()
                  .expect("malformed coordinate, missing y")
                  .parse::<i32>()
                  .expect("malformed coordinate, non-parseable y");

                Some(vector![x, y])
              } else {
                None
              }
            })
            .collect_vec();

          Some(DynamicMenuText {
            x: Tiles(text.x / MENU_MAP_PIXEL_FACTOR),
            y: Tiles(text.y / MENU_MAP_PIXEL_FACTOR),
            select_coords,
            text: text.name.clone(),
          })
        } else {
          None
        }
      })
      .collect_vec();

    DynamicMenu {
      data: self.layers.0.data,
      text,
    }
  }
}

const MENU_MAP_PIXEL_FACTOR: i32 = 8;

/// Expects relative path from project root
pub fn load_dynamic_menu(str_path: &str) -> DynamicMenu {
  let path = Path::new(&current_dir().unwrap()).join(str_path);

  let menu_data = fs::read_to_string(path).expect("failed to read dynamic menu path");
  let menu_map: MenuMap = serde_json::from_str(&menu_data).expect("json was not well-formatted");
  menu_map.into_dynamic()
}
