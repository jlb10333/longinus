use std::collections::HashSet;
use std::rc::Rc;

use rapier2d::na::ArrayStorage;
use rapier2d::prelude::*;
use rapier2d::{na::Vector2, parry::utils::hashmap::HashMap};

use crate::combat::Direction;
use crate::f::Monad;
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
  InventoryPickModule,
  InventoryPickSlot(Option<WeaponModuleKind>, InventoryUpdateData),
  InventoryConfirmEdit(InventoryUpdateData),
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
pub struct MenuSystem {
  pub active_menus: Vec<Menu>,
  pub inventory_update: Option<InventoryUpdateData>,
}

impl System for MenuSystem {
  fn start(_: crate::system::Context) -> std::rc::Rc<dyn System>
  where
    Self: Sized,
  {
    return Rc::new(Self {
      active_menus: vec![],
      inventory_update: None,
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let controls_system = ctx.get::<ControlsSystem>().unwrap();

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

    if self.active_menus.iter().count() > 0 {
      let NextMenuUpdate {
        menus: next_menus,
        inventory_update,
      } = next_menus(
        &self.active_menus[0],
        &input,
        &combat_system.unequipped_modules,
        &combat_system.equipped_modules,
      );
      return Rc::new(Self {
        active_menus: next_menus
          .iter()
          .chain(self.active_menus.clone()[1..].iter())
          .cloned()
          .collect(),
        inventory_update,
      });
    }

    Rc::new(Self {
      active_menus: match open_menu(&input) {
        Some(menu) => vec![menu],
        None => vec![],
      },
      inventory_update: None,
    })
  }
}

fn open_menu(input: &MenuInput) -> Option<Menu> {
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

struct NextMenuUpdate {
  menus: Vec<Menu>,
  inventory_update: Option<InventoryUpdateData>,
}

fn next_menus(
  current_menu: &Menu,
  input: &MenuInput,
  unequipped_modules: &UnequippedModules,
  equipped_modules: &EquippedModules,
) -> NextMenuUpdate {
  if !(input.up || input.down || input.left || input.right || input.confirm || input.cancel) {
    return NextMenuUpdate {
      menus: vec![current_menu.clone()],
      inventory_update: None,
    };
  }

  if input.cancel {
    return NextMenuUpdate {
      menus: vec![],
      inventory_update: None,
    };
  }

  match current_menu.kind.clone() {
    MenuKind::InventoryMain => NextMenuUpdate {
      menus: inventory_main(
        current_menu.cursor_position,
        input,
        unequipped_modules,
        equipped_modules,
      ),
      inventory_update: None,
    },
    MenuKind::InventoryPickModule => NextMenuUpdate {
      menus: inventory_pick_module(
        current_menu.cursor_position,
        input,
        unequipped_modules,
        equipped_modules,
      ),
      inventory_update: None,
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
      }
    }
    MenuKind::InventoryConfirmEdit(_) => NextMenuUpdate {
      menus: vec![current_menu.clone()],
      inventory_update: None,
    },
    MenuKind::PauseMain => NextMenuUpdate {
      menus: vec![current_menu.clone()],
      inventory_update: None,
    },
  }
}

const EDIT_CURSOR: Vector2<i32> = vector![0, 0];
const CLOSE_CURSOR: Vector2<i32> = vector![1, 0];

fn inventory_main(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  unequipped_modules: &UnequippedModules,
  equipped_modules: &EquippedModules,
) -> Vec<Menu> {
  let cursor_position = handle_cursor_movement(cursor_position, 1, 0, input, None);

  if cursor_position == EDIT_CURSOR && input.confirm {
    println!("{} {} new_window", cursor_position.x, cursor_position.y);
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
    println!("{} {} close window", cursor_position.x, cursor_position.y);
    return vec![];
  }

  return vec![Menu {
    cursor_position,
    kind: MenuKind::InventoryMain,
  }];
}

const INVENTORY_WRAP_WIDTH: i32 = 7;

fn inventory_pick_module(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  unequipped_modules: &UnequippedModules,
  equipped_modules: &EquippedModules,
) -> Vec<Menu> {
  let inventory_count: i32 = unequipped_modules.len().try_into().unwrap();

  let inventory_height = inventory_count / INVENTORY_WRAP_WIDTH;

  let cursor_position = handle_cursor_movement(
    cursor_position,
    INVENTORY_WRAP_WIDTH - 1,
    inventory_height,
    input,
    None,
  );

  let leftover_width_on_last_row = inventory_count % INVENTORY_WRAP_WIDTH;

  if input.confirm
    && (cursor_position.y < inventory_height || cursor_position.x < leftover_width_on_last_row)
  {
    return vec![
      Menu {
        cursor_position,
        kind: MenuKind::InventoryPickSlot(
          Some(
            unequipped_modules
              [(cursor_position.x + (cursor_position.y * (INVENTORY_WRAP_WIDTH + 1))) as usize]
              .clone(),
          ),
          InventoryUpdateData {
            equipped_modules: equipped_modules.clone(),
            unequipped_modules: unequipped_modules.clone(),
          },
        ),
      },
      Menu {
        cursor_position,
        kind: MenuKind::InventoryPickModule,
      },
    ];
  }

  vec![Menu {
    cursor_position,
    kind: MenuKind::InventoryPickModule,
  }]
}

fn inventory_pick_slot(
  cursor_position: Vector2<i32>,
  input: &MenuInput,
  currently_holding: Option<WeaponModuleKind>,
  inventory_update: &InventoryUpdateData,
) -> (Vec<Menu>, Option<InventoryUpdateData>) {
  let cursor_position = handle_cursor_movement(
    cursor_position,
    EQUIP_SLOTS_WIDTH - 1,
    EQUIP_SLOTS_HEIGHT - 1,
    input,
    Some(
      &(0..4)
        .map(|x| {
          (
            vector![x, 0],
            [(Direction::Up, vector![0, -1])].iter().cloned().collect(),
          )
        })
        .collect(),
    ),
  );

  println!("{} {}", input.confirm, cursor_position == vector![0, -1]);

  if input.confirm && cursor_position != vector![0, -1] {
    return (
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
    );
  };

  if input.confirm {
    // Confirm change and add whatever module is currently held back into the unequipped modules
    return (
      vec![Menu {
        cursor_position,
        kind: MenuKind::InventoryMain,
      }],
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

  let up = if input.up { -1 } else { 0 };
  let down = if input.down { 1 } else { 0 };
  let right = if input.right { 1 } else { 0 };
  let left = if input.left { -1 } else { 0 };

  let new_attempted_position = cursor_position + vector![left + right, up + down];

  if cursor_position == new_attempted_position {
    return cursor_position;
  }

  return vector![
    if new_attempted_position.x < 0 {
      max_x_inclusive
    } else if new_attempted_position.x > max_x_inclusive {
      0
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
