//422f9ce2a3d6dbf54ef0c2c480737e6175a4bc9a0049d1a863aabffbbf0ddbf4  -
use std::sync::LazyLock;
use super::{dynamic::DynamicMenu, dynamic_load::load_dynamic_menu};
pub struct GameMenus {
  pub inventory_main: DynamicMenu,
  pub module_pickup: DynamicMenu,
  pub pause_load_save: DynamicMenu,
  pub pause_main: DynamicMenu,
  pub save_confirm: DynamicMenu,
}
fn load_game_menus() -> GameMenus {
  GameMenus {
    inventory_main: load_dynamic_menu("assets/menus/inventory_main.menu.json"),
    module_pickup: load_dynamic_menu("assets/menus/module_pickup.menu.json"),
    pause_load_save: load_dynamic_menu("assets/menus/pause_load_save.menu.json"),
    pause_main: load_dynamic_menu("assets/menus/pause_main.menu.json"),
    save_confirm: load_dynamic_menu("assets/menus/save_confirm.menu.json"),
  }
}
pub const GAME_MENUS: LazyLock<GameMenus> = LazyLock::new(load_game_menus);
