use std::{fs, rc::Rc, time};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
  combat::{
    CombatSystem, EQUIP_SLOTS_HEIGHT, EQUIP_SLOTS_WIDTH, UnequippedModules, WeaponModuleKind,
  },
  load_map::MapSystem,
  menu::MenuSystem,
  system::System,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct SaveData {
  pub player_spawn_id: i32,
  pub map_name: String,
  pub unequipped_modules: UnequippedModules,
  pub equipped_modules:
    [[Option<WeaponModuleKind>; EQUIP_SLOTS_HEIGHT as usize]; EQUIP_SLOTS_WIDTH as usize],
  pub acquired_items: Vec<(String, i32)>,
}

fn save_data_path(file_name: String) -> String {
  format!("./storage/{}.json", file_name)
}

const INITIAL_SAVE_FILE_PATH: &str = "./assets/save_initial.json";

pub struct ActiveGameState {
  pub save_data: SaveData,
  pub map_system: MapSystem,
}

pub struct SaveSystem {
  pub active_game_state: Option<ActiveGameState>,
  pub available_save_data: Vec<String>,
}

impl System for SaveSystem {
  fn start(_: crate::system::Context) -> std::rc::Rc<dyn System>
  where
    Self: Sized,
  {
    let available_save_data = fs::read_dir("./")
      .unwrap()
      .flatten()
      .map(|dir_entry| dir_entry.file_name().into_string())
      .flatten()
      .collect::<Vec<_>>();
    return Rc::new(SaveSystem {
      active_game_state: None,
      available_save_data,
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let menu_system = ctx.get::<MenuSystem>().unwrap();

    /* Save current progress */
    if let Some(player_spawn_id) = menu_system.save_point_confirmed_id
      && let Some(active_game_state) = &self.active_game_state
    {
      let combat_system = ctx.get::<CombatSystem>().unwrap();

      let save_data = SaveData {
        player_spawn_id,
        map_name: active_game_state.map_system.current_map_name.clone(),
        unequipped_modules: combat_system.unequipped_modules.clone(),
        equipped_modules: combat_system.equipped_modules.data.0.clone(),
        acquired_items: combat_system.acquired_items.clone(),
      };

      let sys_time: DateTime<Utc> = time::SystemTime::now().into();

      fs::write(
        save_data_path(format!("save_{}", sys_time.format("%+"))),
        serde_json::to_string_pretty(&save_data).unwrap(),
      )
      .unwrap();
    }

    /* Load save data */
    let loaded_game_state = menu_system.map_to_load.as_ref().map(|map_to_load| {
      let save_data = serde_json::from_str(
        &fs::read_to_string(match map_to_load {
          crate::menu::MapToLoad::Initial => INITIAL_SAVE_FILE_PATH,
          crate::menu::MapToLoad::SaveData(path) => &path,
        })
        .unwrap(),
      )
      .expect("JSON was not well-formatted");

      let map_system = MapSystem::start(ctx, &save_data);

      ActiveGameState {
        save_data,
        map_system,
      }
    });

    let active_game_state = if let None = loaded_game_state {
      self
        .active_game_state
        .as_ref()
        .map(|active_game_state| ActiveGameState {
          save_data: active_game_state.save_data.clone(),
          map_system: active_game_state.map_system.run(ctx),
        })
    } else {
      loaded_game_state
    };

    return Rc::new(SaveSystem {
      active_game_state,
      available_save_data: self
        .available_save_data
        .iter()
        .cloned()
        .chain(
          menu_system
            .save_point_confirmed_id
            .map(|_| {
              self
                .active_game_state
                .as_ref()
                .map(|active_game_state| active_game_state.map_system.current_map_name.clone())
            })
            .flatten(),
        )
        .collect(),
    });
  }
}
