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
}

fn save_data_path(file_name: String) -> String {
  format!("./storage/{}.json", file_name)
}

const INITIAL_SAVE_FILE_PATH: &str = "./assets/save_initial.json";

pub struct SaveSystem {
  pub loaded_save_data: Option<SaveData>,
}

impl System for SaveSystem {
  fn start(_: crate::system::Context) -> std::rc::Rc<dyn System>
  where
    Self: Sized,
  {
    let initial_data: SaveData =
      serde_json::from_str(&fs::read_to_string(INITIAL_SAVE_FILE_PATH).unwrap())
        .expect("JSON was not well-formatted");
    return Rc::new(SaveSystem {
      loaded_save_data: Some(initial_data),
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let menu_system = ctx.get::<MenuSystem>().unwrap();

    if let Some(player_spawn_id) = menu_system.save_point_confirmed_id {
      let map_system = ctx.get::<MapSystem>().unwrap();
      let combat_system = ctx.get::<CombatSystem>().unwrap();

      let save_data = SaveData {
        player_spawn_id,
        map_name: map_system.current_map_name.clone(),
        unequipped_modules: combat_system.unequipped_modules.clone(),
        equipped_modules: combat_system.equipped_modules.data.0.clone(),
      };

      let sys_time: DateTime<Utc> = time::SystemTime::now().into();

      fs::write(
        save_data_path(format!("save_{}", sys_time.format("%+"))),
        serde_json::to_string_pretty(&save_data).unwrap(),
      )
      .unwrap();
    }

    return Rc::new(SaveSystem {
      loaded_save_data: None,
    });
  }
}
