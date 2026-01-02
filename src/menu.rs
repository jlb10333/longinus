use std::collections::HashSet;
use std::marker::PhantomData;
use std::rc::Rc;

use rapier2d::prelude::*;
use rapier2d::{na::Vector2, parry::utils::hashmap::HashMap};

use crate::Start;
use crate::combat::Direction;
use crate::ecs::{Destroyed, EntityHandle, Terminal};
use crate::load_map::MapAbilityType;
use crate::physics::PhysicsSystem;
use crate::save::{SaveData, SaveSystem};
use crate::{
  combat::{
    CombatSystem, EQUIP_SLOTS_HEIGHT, EQUIP_SLOTS_WIDTH, EquippedModules, UnequippedModules,
    WeaponModuleKind,
  },
  controls::ControlsSystem,
  system::System,
};

#[derive(Clone)]
pub struct InventoryUpdateData {
  pub equipped_modules: EquippedModules,
  pub unequipped_modules: UnequippedModules,
}

#[derive(Clone)]
pub enum GameMenuKind {
  PauseMain,
  PauseLoadSave,
  InventoryMain,
  InventoryPickSlot(Option<WeaponModuleKind>, InventoryUpdateData),
  SaveConfirm(i32),
  ModulePickupConfirm(WeaponModuleKind),
  AbilityPickupConfirm(MapAbilityType),
  GameOver,
  TerminalShow(Rc<Terminal>),
}

#[derive(Clone)]
pub struct Menu<Kind> {
  pub kind: Kind,
  pub cursor_position: Vector2<i32>,
}

pub type GameMenu = Menu<GameMenuKind>;

#[derive(Clone)]
pub enum MainMenuKind {
  Main(bool),
  MainLoadSave,
  MainLoadSaveConfirm,
}

pub type MainMenu = Menu<MainMenuKind>;

struct MenuInput {
  pub up: bool,
  pub down: bool,
  pub left: bool,
  pub right: bool,
  pub confirm: bool,
  pub cancel: bool,
  pub inventory: bool,
  pub pause: bool,
}

#[derive(Clone, Debug)]
pub enum SaveToLoad {
  Initial,
  SaveData(String),
}

#[derive(Clone, Default)]
pub struct MenuSystem<Input> {
  pub active_menus: Vec<GameMenu>,
  pub active_main_menus: Vec<MainMenu>,
  pub inventory_update: Option<InventoryUpdateData>,
  pub save_point_confirmed_id: Option<i32>,
  pub save_to_load: Option<SaveToLoad>,
  pub quit_decision: Option<QuitDecision>,
  phantom: PhantomData<Input>,
}

impl<Input: Clone + Default + 'static> System for MenuSystem<Input> {
  type Input = Input;
  fn start(
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    if ctx.downcast::<Start>().is_some() {
      let save_system = ctx.get::<SaveSystem<_>>().unwrap();

      return Rc::new(Self {
        active_main_menus: vec![MainMenu {
          cursor_position: vector![0, 0],
          kind: MainMenuKind::Main(!save_system.available_save_data.is_empty()),
        }],
        ..Default::default()
      });
    }

    Rc::new(Self {
      active_menus: vec![],
      ..Default::default()
    })
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

    if controls_system.last_frame.is_none() {
      return Rc::new(self.clone());
    }
    let last_frame = controls_system.last_frame.clone().unwrap();

    let input = MenuInput {
      up: controls_system.menu_up && !(last_frame.menu_up),
      down: controls_system.menu_down && !(last_frame.menu_down),
      right: controls_system.menu_right && !(last_frame.menu_right),
      left: controls_system.menu_left && !(last_frame.menu_left),
      cancel: controls_system.inventory && !(last_frame.inventory)
        || (controls_system.menu_cancel && !(last_frame.menu_cancel)),
      confirm: (controls_system.firing && !(last_frame.firing))
        || (controls_system.menu_confirm && !(last_frame.menu_confirm)),
      pause: controls_system.pause && !(last_frame.pause),
      inventory: controls_system.inventory && !(last_frame.inventory),
    };

    let save_system = ctx.get::<SaveSystem<_>>().unwrap();

    if let Some(ctx) = ctx.downcast::<SaveData>() {
      let combat_system = ctx.get::<CombatSystem>().unwrap();

      if !self.active_menus.is_empty() {
        let NextMenuUpdate {
          menus: next_menus,
          inventory_update,
          save_point_confirmed_id,
          save_to_load,
          quit_decision,
        } = next_menus(
          &self.active_menus[0],
          &input,
          &combat_system.unequipped_modules,
          &combat_system.equipped_modules,
          &save_system.available_save_data,
        );
        return Rc::new(Self {
          active_menus: next_menus
            .iter()
            .chain(self.active_menus.clone()[1..].iter())
            .cloned()
            .collect(),
          inventory_update,
          save_point_confirmed_id,
          save_to_load,
          quit_decision,
          ..Default::default()
        });
      }

      let physics_system = ctx.get::<PhysicsSystem>().unwrap();

      return Rc::new(Self {
        active_menus: open_menu(&input, physics_system),
        ..Default::default()
      });
    }

    if ctx.downcast::<Start>().is_some() {
      let NextMainMenuUpdate {
        menus: next_menus,
        save_to_load,
      } = next_main_menus(
        &self.active_main_menus[0],
        &input,
        &save_system.available_save_data,
      );

      return Rc::new(Self {
        active_main_menus: next_menus
          .iter()
          .chain(self.active_main_menus.clone()[1..].iter())
          .cloned()
          .collect(),
        save_to_load,
        ..Default::default()
      });
    }

    todo!("Expected to be in either a SaveData or ProcessStart GameState");
  }
}

fn open_menu(input: &MenuInput, physics_system: Rc<PhysicsSystem>) -> Vec<GameMenu> {
  if physics_system
    .entities
    .get(&EntityHandle::RigidBody(physics_system.player_handle))
    .unwrap()
    .components
    .get::<Destroyed>()
    .is_some()
  {
    return vec![GameMenu {
      kind: GameMenuKind::GameOver,
      cursor_position: vector![0, 0],
    }];
  }

  let save_confirm = if let Some(id) = physics_system.save_point_contact
    && physics_system.save_point_contact_last_frame.is_none()
  {
    vec![GameMenu {
      kind: GameMenuKind::SaveConfirm(id),
      cursor_position: vector![0, 0],
    }]
  } else {
    vec![]
  };

  let terminal_show = if let Some(terminal) = physics_system.terminal_contact.as_ref()
    && physics_system.terminal_contact_last_frame.is_none()
  {
    vec![GameMenu {
      kind: GameMenuKind::TerminalShow(terminal.clone()),
      cursor_position: vector![0, 0],
    }]
  } else {
    vec![]
  };

  let inventory_main = if input.inventory {
    vec![GameMenu {
      kind: GameMenuKind::InventoryMain,
      cursor_position: vector![0, 0],
    }]
  } else {
    vec![]
  };

  let pause_main = if input.pause {
    vec![GameMenu {
      kind: GameMenuKind::PauseMain,
      cursor_position: vector![0, 0],
    }]
  } else {
    vec![]
  };

  let ability_pickup_confirm = physics_system
    .new_abilities
    .iter()
    .map(|new_ability| GameMenu {
      kind: GameMenuKind::AbilityPickupConfirm(*new_ability),
      cursor_position: vector![0, 0],
    });

  let module_pickup_confirm = physics_system
    .new_weapon_modules
    .iter()
    .map(|(_, new_module)| GameMenu {
      kind: GameMenuKind::ModulePickupConfirm(*new_module),
      cursor_position: vector![0, 0],
    });

  save_confirm
    .into_iter()
    .chain(terminal_show)
    .chain(inventory_main)
    .chain(pause_main)
    .chain(ability_pickup_confirm)
    .chain(module_pickup_confirm)
    .collect()
}

#[derive(Default)]
struct NextMainMenuUpdate {
  menus: Vec<MainMenu>,
  save_to_load: Option<SaveToLoad>,
}

fn next_main_menus(
  current_menu: &MainMenu,
  input: &MenuInput,
  available_saves: &[String],
) -> NextMainMenuUpdate {
  if !(input.up || input.down || input.left || input.right || input.confirm || input.cancel) {
    return NextMainMenuUpdate {
      menus: vec![current_menu.clone()],
      ..Default::default()
    };
  }

  match current_menu.kind {
    MainMenuKind::Main(should_include_continue_option) => {
      let (menus, save_to_load) = menu_main(
        current_menu.cursor_position,
        available_saves,
        input,
        should_include_continue_option,
      );
      NextMainMenuUpdate {
        menus,
        save_to_load,
      }
    }
    MainMenuKind::MainLoadSave => {
      let (menus, save_to_load) =
        menu_load_game(current_menu.cursor_position, input, available_saves);
      NextMainMenuUpdate {
        menus,
        save_to_load: save_to_load.map(SaveToLoad::SaveData),
      }
    }
    _ => todo!("Unimplemented"),
  }
}

#[derive(Default)]
struct NextMenuUpdate {
  menus: Vec<GameMenu>,
  inventory_update: Option<InventoryUpdateData>,
  save_point_confirmed_id: Option<i32>,
  save_to_load: Option<SaveToLoad>,
  quit_decision: Option<QuitDecision>,
}

fn next_menus(
  current_menu: &GameMenu,
  input: &MenuInput,
  unequipped_modules: &UnequippedModules,
  equipped_modules: &EquippedModules,
  available_saves: &Vec<String>,
) -> NextMenuUpdate {
  if !(input.up || input.down || input.left || input.right || input.confirm || input.cancel) {
    return NextMenuUpdate {
      menus: vec![current_menu.clone()],
      ..Default::default()
    };
  }

  if input.cancel {
    if let GameMenuKind::InventoryPickSlot(currently_holding, inventory_update) = &current_menu.kind
    {
      return NextMenuUpdate {
        menus: vec![],
        inventory_update: Some(InventoryUpdateData {
          equipped_modules: inventory_update.equipped_modules,
          unequipped_modules: currently_holding
            .map(|currently_holding| {
              inventory_update
                .unequipped_modules
                .iter()
                .chain([currently_holding].iter())
                .cloned()
                .collect()
            })
            .unwrap_or(inventory_update.unequipped_modules.clone()),
        }),
        ..Default::default()
      };
    }

    return NextMenuUpdate {
      menus: vec![],

      ..Default::default()
    };
  }

  match current_menu.kind.clone() {
    GameMenuKind::PauseMain => {
      let (menus, quit_decision) = pause_main(current_menu.cursor_position, input);
      NextMenuUpdate {
        menus,
        quit_decision,
        ..Default::default()
      }
    }
    GameMenuKind::PauseLoadSave => {
      let (menus, save_to_load) =
        pause_load_game(current_menu.cursor_position, input, available_saves);
      NextMenuUpdate {
        menus,
        quit_decision: save_to_load.map(QuitDecision::LoadSave),
        ..Default::default()
      }
    }
    GameMenuKind::InventoryMain => NextMenuUpdate {
      menus: inventory_main(
        current_menu.cursor_position,
        input,
        unequipped_modules,
        equipped_modules,
      ),
      ..Default::default()
    },
    GameMenuKind::InventoryPickSlot(currently_holding, inventory_update) => {
      let (menus, inventory_update) = inventory_pick_slot(
        current_menu.cursor_position,
        input,
        currently_holding,
        &inventory_update,
      );
      NextMenuUpdate {
        menus,
        inventory_update,
        ..Default::default()
      }
    }
    GameMenuKind::SaveConfirm(id) => {
      let (menus, save_point_confirmed_id) = save_confirm(current_menu.cursor_position, input, id);
      NextMenuUpdate {
        menus,
        save_point_confirmed_id,
        ..Default::default()
      }
    }
    GameMenuKind::ModulePickupConfirm(weapon_module_kind) => NextMenuUpdate {
      menus: module_pickup_confirm(
        input,
        weapon_module_kind,
        &InventoryUpdateData {
          equipped_modules: *equipped_modules,
          unequipped_modules: unequipped_modules.clone(),
        },
      ),
      ..Default::default()
    },
    GameMenuKind::AbilityPickupConfirm(ability_type) => NextMenuUpdate {
      menus: ability_pickup_confirm(input, ability_type),
      ..Default::default()
    },
    GameMenuKind::GameOver => {
      let (quit_decision, menus) = game_over(input);
      NextMenuUpdate {
        menus,
        quit_decision,
        ..Default::default()
      }
    }
    GameMenuKind::TerminalShow(terminal) => NextMenuUpdate {
      menus: terminal_show(current_menu.cursor_position, input, &terminal),
      ..Default::default()
    },
  }
}

fn menu_main(
  cursor_position: Vector2<i32>,
  available_saves: &[String],
  input: &MenuInput,
  should_include_continue_option: bool,
) -> (Vec<MainMenu>, Option<SaveToLoad>) {
  let cursor_position = handle_cursor_movement(
    cursor_position,
    0,
    0,
    if should_include_continue_option { 2 } else { 1 },
    input,
    None,
  );

  /* No change if confirm is not input */
  if !input.confirm {
    return (
      vec![MainMenu {
        cursor_position,
        kind: MainMenuKind::Main(should_include_continue_option),
      }],
      None,
    );
  }

  /* Transition to next menu */
  let continue_game = should_include_continue_option && cursor_position == vector![0, 0];
  let new_game = if should_include_continue_option {
    cursor_position == vector![0, 1]
  } else {
    cursor_position == vector![0, 0]
  };
  let load_game = if should_include_continue_option {
    cursor_position == vector![0, 2]
  } else {
    cursor_position == vector![0, 1]
  };

  if continue_game {
    let most_recent_save = available_saves
      .iter()
      .fold("", |init, elem| if *init > **elem { init } else { elem });
    println!("{}", most_recent_save);
    return (
      vec![],
      Some(SaveToLoad::SaveData(most_recent_save.to_string())),
    );
  }

  if new_game {
    return (vec![], Some(SaveToLoad::Initial));
  }

  if load_game {
    return (
      vec![
        MainMenu {
          cursor_position: vector![0, 0],
          kind: MainMenuKind::MainLoadSave,
        },
        MainMenu {
          cursor_position,
          kind: MainMenuKind::Main(should_include_continue_option),
        },
      ],
      None,
    );
  }

  todo!("Unhandled cursor positon {}", cursor_position);
}

#[derive(Clone)]
pub enum QuitDecision {
  ToMainMenu,
  ToDesktop,
  LoadSave(String),
}

fn menu_load_game(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  available_saves: &[String],
) -> (Vec<MainMenu>, Option<String>) {
  let cursor_position = handle_cursor_movement(
    cursor_position,
    0,
    0,
    available_saves.len() as i32,
    input,
    None,
  );
  /* No change if confirm is not input */
  if !input.confirm {
    return (
      vec![MainMenu {
        cursor_position,
        kind: MainMenuKind::MainLoadSave,
      }],
      None,
    );
  }

  let cancel = cursor_position == vector![0, 0];

  if cancel {
    return (vec![], None);
  }

  let save_index_to_load = (cursor_position.y - 1) as usize;

  (vec![], Some(available_saves[save_index_to_load].clone()))
}

fn pause_main(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
) -> (Vec<GameMenu>, Option<QuitDecision>) {
  let cursor_position = handle_cursor_movement(cursor_position, 0, 0, 2, input, None);

  /* No change if confirm is not input */
  if !input.confirm {
    return (
      vec![GameMenu {
        cursor_position,
        kind: GameMenuKind::PauseMain,
      }],
      None,
    );
  }

  /* Transition to next menu */
  let cancel = cursor_position == vector![0, 0];
  let load_game = cursor_position == vector![0, 1];
  let quit_to_menu = cursor_position == vector![0, 2];

  if cancel {
    return (vec![], None);
  }

  if load_game {
    return (
      vec![
        Menu {
          cursor_position: vector![0, 0],
          kind: GameMenuKind::PauseLoadSave,
        },
        Menu {
          cursor_position,
          kind: GameMenuKind::PauseMain,
        },
      ],
      None,
    );
  }

  if quit_to_menu {
    return (vec![], Some(QuitDecision::ToMainMenu));
  }

  todo!("Unhandled cursor positon {}", cursor_position);
}

fn pause_load_game(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  available_saves: &Vec<String>,
) -> (Vec<GameMenu>, Option<String>) {
  let cursor_position = handle_cursor_movement(
    cursor_position,
    0,
    0,
    available_saves.len() as i32,
    input,
    None,
  );
  /* No change if confirm is not input */
  if !input.confirm {
    return (
      vec![GameMenu {
        cursor_position,
        kind: GameMenuKind::PauseLoadSave,
      }],
      None,
    );
  }

  let cancel = cursor_position == vector![0, 0];

  if cancel {
    return (vec![], None);
  }

  let save_index_to_load = (cursor_position.y - 1) as usize;

  return (vec![], Some(available_saves[save_index_to_load].clone()));
}

const EDIT_CURSOR: Vector2<i32> = vector![0, 0];
const CLOSE_CURSOR: Vector2<i32> = vector![1, 0];

fn inventory_main(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  unequipped_modules: &UnequippedModules,
  equipped_modules: &EquippedModules,
) -> Vec<GameMenu> {
  let cursor_position = handle_cursor_movement(cursor_position, 0, 1, 0, input, None);

  if cursor_position == EDIT_CURSOR && input.confirm {
    return vec![
      GameMenu {
        cursor_position: vector![0, 0],
        kind: GameMenuKind::InventoryPickSlot(
          None,
          InventoryUpdateData {
            equipped_modules: equipped_modules.clone(),
            unequipped_modules: unequipped_modules.clone(),
          },
        ),
      },
      GameMenu {
        cursor_position,
        kind: GameMenuKind::InventoryMain,
      },
    ];
  }

  if cursor_position == CLOSE_CURSOR && input.confirm {
    return vec![];
  }

  return vec![GameMenu {
    cursor_position,
    kind: GameMenuKind::InventoryMain,
  }];
}

pub const INVENTORY_WRAP_WIDTH: i32 = 5;

fn inventory_pick_slot(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  currently_holding: Option<WeaponModuleKind>,
  inventory_update: &InventoryUpdateData,
) -> (Vec<GameMenu>, Option<InventoryUpdateData>) {
  let unequipped_modules_count: i32 = inventory_update
    .unequipped_modules
    .len()
    .try_into()
    .unwrap();

  let unequipped_modules_height = (unequipped_modules_count / INVENTORY_WRAP_WIDTH) + 1;

  let cursor_position = if cursor_position.x < EQUIP_SLOTS_WIDTH {
    handle_cursor_movement(
      cursor_position,
      0,
      EQUIP_SLOTS_WIDTH - 1,
      EQUIP_SLOTS_HEIGHT - 1,
      input,
      Some(
        &(0..EQUIP_SLOTS_WIDTH)
          .map(|x| {
            (
              vector![x, 0],
              [(Direction::Up, vector![0, -1])].iter().cloned().collect(),
            )
          })
          .chain((0..EQUIP_SLOTS_HEIGHT).map(|y| {
            (
              vector![EQUIP_SLOTS_WIDTH - 1, y],
              [(Direction::Right, vector![EQUIP_SLOTS_WIDTH, 0])]
                .iter()
                .cloned()
                .collect(),
            )
          }))
          .collect(),
      ),
    )
  } else {
    handle_cursor_movement(
      cursor_position,
      EQUIP_SLOTS_WIDTH,
      EQUIP_SLOTS_WIDTH + INVENTORY_WRAP_WIDTH,
      unequipped_modules_height,
      input,
      Some(
        &((0..unequipped_modules_height + 1).map(|y| {
          (
            vector![EQUIP_SLOTS_WIDTH, y],
            [(Direction::Left, vector![EQUIP_SLOTS_WIDTH - 1, 0])]
              .iter()
              .cloned()
              .collect(),
          )
        }))
        .collect(),
      ),
    )
  };

  if input.confirm && cursor_position != vector![0, -1] {
    return if cursor_position.x < EQUIP_SLOTS_WIDTH {
      (
        vec![GameMenu {
          cursor_position,
          kind: GameMenuKind::InventoryPickSlot(
            inventory_update.equipped_modules.data.0[cursor_position.y as usize]
              [cursor_position.x as usize]
              .clone(),
            InventoryUpdateData {
              equipped_modules: EquippedModules::from_iterator(
                inventory_update
                  .equipped_modules
                  .iter()
                  .cloned()
                  .enumerate()
                  .map(|(index, value)| {
                    if index as i32 == cursor_position.x + (cursor_position.y * EQUIP_SLOTS_WIDTH) {
                      currently_holding.clone()
                    } else {
                      value
                    }
                  }),
              ),
              unequipped_modules: inventory_update.unequipped_modules.clone(),
            },
          ),
        }],
        None,
      )
    } else {
      let accessing_index = (cursor_position.x - EQUIP_SLOTS_WIDTH
        + (cursor_position.y * INVENTORY_WRAP_WIDTH)) as usize;

      let updated_unequipped_modules =
        if accessing_index < inventory_update.unequipped_modules.len() {
          inventory_update
            .unequipped_modules
            .iter()
            .cloned()
            .enumerate()
            .flat_map(|(index, module)| {
              if index == accessing_index {
                currently_holding
                  .clone()
                  .map(|currently_holding| vec![currently_holding])
                  .unwrap_or(vec![])
              } else {
                vec![module]
              }
            })
            .collect()
        } else {
          currently_holding
            .map(|currently_holding| {
              inventory_update
                .unequipped_modules
                .iter()
                .chain([currently_holding].iter())
                .cloned()
                .collect()
            })
            .unwrap_or(inventory_update.unequipped_modules.clone())
        };

      (
        vec![GameMenu {
          cursor_position,
          kind: GameMenuKind::InventoryPickSlot(
            inventory_update
              .unequipped_modules
              .get(accessing_index)
              .cloned(),
            InventoryUpdateData {
              equipped_modules: inventory_update.equipped_modules.clone(),
              unequipped_modules: updated_unequipped_modules,
            },
          ),
        }],
        None,
      )
    };
  };

  /* Confirm change and add whatever module is currently held back into the unequipped modules */
  if input.confirm {
    return (
      vec![],
      Some(InventoryUpdateData {
        equipped_modules: inventory_update.equipped_modules.clone(),
        unequipped_modules: currently_holding
          .map(|currently_holding| {
            inventory_update
              .unequipped_modules
              .iter()
              .chain([currently_holding].iter())
              .cloned()
              .collect()
          })
          .unwrap_or(inventory_update.unequipped_modules.clone()),
      }),
    );
  }

  return (
    vec![GameMenu {
      cursor_position,
      kind: GameMenuKind::InventoryPickSlot(currently_holding, inventory_update.clone()),
    }],
    None,
  );
}

fn save_confirm(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  id: i32,
) -> (Vec<GameMenu>, Option<i32>) {
  let cursor_position = handle_cursor_movement(cursor_position, 0, 1, 0, input, None);

  if !input.confirm {
    return (
      vec![GameMenu {
        cursor_position,
        kind: GameMenuKind::SaveConfirm(id),
      }],
      None,
    );
  }

  if cursor_position == vector![0, 0] {
    return (vec![], None);
  }

  if cursor_position == vector![1, 0] {
    return (vec![], Some(id));
  }

  todo!("Unaccounted cursor position {}", cursor_position);
}

fn module_pickup_confirm(
  input: &MenuInput,
  weapon_module_kind: WeaponModuleKind,
  inventory_update: &InventoryUpdateData,
) -> Vec<GameMenu> {
  if input.confirm {
    vec![
      GameMenu {
        cursor_position: vector![0, 0],
        kind: GameMenuKind::InventoryPickSlot(None, inventory_update.clone()),
      },
      GameMenu {
        cursor_position: vector![0, 0],
        kind: GameMenuKind::InventoryMain,
      },
    ]
  } else {
    vec![GameMenu {
      cursor_position: vector![0, 0],
      kind: GameMenuKind::ModulePickupConfirm(weapon_module_kind),
    }]
  }
}

fn ability_pickup_confirm(input: &MenuInput, ability_type: MapAbilityType) -> Vec<GameMenu> {
  if input.confirm {
    vec![]
  } else {
    vec![GameMenu {
      cursor_position: vector![0, 0],
      kind: GameMenuKind::AbilityPickupConfirm(ability_type),
    }]
  }
}

fn game_over(input: &MenuInput) -> (Option<QuitDecision>, Vec<GameMenu>) {
  if input.confirm {
    (Some(QuitDecision::ToMainMenu), vec![])
  } else {
    (
      None,
      vec![GameMenu {
        cursor_position: vector![0, 0],
        kind: GameMenuKind::GameOver,
      }],
    )
  }
}

pub const TERMINAL_DISPLAY_LINES_BEFORE_SCROLL: i32 = 20;

fn terminal_show(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  terminal: &Rc<Terminal>,
) -> Vec<GameMenu> {
  if input.confirm {
    vec![]
  } else {
    let num_lines = terminal.content.split('\n').count();
    let scrollable_lines = (num_lines as i32 - TERMINAL_DISPLAY_LINES_BEFORE_SCROLL).max(0);

    let cursor_position =
      handle_cursor_movement(cursor_position, 0, 0, scrollable_lines, input, None);

    vec![GameMenu {
      cursor_position,
      kind: GameMenuKind::TerminalShow(terminal.clone()),
    }]
  }
}

fn menu_input_to_direction(input: &MenuInput) -> HashSet<Direction> {
  [
    if input.up && !input.down {
      Some(Direction::Up)
    } else if input.down && !input.up {
      Some(Direction::Down)
    } else {
      None
    },
    if input.left && !input.right {
      Some(Direction::Left)
    } else if input.right && !input.left {
      Some(Direction::Right)
    } else {
      None
    },
  ]
  .iter()
  .flatten()
  .cloned()
  .collect()
}

fn handle_cursor_movement(
  cursor_position: Vector2<i32>,
  min_x_inclusive: i32,
  max_x_inclusive: i32,
  max_y_inclusive: i32,
  input: &MenuInput,
  overrides: Option<&HashMap<Vector2<i32>, HashMap<Direction, Vector2<i32>>>>,
) -> Vector2<i32> {
  let override_movement = overrides
    .map(|overrides| overrides.get(&cursor_position))
    .flatten()
    .map(|direction_map| {
      menu_input_to_direction(input)
        .iter()
        .map(|input_direction| direction_map.get(input_direction))
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
    })
    .iter()
    .flatten()
    .cloned()
    .collect::<Vec<_>>();

  if override_movement.len() > 0 {
    return override_movement[0];
  }

  /* Debug - warn console if multiple conflicting cursor overrides are found */
  if override_movement.len() > 1 {
    println!(
      "Conflicting cursor movement overrides! Found {} for cursor position {} {}",
      override_movement.len(),
      cursor_position.x,
      cursor_position.y
    );
  }

  let up = if input.up { -1 } else { 0 };
  let down = if input.down { 1 } else { 0 };
  let right = if input.right { 1 } else { 0 };
  let left = if input.left { -1 } else { 0 };

  let new_attempted_position = cursor_position + vector![left + right, up + down];

  if cursor_position == new_attempted_position {
    return cursor_position;
  }

  return vector![
    if new_attempted_position.x < min_x_inclusive {
      max_x_inclusive
    } else if new_attempted_position.x > max_x_inclusive {
      min_x_inclusive
    } else {
      new_attempted_position.x
    },
    if new_attempted_position.y < 0 {
      max_y_inclusive
    } else if new_attempted_position.y > max_y_inclusive {
      0
    } else {
      new_attempted_position.y
    },
  ];
}
