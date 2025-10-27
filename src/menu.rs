use std::rc::Rc;

use rapier2d::na::Vector2;
use rapier2d::prelude::*;

use crate::{
  combat::{EquippedWeaponInventory, WeaponInventory, WeaponModule},
  controls::ControlsSystem,
  system::System,
  units::UnitConvert2,
};

#[derive(Clone)]
pub struct InventoryUpdateData {
  equipped_weapons: EquippedWeaponInventory,
  weapon_inventory: WeaponInventory,
}

#[derive(Clone, Debug)]
pub enum MenuKind {
  PauseMain,
  InventoryMain,
  InventoryPickModule,
  // InventoryPickSlot(WeaponModule, InventoryUpdateData),
  // InventoryConfirmEdit(InventoryUpdateData),
  InventoryPickSlot(i32, i32),
  InventoryConfirmEdit(i32),
}

#[derive(Clone, Debug)]
pub struct Menu {
  kind: MenuKind,
  cursor_position: Vector2<i32>,
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
        "{} {} {} {:?}",
        self.active_menus.iter().count(),
        self.active_menus[0].cursor_position.x,
        self.active_menus[0].cursor_position.y,
        self.active_menus[0].kind,
      );
    }

    if self.active_menus.iter().count() > 0 {
      return Rc::new(Self {
        active_menus: next_menus(&self.active_menus[0], &input)
          .iter()
          .chain(self.active_menus.clone()[1..].iter())
          .cloned()
          .collect(),
        inventory_update: None,
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

fn next_menus(current_menu: &Menu, input: &MenuInput) -> Vec<Menu> {
  if !(input.up || input.down || input.left || input.right || input.confirm || input.cancel) {
    return vec![current_menu.clone()];
  }

  if input.cancel {
    return vec![];
  }

  match current_menu.kind {
    MenuKind::InventoryMain => inventory_main(current_menu.cursor_position, input),
    MenuKind::InventoryPickModule => vec![current_menu.clone()],
    MenuKind::InventoryPickSlot(_, _) => vec![current_menu.clone()],
    MenuKind::InventoryConfirmEdit(_) => vec![current_menu.clone()],
    MenuKind::PauseMain => vec![current_menu.clone()],
  }
}

fn inventory_main(cursor_position: Vector2<i32>, input: &MenuInput) -> Vec<Menu> {
  let cursor_position = handle_cursor_movement(cursor_position, 1, 0, input);

  let edit_cursor = vector![0, 0];
  let close_cursor = vector![1, 0];

  if cursor_position == edit_cursor && input.confirm {
    println!("{} {} new_window", cursor_position.x, cursor_position.y);
    return vec![
      Menu {
        cursor_position: vector![0, 0],
        kind: MenuKind::InventoryPickModule,
      },
      Menu {
        cursor_position,
        kind: MenuKind::InventoryMain,
      },
    ];
  }

  if cursor_position == close_cursor && input.confirm {
    println!("{} {} close window", cursor_position.x, cursor_position.y);
    return vec![];
  }

  return vec![Menu {
    cursor_position,
    kind: MenuKind::InventoryMain,
  }];
}

fn handle_cursor_movement(
  cursor_position: Vector2<i32>,
  max_x_inclusive: i32,
  max_y_inclusive: i32,
  input: &MenuInput,
) -> Vector2<i32> {
  let up = if input.up { -1 } else { 0 };
  let down = if input.down { 1 } else { 0 };
  let right = if input.right { 1 } else { 0 };
  let left = if input.left { -1 } else { 0 };

  let new_attempted_position = cursor_position + vector![left + right, up + down];

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
