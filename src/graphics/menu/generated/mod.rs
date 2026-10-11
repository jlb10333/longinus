//6ad90d6b1bb9fb548f95ac8b99e90cb5e7c6a4364574df09c60c84f45568d9f2  -
use std::sync::LazyLock;
use super::{dynamic::DynamicMenu, dynamic_load::load_dynamic_menu};
pub struct GameMenus {
  pub inventory_main: DynamicMenu,
}
fn load_game_menus() -> GameMenus {
  GameMenus {
    inventory_main: load_dynamic_menu(assets/menus/inventory_main.menu.json),
  }
}
pub const GAME_MENUS: LazyLock<GameMenus> = LazyLock::new(load_game_menus);
