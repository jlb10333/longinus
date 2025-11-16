use std::collections::HashSet;
use std::rc::Rc;

use rapier2d::prelude::*;
use rapier2d::{na::Vector2, parry::utils::hashmap::HashMap};

use crate::combat::Direction;
use crate::physics::PhysicsSystem;
use crate::save::{SaveData, SaveSystem};
use crate::{
  combat::{
    CombatSystem, EQUIP_SLOTS_HEIGHT, EQUIP_SLOTS_WIDTH, EquippedModules, UnequippedModules,
    WeaponModuleKind,
  },
  controls::ControlsSystem,
  system::System,
  units::UnitConvert2,
};

#[derive(Clone)]
pub struct InventoryUpdateData {
  pub equipped_modules: EquippedModules,
  pub unequipped_modules: UnequippedModules,
}

#[derive(Clone)]
pub enum MenuKind {
  PauseMain,
  InventoryMain,
  InventoryPickSlot(Option<WeaponModuleKind>, InventoryUpdateData),
  InventoryConfirmEdit(InventoryUpdateData),
  SaveConfirm(i32),
}

#[derive(Clone)]
pub struct Menu {
  pub kind: MenuKind,
  pub cursor_position: Vector2<i32>,
}

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

#[derive(Clone)]
pub enum MapToLoad {
  Initial,
  SaveData(String),
}

#[derive(Clone, Default)]
pub struct MenuSystem {
  pub active_menus: Vec<Menu>,
  pub inventory_update: Option<InventoryUpdateData>,
  pub save_point_confirmed_id: Option<i32>,
  pub map_to_load: Option<MapToLoad>,
}

impl System for MenuSystem {
  type Input = SaveData;
  fn start(
    _: &crate::system::GameState<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    return Rc::new(Self {
      active_menus: vec![],
      ..Default::default()
    });
  }

  fn run(
    &self,
    ctx: &crate::system::GameState<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

    if controls_system.last_frame.is_none() {
      return Rc::new(self.clone());
    }
    let last_frame = controls_system.last_frame.clone().unwrap();

    let input = MenuInput {
      up: controls_system.left_stick.y() > 0.0 && !(last_frame.left_stick.y() > 0.0),
      down: controls_system.left_stick.y() < 0.0 && !(last_frame.left_stick.y() < 0.0),
      right: controls_system.left_stick.x() > 0.0 && !(last_frame.left_stick.x() > 0.0),
      left: controls_system.left_stick.x() < 0.0 && !(last_frame.left_stick.x() < 0.0),
      cancel: controls_system.inventory && !(last_frame.inventory),
      confirm: controls_system.firing && !(last_frame.firing),
      pause: controls_system.pause && !(last_frame.pause),
      inventory: controls_system.inventory && !(last_frame.inventory),
    };

    if self.active_menus.iter().count() > 0 {
      println!(
        "{} {} {}",
        self.active_menus.iter().count(),
        self.active_menus[0].cursor_position.x,
        self.active_menus[0].cursor_position.y,
      );
    }

    let combat_system = ctx.get::<CombatSystem>().unwrap();
    let save_system = ctx.get::<SaveSystem<_>>().unwrap();

    if self.active_menus.iter().count() > 0 {
      let NextMenuUpdate {
        menus: next_menus,
        inventory_update,
        save_point_confirmed_id,
        map_to_load,
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
        map_to_load,
      });
    }

    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    Rc::new(Self {
      active_menus: match open_menu(&input, physics_system) {
        Some(menu) => vec![menu],
        None => vec![],
      },
      ..Default::default()
    })
  }
}

fn open_menu(input: &MenuInput, physics_system: Rc<PhysicsSystem>) -> Option<Menu> {
  if let Some(id) = physics_system.save_point_contact
    && physics_system.save_point_contact_last_frame.is_none()
  {
    return Some(Menu {
      kind: MenuKind::SaveConfirm(id),
      cursor_position: vector![0, 0],
    });
  }

  if input.inventory {
    return Some(Menu {
      kind: MenuKind::InventoryMain,
      cursor_position: vector![0, 0],
    });
  }

  if input.pause {
    return Some(Menu {
      kind: MenuKind::PauseMain,
      cursor_position: vector![0, 0],
    });
  }

  return None;
}

#[derive(Default)]
struct NextMenuUpdate {
  menus: Vec<Menu>,
  inventory_update: Option<InventoryUpdateData>,
  save_point_confirmed_id: Option<i32>,
  map_to_load: Option<MapToLoad>,
}

fn next_menus(
  current_menu: &Menu,
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
    return NextMenuUpdate {
      menus: vec![],
      ..Default::default()
    };
  }

  match current_menu.kind.clone() {
    MenuKind::PauseMain => {
      let (menus, map_to_load) = pause_main(current_menu.cursor_position, available_saves, input);
      NextMenuUpdate {
        menus,
        map_to_load,
        ..Default::default()
      }
    }
    MenuKind::InventoryMain => NextMenuUpdate {
      menus: inventory_main(
        current_menu.cursor_position,
        input,
        unequipped_modules,
        equipped_modules,
      ),
      ..Default::default()
    },
    MenuKind::InventoryPickSlot(currently_holding, inventory_update) => {
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
    MenuKind::InventoryConfirmEdit(_) => NextMenuUpdate {
      menus: vec![current_menu.clone()],
      ..Default::default()
    },
    MenuKind::SaveConfirm(id) => {
      let (menus, save_point_confirmed_id) = save_confirm(current_menu.cursor_position, input, id);
      NextMenuUpdate {
        menus,
        save_point_confirmed_id,
        ..Default::default()
      }
    }
  }
}

fn pause_main(
  cursor_position: Vector2<i32>,
  available_saves: &Vec<String>,
  input: &MenuInput,
) -> (Vec<Menu>, Option<MapToLoad>) {
  let should_include_continue_option = available_saves.len() > 0;

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
      vec![Menu {
        cursor_position,
        kind: MenuKind::PauseMain,
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
    available_saves.iter().for_each(|save| println!("{}", save));
    let most_recent_save = available_saves
      .iter()
      .fold("", |init, elem| if *init > **elem { init } else { elem });
    println!("{}", most_recent_save);
    return (
      vec![],
      Some(MapToLoad::SaveData(most_recent_save.to_string())),
    );
  }

  if new_game {
    return (vec![], Some(MapToLoad::Initial));
  }

  if load_game {
    todo!();
  }

  panic!("Unhandled cursor positon {}", cursor_position);
}

const EDIT_CURSOR: Vector2<i32> = vector![0, 0];
const CLOSE_CURSOR: Vector2<i32> = vector![1, 0];

fn inventory_main(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  unequipped_modules: &UnequippedModules,
  equipped_modules: &EquippedModules,
) -> Vec<Menu> {
  let cursor_position = handle_cursor_movement(cursor_position, 0, 1, 0, input, None);

  if cursor_position == EDIT_CURSOR && input.confirm {
    return vec![
      Menu {
        cursor_position: vector![0, 0],
        kind: MenuKind::InventoryPickSlot(
          None,
          InventoryUpdateData {
            equipped_modules: equipped_modules.clone(),
            unequipped_modules: unequipped_modules.clone(),
          },
        ),
      },
      Menu {
        cursor_position,
        kind: MenuKind::InventoryMain,
      },
    ];
  }

  if cursor_position == CLOSE_CURSOR && input.confirm {
    return vec![];
  }

  return vec![Menu {
    cursor_position,
    kind: MenuKind::InventoryMain,
  }];
}

pub const INVENTORY_WRAP_WIDTH: i32 = 7;

fn inventory_pick_slot(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  currently_holding: Option<WeaponModuleKind>,
  inventory_update: &InventoryUpdateData,
) -> (Vec<Menu>, Option<InventoryUpdateData>) {
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
        vec![Menu {
          cursor_position,
          kind: MenuKind::InventoryPickSlot(
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
        + (cursor_position.y * (INVENTORY_WRAP_WIDTH + 1))) as usize;

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
        vec![Menu {
          cursor_position,
          kind: MenuKind::InventoryPickSlot(
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
    vec![Menu {
      cursor_position,
      kind: MenuKind::InventoryPickSlot(currently_holding, inventory_update.clone()),
    }],
    None,
  );
}

fn save_confirm(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  id: i32,
) -> (Vec<Menu>, Option<i32>) {
  let cursor_position = handle_cursor_movement(cursor_position, 0, 1, 0, input, None);

  if !input.confirm {
    return (
      vec![Menu {
        cursor_position,
        kind: MenuKind::SaveConfirm(id),
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

  panic!("Unaccounted cursor position {}", cursor_position);
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
