//c4165343ffb5c021f2cd95fd7c6c2ce8678dc746b4faa538f95d9278c7df9478  -
use std::sync::LazyLock;
use super::{dynamic::DynamicMenu, dynamic_load::load_dynamic_menu};
pub struct GameMenus {
  pub inventory_main: DynamicMenu,
  pub pause_main: DynamicMenu,
  pub save_confirm: DynamicMenu,
}
fn load_game_menus() -> GameMenus {
  GameMenus {
    inventory_main: load_dynamic_menu("assets/menus/inventory_main.menu.json"),
    pause_main: load_dynamic_menu("assets/menus/pause_main.menu.json"),
    save_confirm: load_dynamic_menu("assets/menus/save_confirm.menu.json"),
  }
}
pub const GAME_MENUS: LazyLock<GameMenus> = LazyLock::new(load_game_menus);
