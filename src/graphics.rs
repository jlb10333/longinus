use std::{rc::Rc, thread::sleep, time::Duration};

use macroquad::prelude::*;
use rapier2d::prelude::*;

use crate::{
  camera::CameraSystem,
  combat::{
    CombatSystem, EQUIP_SLOTS_WIDTH, WeaponModuleKind, distance_projection_screen, get_reticle_pos,
    get_slot_positions,
  },
  ecs::MapTransitionOnCollision,
  graphics_utils::draw_collider,
  menu::{INVENTORY_WRAP_WIDTH, Menu, MenuSystem},
  physics::PhysicsSystem,
  system::System,
  units::{PhysicsVector, ScreenVector, UnitConvert, UnitConvert2},
};

const TARGET_FPS: f32 = 60.0;
const MIN_FRAME_TIME: f32 = 1.0 / TARGET_FPS;

const RETICLE_SIZE: f32 = 3.0;

/* DEBUG OPTIONS */
const SHOW_COLLIDERS: bool = true;
const SHOW_SLOTS: bool = true;

pub struct GraphicsSystem;

impl System for GraphicsSystem {
  fn start(_: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    return Rc::new(GraphicsSystem);
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let camera_system = ctx.get::<CameraSystem>().unwrap();
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();
    let combat_system = ctx.get::<CombatSystem>().unwrap();

    /* Background */
    clear_background(RED);

    /* Debug */
    if SHOW_COLLIDERS {
      physics_system
        .collider_set
        .iter()
        .for_each(|(_, collider)| draw_collider(collider, camera_system.translation, None));
    }

    /* Draw player */
    let player_screen_pos = PhysicsVector::from_vec(
      *physics_system.rigid_body_set[physics_system.player_handle].translation(),
    )
    .into_pos(camera_system.translation);

    draw_circle(player_screen_pos.x(), player_screen_pos.y(), 12.5, GREEN);

    /* Draw reticle */
    let reticle_pos = get_reticle_pos(combat_system.reticle_angle);

    draw_circle(
      player_screen_pos.x() + reticle_pos.x(),
      player_screen_pos.y() + reticle_pos.y(),
      RETICLE_SIZE,
      BLACK,
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

        draw_circle(slot_screen_pos.x(), slot_screen_pos.y(), 2.0, BLUE);
        draw_circle(
          slot_next_screen_pos.x(),
          slot_next_screen_pos.y(),
          2.0,
          WHITE,
        );
      });
    }

    /* Draw the scuffed menu */
    let menu_system = ctx.get::<MenuSystem>().unwrap();

    menu_system.active_menus.iter().rev().for_each(draw_menu);

    /* Maintain target fps */
    let frame_time = get_frame_time();

    if frame_time < MIN_FRAME_TIME {
      let time_to_sleep = (MIN_FRAME_TIME - frame_time) * 1000.0; // Calculate sleep time in ms
      sleep(Duration::from_millis(time_to_sleep as u64)); // Sleep
    }

    return Rc::new(GraphicsSystem);
  }
}

fn draw_menu(menu: &Menu) {
  match menu.kind.clone() {
    crate::menu::MenuKind::InventoryMain => {
      draw_rectangle(
        screen_width() * 0.1,
        screen_height() * 0.1,
        screen_width() * 0.8,
        screen_height() * 0.8,
        BLUE,
      );

      draw_text(
        "inventory",
        screen_width() * 0.2,
        screen_height() * 0.3,
        40.0,
        WHITE,
      );

      draw_text(
        if menu.cursor_position == vector![0, 0] {
          "edit-"
        } else {
          "edit"
        },
        screen_width() * 0.2,
        screen_height() * 0.6,
        40.0,
        WHITE,
      );
      draw_text(
        if menu.cursor_position == vector![1, 0] {
          "close-"
        } else {
          "close"
        },
        screen_width() * 0.5,
        screen_height() * 0.6,
        40.0,
        WHITE,
      );
    }
    crate::menu::MenuKind::InventoryPickSlot(_, inventory_update) => {
      draw_rectangle(
        screen_width() * 0.45,
        screen_height() * 0.45,
        screen_width() * 0.5,
        screen_height() * 0.5,
        LIGHTGRAY,
      );

      draw_text(
        "-",
        (0.5 + (menu.cursor_position.x as f32 * 0.05)) * screen_width(),
        (0.5 + (menu.cursor_position.y as f32 * 0.05)) * screen_height(),
        40.0,
        WHITE,
      );

      inventory_update
        .equipped_modules
        .iter()
        .enumerate()
        .for_each(|(index, equipped_module)| {
          equipped_module.clone().map(|module_kind| {
            draw_text(
              debug_module_text(&module_kind),
              (0.5 + ((index as i32 % EQUIP_SLOTS_WIDTH) as f32 * 0.05)) * screen_width(),
              (0.5 + ((index as i32 / EQUIP_SLOTS_WIDTH) as f32 * 0.05)) * screen_height(),
              40.0,
              WHITE,
            );
          });
        });

      inventory_update
        .unequipped_modules
        .iter()
        .enumerate()
        .for_each(|(index, unequipped_module_kind)| {
          draw_text(
            debug_module_text(unequipped_module_kind),
            (0.5
              + (EQUIP_SLOTS_WIDTH as f32 * 0.05)
              + ((index as i32 % INVENTORY_WRAP_WIDTH) as f32 * 0.05))
              * screen_width(),
            (0.5 + ((index as i32 / INVENTORY_WRAP_WIDTH) as f32 * 0.05)) * screen_height(),
            40.0,
            WHITE,
          );
        });
    }
    crate::menu::MenuKind::InventoryConfirmEdit(_) => {}
    crate::menu::MenuKind::PauseMain => {}
  }
}

fn debug_module_text(module_kind: &WeaponModuleKind) -> &'static str {
  match module_kind {
    WeaponModuleKind::Plasma => "P",
    WeaponModuleKind::DoubleDamage => "D",
    WeaponModuleKind::Front2Slot => "2",
  }
}
