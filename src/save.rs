use std::{env::current_dir, fs, iter::once, marker::PhantomData, path::Path, rc::Rc, time};

use chrono::{DateTime, Utc};
use rpds::HashTrieSet;
use serde::{Deserialize, Serialize};
use serde_json::Result;

use crate::{
  GameInput,
  ability::{AbilitySystem, ManaTanksCapacityInfo},
  combat::{
    CombatSystem, EQUIP_SLOTS_HEIGHT, EQUIP_SLOTS_WIDTH, UnequippedModules, WeaponModuleKind,
  },
  ecs::{Damageable, EntityHandle},
  load_map::MapSystem,
  menu::{MenuSystem, SaveToLoad},
  physics::PhysicsSystem,
  system::System,
};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct SaveData {
  pub player_spawn_id: i32,
  pub map_name: String,
  pub unequipped_modules: UnequippedModules,
  pub equipped_modules:
    [[Option<WeaponModuleKind>; EQUIP_SLOTS_HEIGHT as usize]; EQUIP_SLOTS_WIDTH as usize],
  pub exhausted_entities: HashTrieSet<(String, i32)>,
  pub player_health: f32,
  pub player_max_health: f32,
  pub acquired_boost: bool,
  pub acquired_chain: bool,
  pub visited_maps: Vec<String>,
  pub mana_tanks_capacity: ManaTanksCapacityInfo,
  pub frame_count: i64,
  #[serde(with = "chrono::serde::ts_seconds")]
  pub saved_at: DateTime<Utc>,
}

impl SaveData {
  pub fn canonical_name(&self) -> String {
    format!("save_{}", self.saved_at.timestamp())
  }
}

fn initital_save_file_path() -> String {
  Path::new(&current_dir().unwrap())
    .join("assets")
    .join("save_initial.json")
    .as_os_str()
    .to_str()
    .unwrap()
    .to_string()
}

fn save_data_path(save_filename: &str) -> String {
  Path::new(&current_dir().unwrap())
    .join("storage")
    .join(save_filename)
    .as_os_str()
    .to_str()
    .unwrap()
    .to_string()
}

fn save_dir_path() -> String {
  Path::new(&current_dir().unwrap())
    .join("storage")
    .as_os_str()
    .to_str()
    .unwrap()
    .to_string()
}

pub fn load_save(save_to_load: &SaveToLoad) -> Result<SaveData> {
  serde_json::from_str(
    &fs::read_to_string(match save_to_load {
      SaveToLoad::Initial => initital_save_file_path(),
      SaveToLoad::SaveData(path) => save_data_path(path),
    })
    .unwrap(),
  )
}

#[derive(Clone)]
pub struct SaveSystem<Input> {
  pub available_save_data: Rc<Vec<SaveData>>,
  phantom: PhantomData<Input>,
}

impl<Input: Clone + 'static> System for SaveSystem<Input> {
  type Input = Input;

  fn start(
    _: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    let mut available_save_data = fs::read_dir(save_dir_path())
      .unwrap()
      .flatten()
      .flat_map(|dir_entry| dir_entry.file_name().into_string())
      .filter_map(|path| load_save(&SaveToLoad::SaveData(path)).ok())
      .collect::<Vec<_>>();
    available_save_data.sort_by(|a, b| a.saved_at.cmp(&b.saved_at).reverse());
    Rc::new(Self {
      available_save_data: Rc::new(available_save_data),
      phantom: PhantomData,
    })
  }

  fn update(
    self: Rc<Self>,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let new_save_data = ctx.downcast::<GameInput>().and_then(|ctx| {
      let menu_system = ctx.get::<MenuSystem<_>>().unwrap();
      let map_system = ctx.get::<MapSystem>().unwrap();
      let combat_system = ctx.get::<CombatSystem>().unwrap();
      let physics_system = ctx.get::<PhysicsSystem>().unwrap();
      let ability_system = ctx.get::<AbilitySystem>().unwrap();

      /* MARK: Save current progress */
      menu_system.save_point_confirmed_id.map(|player_spawn_id| {
        let player_entity = physics_system
          .entities
          .get(&EntityHandle::RigidBody(physics_system.player_handle))
          .unwrap();

        let player_damageable = player_entity.components.get::<Damageable>().unwrap();

        let sys_time: DateTime<Utc> = time::SystemTime::now().into();

        let save_data = SaveData {
          player_spawn_id,
          map_name: map_system.current_map_name.clone(),
          unequipped_modules: combat_system.unequipped_modules.clone(),
          equipped_modules: combat_system.equipped_modules.data.0,
          exhausted_entities: combat_system.exhausted_entities.clone(),
          player_health: player_damageable.health,
          player_max_health: player_damageable.max_health,
          acquired_boost: ability_system.acquired_boost,
          acquired_chain: ability_system.acquired_chain,
          mana_tanks_capacity: ability_system.mana_tanks.capacity,
          visited_maps: map_system.map_registry.keys().cloned().collect(),
          frame_count: physics_system.frame_count,
          saved_at: sys_time,
        };

        let new_save_path = save_data.canonical_name();

        fs::write(
          save_data_path(&new_save_path),
          serde_json::to_string_pretty(&save_data).unwrap(),
        )
        .unwrap();

        save_data
      })
    });

    if let Some(new_save_data) = new_save_data {
      Rc::new(SaveSystem {
        available_save_data: Rc::new(
          once(new_save_data)
            .chain(self.available_save_data.iter().cloned())
            .collect(),
        ),
        phantom: PhantomData,
      })
    } else {
      self
    }
  }

  fn fixed_update(
    self: Rc<Self>,
    _: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    self
  }
}
