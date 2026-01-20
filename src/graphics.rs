use std::{marker::PhantomData, rc::Rc, thread::sleep, time::Duration};

use macroquad::prelude::*;
use rapier2d::prelude::*;

use crate::{
  ability::AbilitySystem,
  camera::CameraSystem,
  combat::{
    CombatSystem, Direction, EQUIP_SLOTS_WIDTH, WeaponModule, WeaponModuleKind,
    distance_projection_screen, get_reticle_pos, get_slot_positions, weapon_module_from_kind,
  },
  controls::ControlsSystem,
  ecs::{Activator, Damageable, Damager, Enemy, EntityHandle, GravitySource, Id},
  graphics_utils::{draw_collider, draw_label},
  load_map::{MapSystem, physics_scalar_to_map},
  menu::{GameMenu, INVENTORY_WRAP_WIDTH, MainMenu, MenuSystem},
  physics::{PLAYER_INTERACTION_GROUPS, PhysicsSystem},
  save::SaveSystem,
  system::System,
  units::{PhysicsVector, ScreenVector, UnitConvert, UnitConvert2},
};

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

const RETICLE_SIZE: f32 = 3.0;

/* DEBUG OPTIONS */
const SHOW_COLLIDERS: bool = true;
const SHOW_SLOTS: bool = true;

/* Colors */
pub const COLOR_1: Color = Color {
  r: 214.0 / 255.0,
  g: 246.0 / 255.0,
  b: 214.0 / 255.0,
  a: 1.0,
};
pub const COLOR_2: Color = Color {
  r: 107.0 / 255.0,
  g: 165.0 / 255.0,
  b: 107.0 / 255.0,
  a: 1.0,
};
pub const COLOR_3: Color = Color {
  r: 29.0 / 255.0,
  g: 88.0 / 255.0,
  b: 73.0 / 255.0,
  a: 1.0,
};
pub const COLOR_4: Color = Color {
  r: 0.0 / 255.0,
  g: 18.0 / 255.0,
  b: 25.0 / 255.0,
  a: 1.0,
};

pub struct GraphicsSystem<Input>(PhantomData<Input>);

const MINI_MAP_TILE_WIDTH: f32 = 2.0;
const MINI_MAP_TILE_HEIGHT: f32 = 2.0;

impl<Input: Clone + Default + 'static> System for GraphicsSystem<Input> {
  type Input = Input;

  fn start(_: &crate::system::ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    Rc::new(GraphicsSystem(PhantomData))
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    /* Background */
    clear_background(COLOR_1);

    draw_fps();

    if let Some(ctx) = ctx.downcast::<_>() {
      let camera_system = ctx.get::<CameraSystem>().unwrap();
      let combat_system = ctx.get::<CombatSystem>().unwrap();
      let physics_system = ctx.get::<PhysicsSystem>().unwrap();
      let map_system = ctx.get::<MapSystem>().unwrap();
      let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

      /* Debug */
      if SHOW_COLLIDERS {
        physics_system.entities.iter().for_each(|(handle, entity)| {
          let alpha = if let Some(gravity_source) = entity.components.get::<GravitySource>()
            && let Some(target_activator_id) = gravity_source.activator_id
            && let Some((_, other_entity)) =
              physics_system.entities.iter().find(|(_, other_entity)| {
                if let Some(id) = other_entity.components.get::<Id>()
                  && id.id == target_activator_id
                {
                  true
                } else {
                  false
                }
              })
            && let Some(activator) = other_entity.components.get::<Activator>()
          {
            Some(activator.activation / 2.0)
          } else {
            None
          };

          handle
            .colliders(&physics_system.rigid_body_set)
            .iter()
            .for_each(|&&collider_handle| {
              let collider = &physics_system.collider_set[collider_handle];
              draw_collider(
                collider,
                camera_system.translation,
                None,
                if entity.components.get::<Damager>().is_some()
                  && collider.collision_groups().test(PLAYER_INTERACTION_GROUPS)
                {
                  Some(COLOR_3)
                } else {
                  None
                },
                alpha,
              );
            });

          /* Draw entity labels */
          if let EntityHandle::RigidBody(rigid_body_handle) = handle {
            draw_label(
              PhysicsVector::from_vec(
                *physics_system.rigid_body_set[*rigid_body_handle].translation(),
              ),
              camera_system.translation,
              entity.label.clone(),
              Some(BLACK),
            );
          }

          handle
            .colliders(&physics_system.rigid_body_set)
            .into_iter()
            .for_each(|&collider_handle| {
              draw_label(
                PhysicsVector::from_vec(
                  *physics_system.collider_set[collider_handle].translation(),
                ),
                camera_system.translation,
                entity.label.clone(),
                Some(BLACK),
              );
            })
        });

        physics_system
          .collider_set
          .iter()
          .filter(|(handle, _)| {
            !physics_system
              .entities
              .contains_key(&EntityHandle::Collider(*handle))
              && if let Some(parent_handle) = physics_system.collider_set[*handle].parent() {
                !physics_system
                  .entities
                  .contains_key(&EntityHandle::RigidBody(parent_handle))
              } else {
                true
              }
          })
          .for_each(|(_, collider)| {
            draw_collider(collider, camera_system.translation, None, None, None);
          });
      }

      let player_physics_pos = PhysicsVector::from_vec(
        *physics_system.rigid_body_set[physics_system.player_handle].translation(),
      );

      /* Draw scuffed map overlay */
      if controls_system.map {
        let (_, current_world_map) = map_system
          .map_registry
          .iter()
          .find(|(map_name, _)| **map_name == map_system.current_map_name)
          .unwrap();

        let player_x =
          (physics_scalar_to_map(player_physics_pos.data.0[0][0]) + current_world_map.x) / 8.0;
        let player_y = current_world_map.height
          + (current_world_map.y - physics_scalar_to_map(player_physics_pos.data.0[0][1])) / 8.0;

        let center_x = screen_width() / 2.0;
        let center_y = screen_height() / 2.0;

        draw_triangle(
          Vec2 {
            x: center_x - 2.0,
            y: center_y - 1.0,
          },
          Vec2 {
            x: center_x + 2.0,
            y: center_y - 1.0,
          },
          Vec2 {
            x: center_x,
            y: center_y + 3.0,
          },
          RED,
        );

        map_system.map_registry.iter().for_each(|(_, world_map)| {
          world_map
            .tiles
            .iter()
            .enumerate()
            .for_each(|(index, tile)| {
              if *tile == 0 {
                return;
              }

              let tile_x = index as f32 % world_map.width;
              let tile_y = (index as f32 / world_map.width).floor();

              let map_x = (tile_x + (world_map.x / 8.0) - player_x) * MINI_MAP_TILE_WIDTH;
              let map_y = (tile_y + (world_map.y / 8.0) - player_y) * MINI_MAP_TILE_HEIGHT;

              let x = map_x + center_x;
              let y = map_y + center_y;

              draw_rectangle(
                x,
                y,
                MINI_MAP_TILE_WIDTH,
                MINI_MAP_TILE_HEIGHT,
                BLACK.with_alpha(0.2),
              );
            });
        });
      }

      /* Draw reticle */
      let player_screen_pos = player_physics_pos.into_pos(camera_system.translation);

      let reticle_pos = get_reticle_pos(combat_system.reticle_angle);

      draw_circle(
        player_screen_pos.x() + reticle_pos.x(),
        player_screen_pos.y() + reticle_pos.y(),
        RETICLE_SIZE,
        COLOR_4,
      );

      /* DEBUG - Draw slots */
      if SHOW_SLOTS {
        let slot_positions = get_slot_positions(combat_system.reticle_angle);
        slot_positions.iter().for_each(|(_, slot)| {
          let slot_screen_offset = slot.offset.convert();

          let slot_screen_pos =
            ScreenVector::from_vec(player_screen_pos.into_vec() + slot_screen_offset.into_vec());

          let slot_next_screen_pos = ScreenVector::from_vec(
            slot_screen_pos.into_vec() + distance_projection_screen(slot.angle, 7.0).into_vec(),
          );

          draw_circle(slot_screen_pos.x(), slot_screen_pos.y(), 2.0, COLOR_3);
          draw_circle(
            slot_next_screen_pos.x(),
            slot_next_screen_pos.y(),
            2.0,
            COLOR_4,
          );
        });
      }

      /* Draw overlays */
      let player = physics_system
        .entities
        .get(&EntityHandle::RigidBody(physics_system.player_handle))
        .unwrap();

      let player_damageable = player.components.get::<Damageable>().unwrap();

      draw_text(
        &format!(
          "HP {}/{}",
          player_damageable.health, player_damageable.max_health
        ),
        screen_width() * 0.01,
        screen_height() * 0.8,
        40.0,
        COLOR_4,
      );

      let ability_system = ctx.get::<AbilitySystem>().unwrap();

      draw_text(
        &format!(
          "MANA {}/{}",
          ability_system.mana_tanks.rechargeable_mana_level as i32,
          ability_system
            .mana_tanks
            .capacity
            .max_rechargeable_mana_level() as i32
        ),
        screen_width() * 0.01,
        screen_height() * 0.85,
        40.0,
        COLOR_4,
      );

      draw_text(
        &format!(
          "MANA Backup {}/{}",
          ability_system.mana_tanks.non_rechargeable_mana_level as i32,
          ability_system
            .mana_tanks
            .capacity
            .max_non_rechargeable_mana_level() as i32
        ),
        screen_width() * 0.01,
        screen_height() * 0.9,
        40.0,
        COLOR_4,
      );
    }

    /* Draw the scuffed menu */
    let menu_system = ctx.get::<MenuSystem<_>>().unwrap();
    let save_system = ctx.get::<SaveSystem<_>>().unwrap();

    menu_system
      .active_main_menus
      .iter()
      .rev()
      .for_each(|menu| draw_main_menu(menu, &save_system.available_save_data));
    menu_system
      .active_menus
      .iter()
      .rev()
      .for_each(|menu| draw_menu(menu, &save_system.available_save_data));

    /* Maintain target fps */
    let frame_time = get_frame_time();

    if frame_time < MIN_FRAME_TIME {
      let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms

      sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
    }

    Rc::new(GraphicsSystem(PhantomData))
  }
}

fn draw_main_menu(menu: &MainMenu, available_sava_data: &[String]) {
  match menu.kind.clone() {
    /* MARK: Menu Main */
    crate::menu::MainMenuKind::Main(should_include_continue_option) => {
      draw_rectangle(0.0, 0.0, screen_width(), screen_height(), COLOR_4);

      draw_text(
        "LONGINUS",
        screen_width() * 0.2,
        screen_height() * 0.3,
        40.0,
        COLOR_1,
      );

      draw_text(
        &format!(
          "{}{}{}",
          if menu.cursor_position == vector![0, 0] {
            "-"
          } else {
            ""
          },
          if should_include_continue_option {
            "continue"
          } else {
            "new_game"
          },
          if menu.cursor_position == vector![0, 0] {
            "-"
          } else {
            ""
          }
        ),
        screen_width() * 0.2,
        screen_height() * 0.6,
        40.0,
        COLOR_1,
      );
      draw_text(
        &format!(
          "{}{}{}",
          if menu.cursor_position == vector![0, 1] {
            "-"
          } else {
            ""
          },
          if should_include_continue_option {
            "new_game"
          } else {
            "load_game"
          },
          if menu.cursor_position == vector![0, 1] {
            "-"
          } else {
            ""
          }
        ),
        screen_width() * 0.2,
        screen_height() * 0.7,
        40.0,
        COLOR_1,
      );
      if should_include_continue_option {
        draw_text(
          if menu.cursor_position == vector![0, 2] {
            "-load_game-"
          } else {
            "load_game"
          },
          screen_width() * 0.2,
          screen_height() * 0.8,
          40.0,
          COLOR_1,
        );
      }
    }
    crate::menu::MainMenuKind::MainLoadSave => {
      draw_rectangle(
        screen_width() * 0.45,
        screen_height() * 0.45,
        screen_width() * 0.5,
        screen_height() * 0.5,
        COLOR_2,
      );
      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-cancel-"
        } else {
          "cancel"
        },
        screen_width() * 0.5,
        screen_height() * 0.5,
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
            screen_width() * 0.5,
            screen_height() * (0.55 + (index as f32 * 0.05)),
            40.0,
            COLOR_1,
          );
        });
    }
    _ => todo!("Unimplemented"),
  }
}

fn draw_menu(menu: &GameMenu, available_sava_data: &[String]) {
  match menu.kind.clone() {
    /* MARK: Pause Main */
    crate::menu::GameMenuKind::PauseMain => {
      draw_rectangle(
        screen_width() * 0.1,
        screen_height() * 0.1,
        screen_width() * 0.8,
        screen_height() * 0.8,
        COLOR_3,
      );

      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-resume-"
        } else {
          "resume"
        },
        screen_width() * 0.2,
        screen_height() * 0.6,
        40.0,
        COLOR_1,
      );
      draw_text(
        if menu.cursor_position == vector![0, 1] {
          "-load game-"
        } else {
          "load game"
        },
        screen_width() * 0.2,
        screen_height() * 0.65,
        40.0,
        COLOR_1,
      );
      draw_text(
        if menu.cursor_position == vector![0, 2] {
          "-quit to menu-"
        } else {
          "quit to menu"
        },
        screen_width() * 0.2,
        screen_height() * 0.7,
        40.0,
        COLOR_1,
      );
    }
    /* MARK: Pause Load Save */
    crate::menu::GameMenuKind::PauseLoadSave => {
      draw_rectangle(
        screen_width() * 0.45,
        screen_height() * 0.45,
        screen_width() * 0.5,
        screen_height() * 0.5,
        COLOR_2,
      );
      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-cancel"
        } else {
          "cancel"
        },
        screen_width() * 0.5,
        screen_height() * 0.5,
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
            screen_width() * 0.5,
            screen_height() * (0.55 + (index as f32 * 0.05)),
            40.0,
            COLOR_1,
          );
        });
    }
    /* MARK: Inventory Main */
    crate::menu::GameMenuKind::InventoryMain => {
      draw_rectangle(
        screen_width() * 0.1,
        screen_height() * 0.1,
        screen_width() * 0.8,
        screen_height() * 0.8,
        COLOR_3,
      );

      draw_text(
        "inventory",
        screen_width() * 0.2,
        screen_height() * 0.4,
        80.0,
        COLOR_1,
      );

      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "-edit-"
        } else {
          "edit"
        },
        screen_width() * 0.2,
        screen_height() * 0.6,
        40.0,
        COLOR_1,
      );
      draw_text(
        if menu.cursor_position == vector![1, 0] {
          "-close-"
        } else {
          "close"
        },
        screen_width() * 0.5,
        screen_height() * 0.6,
        40.0,
        COLOR_1,
      );
    }
    /* MARK: Inventory pick slot */
    crate::menu::GameMenuKind::InventoryPickSlot(_, inventory_update) => {
      draw_rectangle(
        screen_width() * 0.45,
        screen_height() * 0.4,
        screen_width() * 0.5,
        screen_height() * 0.5,
        COLOR_2,
      );

      draw_text(
        if menu.cursor_position.x == 0 && menu.cursor_position.y == -1 {
          "-confirm-"
        } else {
          "confirm"
        },
        0.5 * screen_width(),
        0.45 * screen_height(),
        40.0,
        COLOR_1,
      );

      (0..4).for_each(|x| {
        (0..4).for_each(|y| {
          draw_rectangle(
            (0.5 + (x as f32 * 0.05)) * screen_width(),
            (0.5 + (y as f32 * 0.05)) * screen_height(),
            0.05 * screen_width(),
            0.05 * screen_height(),
            COLOR_3,
          );

          draw_rectangle(
            (0.51 + (x as f32 * 0.05)) * screen_width(),
            (0.51 + (y as f32 * 0.05)) * screen_height(),
            0.03 * screen_width(),
            0.03 * screen_height(),
            COLOR_2,
          );
        })
      });

      if menu.cursor_position.y > -1 {
        draw_rectangle(
          (0.5 + (menu.cursor_position.x as f32 * 0.05)) * screen_width(),
          (0.5 + (menu.cursor_position.y as f32 * 0.05)) * screen_height(),
          0.05 * screen_width(),
          0.05 * screen_height(),
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
                0.5 * screen_width(),
                (0.8 + (index as f32 * 0.02)) * screen_height(),
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
              (0.5113 + (module_x)) * screen_width(),
              (0.535 + (module_y)) * screen_height(),
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
                      (0.52 + module_x) * screen_width(),
                      (0.51 + module_y) * screen_height(),
                      0.01 * screen_width(),
                      0.005 * screen_height(),
                      COLOR_4,
                    );
                  }
                  Direction::Down => {
                    draw_rectangle(
                      (0.52 + module_x) * screen_width(),
                      (0.535 + module_y) * screen_height(),
                      0.01 * screen_width(),
                      0.005 * screen_height(),
                      COLOR_4,
                    );
                  }
                  Direction::Left => {
                    draw_rectangle(
                      (0.51 + module_x) * screen_width(),
                      (0.52 + module_y) * screen_height(),
                      0.005 * screen_width(),
                      0.01 * screen_height(),
                      COLOR_4,
                    );
                  }
                  Direction::Right => {
                    draw_rectangle(
                      (0.535 + module_x) * screen_width(),
                      (0.52 + module_y) * screen_height(),
                      0.005 * screen_width(),
                      0.01 * screen_height(),
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
            (0.5113 + (module_x)) * screen_width(),
            (0.535 + (module_y)) * screen_height(),
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
                    (0.52 + module_x) * screen_width(),
                    (0.51 + module_y) * screen_height(),
                    0.01 * screen_width(),
                    0.005 * screen_height(),
                    COLOR_4,
                  );
                }
                Direction::Down => {
                  draw_rectangle(
                    (0.52 + module_x) * screen_width(),
                    (0.535 + module_y) * screen_height(),
                    0.01 * screen_width(),
                    0.005 * screen_height(),
                    COLOR_4,
                  );
                }
                Direction::Left => {
                  draw_rectangle(
                    (0.51 + module_x) * screen_width(),
                    (0.52 + module_y) * screen_height(),
                    0.005 * screen_width(),
                    0.01 * screen_height(),
                    COLOR_4,
                  );
                }
                Direction::Right => {
                  draw_rectangle(
                    (0.535 + module_x) * screen_width(),
                    (0.52 + module_y) * screen_height(),
                    0.005 * screen_width(),
                    0.01 * screen_height(),
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
        screen_width() * 0.3,
        screen_height() * 0.45,
        screen_width() * 0.4,
        screen_height() * 0.1,
        COLOR_2,
      );

      draw_text(
        "Cancel",
        0.4 * screen_width(),
        0.5 * screen_height(),
        40.0,
        COLOR_1,
      );

      draw_text(
        "Save",
        0.6 * screen_width(),
        0.5 * screen_height(),
        40.0,
        COLOR_1,
      );

      draw_text(
        "-",
        (0.4 + (menu.cursor_position.x as f32 * 0.2)) * screen_width(),
        0.53 * screen_height(),
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::ModulePickupConfirm(weapon_module_kind) => {
      draw_rectangle(
        screen_width() * 0.3,
        screen_height() * 0.4,
        screen_width() * 0.4,
        screen_height() * 0.15,
        COLOR_2,
      );

      draw_text(
        &format!(
          "{} {} aquired",
          match weapon_module_from_kind(weapon_module_kind) {
            WeaponModule::Generator(_) => {
              "Weapon"
            }
            WeaponModule::Modulator(_, _) => {
              "Modifier"
            }
          },
          debug_module_symbol(weapon_module_kind)
        ),
        0.4 * screen_width(),
        0.45 * screen_height(),
        40.0,
        COLOR_1,
      );

      draw_text(
        "-edit-",
        0.4 * screen_width(),
        0.5 * screen_height(),
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::AbilityPickupConfirm(ability) => {
      draw_rectangle(
        screen_width() * 0.3,
        screen_height() * 0.4,
        screen_width() * 0.4,
        screen_height() * 0.15,
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
        0.4 * screen_width(),
        0.45 * screen_height(),
        40.0,
        COLOR_1,
      );

      draw_text(
        "-close-",
        0.4 * screen_width(),
        0.5 * screen_height(),
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::GameOver => {
      draw_rectangle(0.0, 0.0, screen_width(), screen_height(), COLOR_4);

      draw_text(
        "GAME OVER",
        0.4 * screen_width(),
        0.6 * screen_height(),
        40.0,
        COLOR_1,
      );
    }
    crate::menu::GameMenuKind::TerminalShow(terminal) => {
      draw_rectangle(
        0.25 * screen_width(),
        0.2 * screen_height(),
        0.5 * screen_width(),
        0.6 * screen_height(),
        COLOR_4,
      );

      draw_text(
        &terminal.created_at,
        0.265 * screen_width(),
        0.25 * screen_height(),
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
            0.265 * screen_width(),
            (0.35 + (0.025 * index as f32)) * screen_height(),
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
  }
}
