//310096d783601bcd8d399092cc48966df74555f70c3373ffc722cfb75d6dc9e1  -
use std::sync::LazyLock;
use super::{dynamic::DynamicMenu, dynamic_load::load_dynamic_menu};
pub struct GameMenus {
  pub game_overlay: DynamicMenu,
  pub inventory_main: DynamicMenu,
  pub module_pickup: DynamicMenu,
  pub pause_load_save: DynamicMenu,
  pub pause_main: DynamicMenu,
  pub save_confirm: DynamicMenu,
}
fn load_game_menus() -> GameMenus {
  GameMenus {
    game_overlay: load_dynamic_menu("assets/menus/game_overlay.menu.json"),
    inventory_main: load_dynamic_menu("assets/menus/inventory_main.menu.json"),
    module_pickup: load_dynamic_menu("assets/menus/module_pickup.menu.json"),
    pause_load_save: load_dynamic_menu("assets/menus/pause_load_save.menu.json"),
    pause_main: load_dynamic_menu("assets/menus/pause_main.menu.json"),
    save_confirm: load_dynamic_menu("assets/menus/save_confirm.menu.json"),
  }
}
pub const GAME_MENUS: LazyLock<GameMenus> = LazyLock::new(load_game_menus);
