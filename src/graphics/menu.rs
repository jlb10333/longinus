use std::ops::Deref;

use super::{GameMaterials, GameTextParams, consts::*, draw_game_text};
use derive_more::{Add, Div, Mul, Sub};
use macroquad::prelude::*;
use rapier2d::prelude::*;

use crate::{
  GameTextures,
  combat::{Direction, EQUIP_SLOTS_WIDTH, WeaponModule, WeaponModuleKind, weapon_module_from_kind},
  graphics::{GameColor, draw_sprites},
  menu::{GameMenu, GameMenuKind, INVENTORY_WRAP_WIDTH},
  sprite::tiled_sprites_to_draw,
  units::{PhysicsScalar, PhysicsVector, UnitConvert, UnitConvert2},
};

pub fn draw_menu(
  menu: &GameMenu,
  available_sava_data: &[String],
  game_textures: &GameTextures,
  game_materials: &GameMaterials,
) {
  let draw_menu_box = draw_menu_box_g(game_textures);
  let draw_game_text = |text, dest, color| {
    draw_game_text(
      text,
      &game_textures.ui_textures.text,
      dest,
      GameTextParams {
        color,
        ..Default::default()
      },
      game_materials,
    )
  };

  match menu.kind {
    GameMenuKind::PauseMain => {
      draw_menu_box(TileRect {
        x: SCREEN_WIDTH_TILES / 2,
        y: SCREEN_HEIGHT_TILES / 2,
        w: SCREEN_WIDTH_TILES - Tiles(6),
        h: SCREEN_HEIGHT_TILES - Tiles(6),
      });
      draw_game_text(
        if menu.cursor_position == vector![0, 0] {
          "-resume-"
        } else {
          "resume"
        },
        Vec2 {
          x: Tiles(4).to_screen(),
          y: Tiles(8).to_screen(),
        },
        GameColor::Color1,
      );
      draw_game_text(
        if menu.cursor_position == vector![0, 1] {
          "-load game-"
        } else {
          "load game"
        },
        Vec2 {
          x: Tiles(4).to_screen(),
          y: Tiles(11).to_screen(),
        },
        GameColor::Color1,
      );
      draw_game_text(
        if menu.cursor_position == vector![0, 2] {
          "-quit to menu-"
        } else {
          "quit to menu"
        },
        Vec2 {
          x: Tiles(4).to_screen(),
          y: Tiles(14).to_screen(),
        },
        GameColor::Color1,
      );
    }
    GameMenuKind::PauseLoadSave => {
      draw_menu_box(TileRect {
        x: (SCREEN_WIDTH_TILES / 2) + Tiles(4),
        y: (SCREEN_HEIGHT_TILES / 2) + Tiles(2),
        w: SCREEN_WIDTH_TILES / 2,
        h: SCREEN_HEIGHT_TILES / 2,
      });
      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-cancel"
        } else {
          "cancel"
        },
        VIRTUAL_SCREEN_WIDTH * 0.5,
        VIRTUAL_SCREEN_HEIGHT * 0.5,
        40.0,
        COLOR_1,
      );
      available_sava_data
        .iter()
        .enumerate()
        .for_each(|(index, save)| {
          draw_text(
            &format!(
              "{}{}",
              if menu.cursor_position.y - 1 == index as i32 {
                "-"
              } else {
                ""
              },
              save
            ),
            VIRTUAL_SCREEN_WIDTH * 0.5,
            VIRTUAL_SCREEN_HEIGHT * (0.55 + (index as f32 * 0.05)),
            40.0,
            COLOR_1,
          );
        });
    }
    _ => draw_menu_deprecated(menu, available_sava_data),
  }
}

pub fn draw_menu_deprecated(menu: &GameMenu, available_sava_data: &[String]) {
  match &menu.kind {
    /* MARK: Pause Main */
    crate::menu::GameMenuKind::PauseMain => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.1,
        VIRTUAL_SCREEN_HEIGHT * 0.1,
        VIRTUAL_SCREEN_WIDTH * 0.8,
        VIRTUAL_SCREEN_HEIGHT * 0.8,
        COLOR_3,
      );

      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-resume-"
        } else {
          "resume"
        },
        VIRTUAL_SCREEN_WIDTH * 0.2,
        VIRTUAL_SCREEN_HEIGHT * 0.6,
        40.0,
        COLOR_1,
      );
      draw_text(
        if menu.cursor_position == vector![0, 1] {
          "-load game-"
        } else {
          "load game"
        },
        VIRTUAL_SCREEN_WIDTH * 0.2,
        VIRTUAL_SCREEN_HEIGHT * 0.65,
        40.0,
        COLOR_1,
      );
      draw_text(
        if menu.cursor_position == vector![0, 2] {
          "-quit to menu-"
        } else {
          "quit to menu"
        },
        VIRTUAL_SCREEN_WIDTH * 0.2,
        VIRTUAL_SCREEN_HEIGHT * 0.7,
        40.0,
        COLOR_1,
      );
    }
    /* MARK: Pause Load Save */
    crate::menu::GameMenuKind::PauseLoadSave => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.45,
        VIRTUAL_SCREEN_HEIGHT * 0.45,
        VIRTUAL_SCREEN_WIDTH * 0.5,
        VIRTUAL_SCREEN_HEIGHT * 0.5,
        COLOR_2,
      );
      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-cancel"
        } else {
          "cancel"
        },
        VIRTUAL_SCREEN_WIDTH * 0.5,
        VIRTUAL_SCREEN_HEIGHT * 0.5,
        40.0,
        COLOR_1,
      );
      available_sava_data
        .iter()
        .enumerate()
        .for_each(|(index, save)| {
          draw_text(
            &format!(
              "{}{}",
              if menu.cursor_position.y - 1 == index as i32 {
                "-"
              } else {
                ""
              },
              save
            ),
            VIRTUAL_SCREEN_WIDTH * 0.5,
            VIRTUAL_SCREEN_HEIGHT * (0.55 + (index as f32 * 0.05)),
            40.0,
            COLOR_1,
          );
        });
    }
    /* MARK: Inventory Main */
    crate::menu::GameMenuKind::InventoryMain => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.1,
        VIRTUAL_SCREEN_HEIGHT * 0.1,
        VIRTUAL_SCREEN_WIDTH * 0.8,
        VIRTUAL_SCREEN_HEIGHT * 0.8,
        COLOR_3,
      );

      draw_text(
        "inventory",
        VIRTUAL_SCREEN_WIDTH * 0.2,
        VIRTUAL_SCREEN_HEIGHT * 0.4,
        80.0,
        COLOR_1,
      );

      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-edit-"
        } else {
          "edit"
        },
        VIRTUAL_SCREEN_WIDTH * 0.2,
        VIRTUAL_SCREEN_HEIGHT * 0.6,
        40.0,
        COLOR_1,
      );
      draw_text(
        if menu.cursor_position == vector![1, 0] {
          "-close-"
        } else {
          "close"
        },
        VIRTUAL_SCREEN_WIDTH * 0.5,
        VIRTUAL_SCREEN_HEIGHT * 0.6,
        40.0,
        COLOR_1,
      );
    }
    /* MARK: Inventory pick slot */
    crate::menu::GameMenuKind::InventoryPickSlot(_, inventory_update) => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.45,
        VIRTUAL_SCREEN_HEIGHT * 0.4,
        VIRTUAL_SCREEN_WIDTH * 0.5,
        VIRTUAL_SCREEN_HEIGHT * 0.5,
        COLOR_2,
      );

      draw_text(
        if menu.cursor_position.x == 0 && menu.cursor_position.y == -1 {
          "-confirm-"
        } else {
          "confirm"
        },
        0.5 * VIRTUAL_SCREEN_WIDTH,
        0.45 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );

      (0..4).for_each(|x| {
        (0..4).for_each(|y| {
          draw_rectangle(
            (0.5 + (x as f32 * 0.05)) * VIRTUAL_SCREEN_WIDTH,
            (0.5 + (y as f32 * 0.05)) * VIRTUAL_SCREEN_HEIGHT,
            0.05 * VIRTUAL_SCREEN_WIDTH,
            0.05 * VIRTUAL_SCREEN_HEIGHT,
            COLOR_3,
          );

          draw_rectangle(
            (0.51 + (x as f32 * 0.05)) * VIRTUAL_SCREEN_WIDTH,
            (0.51 + (y as f32 * 0.05)) * VIRTUAL_SCREEN_HEIGHT,
            0.03 * VIRTUAL_SCREEN_WIDTH,
            0.03 * VIRTUAL_SCREEN_HEIGHT,
            COLOR_2,
          );
        })
      });

      if menu.cursor_position.y > -1 {
        draw_rectangle(
          (0.5 + (menu.cursor_position.x as f32 * 0.05)) * VIRTUAL_SCREEN_WIDTH,
          (0.5 + (menu.cursor_position.y as f32 * 0.05)) * VIRTUAL_SCREEN_HEIGHT,
          0.05 * VIRTUAL_SCREEN_WIDTH,
          0.05 * VIRTUAL_SCREEN_HEIGHT,
          COLOR_3,
        );

        let hovering_module = if menu.cursor_position.x < EQUIP_SLOTS_WIDTH {
          inventory_update.equipped_modules
            [(menu.cursor_position.x + (menu.cursor_position.y * EQUIP_SLOTS_WIDTH)) as usize]
        } else {
          inventory_update
            .unequipped_modules
            .get(
              (menu.cursor_position.x - EQUIP_SLOTS_WIDTH
                + (menu.cursor_position.y * INVENTORY_WRAP_WIDTH)) as usize,
            )
            .copied()
        };

        if let Some(hovering_module) = hovering_module {
          debug_module_text(hovering_module)
            .iter()
            .enumerate()
            .for_each(|(index, text)| {
              draw_text(
                text,
                VIRTUAL_SCREEN_WIDTH,
                (0.8 + (index as f32 * 0.02)) * VIRTUAL_SCREEN_HEIGHT,
                25.0,
                COLOR_1,
              );
            });
        }
      }

      inventory_update
        .equipped_modules
        .iter()
        .enumerate()
        .for_each(|(index, &equipped_module)| {
          if let Some(module_kind) = equipped_module {
            let module_x = (index as i32 % EQUIP_SLOTS_WIDTH) as f32 * 0.05;
            let module_y = (index as i32 / EQUIP_SLOTS_WIDTH) as f32 * 0.05;

            draw_text(
              debug_module_symbol(module_kind),
              (0.5113 + (module_x)) * VIRTUAL_SCREEN_WIDTH,
              (0.535 + (module_y)) * VIRTUAL_SCREEN_HEIGHT,
              30.0,
              COLOR_1,
            );

            if let WeaponModule::Modulator(_, attachment_points) =
              weapon_module_from_kind(module_kind)
            {
              attachment_points
                .iter()
                .for_each(|attachment_point| match attachment_point {
                  Direction::Up => {
                    draw_rectangle(
                      (0.52 + module_x) * VIRTUAL_SCREEN_WIDTH,
                      (0.51 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                      0.01 * VIRTUAL_SCREEN_WIDTH,
                      0.005 * VIRTUAL_SCREEN_HEIGHT,
                      COLOR_4,
                    );
                  }
                  Direction::Down => {
                    draw_rectangle(
                      (0.52 + module_x) * VIRTUAL_SCREEN_WIDTH,
                      (0.535 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                      0.01 * VIRTUAL_SCREEN_WIDTH,
                      0.005 * VIRTUAL_SCREEN_HEIGHT,
                      COLOR_4,
                    );
                  }
                  Direction::Left => {
                    draw_rectangle(
                      (0.51 + module_x) * VIRTUAL_SCREEN_WIDTH,
                      (0.52 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                      0.005 * VIRTUAL_SCREEN_WIDTH,
                      0.01 * VIRTUAL_SCREEN_HEIGHT,
                      COLOR_4,
                    );
                  }
                  Direction::Right => {
                    draw_rectangle(
                      (0.535 + module_x) * VIRTUAL_SCREEN_WIDTH,
                      (0.52 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                      0.005 * VIRTUAL_SCREEN_WIDTH,
                      0.01 * VIRTUAL_SCREEN_HEIGHT,
                      COLOR_4,
                    );
                  }
                });
            }
          };
        });

      inventory_update
        .unequipped_modules
        .iter()
        .enumerate()
        .for_each(|(index, &unequipped_module_kind)| {
          let module_x = (EQUIP_SLOTS_WIDTH + (index as i32 % INVENTORY_WRAP_WIDTH)) as f32 * 0.05;
          let module_y = (index as i32 / INVENTORY_WRAP_WIDTH) as f32 * 0.05;

          draw_text(
            debug_module_symbol(unequipped_module_kind),
            (0.5113 + (module_x)) * VIRTUAL_SCREEN_WIDTH,
            (0.535 + (module_y)) * VIRTUAL_SCREEN_HEIGHT,
            30.0,
            COLOR_1,
          );

          if let WeaponModule::Modulator(_, attachment_points) =
            weapon_module_from_kind(unequipped_module_kind)
          {
            attachment_points
              .iter()
              .for_each(|attachment_point| match attachment_point {
                Direction::Up => {
                  draw_rectangle(
                    (0.52 + module_x) * VIRTUAL_SCREEN_WIDTH,
                    (0.51 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                    0.01 * VIRTUAL_SCREEN_WIDTH,
                    0.005 * VIRTUAL_SCREEN_HEIGHT,
                    COLOR_4,
                  );
                }
                Direction::Down => {
                  draw_rectangle(
                    (0.52 + module_x) * VIRTUAL_SCREEN_WIDTH,
                    (0.535 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                    0.01 * VIRTUAL_SCREEN_WIDTH,
                    0.005 * VIRTUAL_SCREEN_HEIGHT,
                    COLOR_4,
                  );
                }
                Direction::Left => {
                  draw_rectangle(
                    (0.51 + module_x) * VIRTUAL_SCREEN_WIDTH,
                    (0.52 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                    0.005 * VIRTUAL_SCREEN_WIDTH,
                    0.01 * VIRTUAL_SCREEN_HEIGHT,
                    COLOR_4,
                  );
                }
                Direction::Right => {
                  draw_rectangle(
                    (0.535 + module_x) * VIRTUAL_SCREEN_WIDTH,
                    (0.52 + module_y) * VIRTUAL_SCREEN_HEIGHT,
                    0.005 * VIRTUAL_SCREEN_WIDTH,
                    0.01 * VIRTUAL_SCREEN_HEIGHT,
                    COLOR_4,
                  );
                }
              });
          }
        });
    }
    /* MARK: Save Confirm */
    crate::menu::GameMenuKind::SaveConfirm(_) => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.3,
        VIRTUAL_SCREEN_HEIGHT * 0.45,
        VIRTUAL_SCREEN_WIDTH * 0.4,
        VIRTUAL_SCREEN_HEIGHT * 0.1,
        COLOR_2,
      );

      draw_text(
        "Cancel",
        0.4 * VIRTUAL_SCREEN_WIDTH,
        0.5 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );

      draw_text(
        "Save",
        0.6 * VIRTUAL_SCREEN_WIDTH,
        0.5 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );

      draw_text(
        "-",
        (0.4 + (menu.cursor_position.x as f32 * 0.2)) * VIRTUAL_SCREEN_WIDTH,
        0.53 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::ModulePickupConfirm(weapon_module_kind) => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.3,
        VIRTUAL_SCREEN_HEIGHT * 0.4,
        VIRTUAL_SCREEN_WIDTH * 0.4,
        VIRTUAL_SCREEN_HEIGHT * 0.15,
        COLOR_2,
      );

      draw_text(
        &format!(
          "{} {} aquired",
          match weapon_module_from_kind(*weapon_module_kind) {
            WeaponModule::Generator(_) => {
              "Weapon"
            }
            WeaponModule::Modulator(_, _) => {
              "Modifier"
            }
          },
          debug_module_symbol(*weapon_module_kind)
        ),
        0.4 * VIRTUAL_SCREEN_WIDTH,
        0.45 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );

      draw_text(
        "-edit-",
        0.4 * VIRTUAL_SCREEN_WIDTH,
        0.5 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::AbilityPickupConfirm(ability) => {
      draw_rectangle(
        VIRTUAL_SCREEN_WIDTH * 0.3,
        VIRTUAL_SCREEN_HEIGHT * 0.4,
        VIRTUAL_SCREEN_WIDTH * 0.4,
        VIRTUAL_SCREEN_HEIGHT * 0.15,
        COLOR_2,
      );

      draw_text(
        &format!(
          "Ability {} aquired",
          match ability {
            crate::load_map::MapAbilityType::Boost => "BOOST",
            crate::load_map::MapAbilityType::Chain => "CHAIN",
          },
        ),
        0.4 * VIRTUAL_SCREEN_WIDTH,
        0.45 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );

      draw_text(
        "-close-",
        0.4 * VIRTUAL_SCREEN_WIDTH,
        0.5 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::GameOver => {
      draw_rectangle(
        0.0,
        0.0,
        VIRTUAL_SCREEN_WIDTH,
        VIRTUAL_SCREEN_HEIGHT,
        COLOR_4,
      );

      draw_text(
        "GAME OVER",
        0.4 * VIRTUAL_SCREEN_WIDTH,
        0.6 * VIRTUAL_SCREEN_HEIGHT,
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::TerminalShow(terminal) => {
      draw_rectangle(
        0.25 * VIRTUAL_SCREEN_WIDTH,
        0.2 * VIRTUAL_SCREEN_HEIGHT,
        0.5 * VIRTUAL_SCREEN_WIDTH,
        0.6 * VIRTUAL_SCREEN_HEIGHT,
        COLOR_4,
      );

      draw_text(
        &terminal.created_at,
        0.265 * VIRTUAL_SCREEN_WIDTH,
        0.25 * VIRTUAL_SCREEN_HEIGHT,
        20.0,
        COLOR_1,
      );

      terminal
        .content
        .split('\n')
        .enumerate()
        .for_each(|(index, line)| {
          draw_text(
            line,
            0. * VIRTUAL_SCREEN_WIDTH,
            (0.35 + (0.025 * index as f32)) * VIRTUAL_SCREEN_HEIGHT,
            25.0,
            COLOR_1,
          );
        });
    }
  }
}

fn debug_module_symbol(module_kind: WeaponModuleKind) -> &'static str {
  match module_kind {
    WeaponModuleKind::Plasma => "PLAS",
    WeaponModuleKind::Missile => "MISL",
    WeaponModuleKind::DoubleDamage75Freq => "D75F",
    WeaponModuleKind::DoubleFreq75Damage => "F75D",
    WeaponModuleKind::Front2Slot => "2FSL",
    WeaponModuleKind::FortyFiveSlot => "45SL",
    WeaponModuleKind::SideSlot => "SDSL",
    WeaponModuleKind::MirrorSlot => "RVSL",
    WeaponModuleKind::ManaCost => "M4NC",
    WeaponModuleKind::StatusDeteriorate => "DETR",
    WeaponModuleKind::StatusVulnerable => "VLNR",
    WeaponModuleKind::StatusWeakness => "W3KR",
    WeaponModuleKind::ManaFree => "M4NC",
  }
}

fn debug_module_text(module_kind: WeaponModuleKind) -> Vec<&'static str> {
  match module_kind {
    WeaponModuleKind::Plasma => vec!["weapon; shoots moderately fast with moderate damage"],
    WeaponModuleKind::Missile => {
      vec![
        "weapon; shoots slowly and accelerates after firing, with high damage",
        "and an explosion on impact",
      ]
    }
    WeaponModuleKind::DoubleDamage75Freq => {
      vec!["modifier; doubles damage but reduces frequency by 25%"]
    }
    WeaponModuleKind::DoubleFreq75Damage => {
      vec!["modifier; doubles frequency but reduces damage by 25%"]
    }
    WeaponModuleKind::Front2Slot => {
      vec!["modifier; allows weapon to fire from the front two projectile slots"]
    }
    WeaponModuleKind::FortyFiveSlot => {
      vec!["modifier; allows weapon to fire from the front diagonal projectile slots"]
    }
    WeaponModuleKind::SideSlot => {
      vec!["modifier; allows weapon to fire from the side projectile slots"]
    }
    WeaponModuleKind::MirrorSlot => {
      vec![
        "modifier; allows weapon to fire from the reverse equivalents of any",
        "front slots it currently fires from",
      ]
    }
    WeaponModuleKind::ManaCost => {
      vec!["modifier; doubles damage but incurs a mana cost for each shot"]
    }
    WeaponModuleKind::StatusDeteriorate => {
      vec![
        "modifier; applies the DETERIORATE status which causes enemies to be",
        "damaged over time",
      ]
    }
    WeaponModuleKind::StatusVulnerable => {
      vec![
        "modifier; applies the VULNERABLE status which causes enemies to",
        "receive more damage",
      ]
    }
    WeaponModuleKind::StatusWeakness => {
      vec![
        "modifier; applies the WEAKNESS status which causes enemies to deal",
        "less damage",
      ]
    }
    WeaponModuleKind::ManaFree => {
      vec!["modifier; damagefree, manafree"]
    }
  }
}

#[derive(Clone, Copy, Add, Sub, Mul, Div)]
pub struct Tiles(pub i32);

impl Deref for Tiles {
  type Target = i32;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl Tiles {
  fn to_screen(self) -> f32 {
    self.0 as f32 * 8.0 * VIRTUAL_PIXEL_FACTOR
  }
}

struct TileRect {
  pub x: Tiles,
  pub y: Tiles,
  pub w: Tiles,
  pub h: Tiles,
}

fn draw_menu_box_g(game_textures: &GameTextures) -> impl Fn(TileRect) {
  |dest| {
    let sprites_to_draw = tiled_sprites_to_draw(
      &PhysicsVector::from_vec(vector![*dest.w as f32, *dest.h as f32]),
      &game_textures.ui_textures.menu,
      None,
      None,
    );

    draw_sprites(
      &sprites_to_draw,
      vector![
        PhysicsScalar(*dest.x as f32).convert(),
        PhysicsScalar(*dest.y as f32).convert()
      ],
      0.0,
      false,
    );
  }
}
