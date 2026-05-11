use std::{cmp::Ordering, f32::consts::PI, marker::PhantomData, rc::Rc};

use itertools::Itertools;
use macroquad::prelude::*;
use rapier2d::prelude::*;

use crate::{
  ability::AbilitySystem,
  balance::BALANCING,
  camera::CameraSystem,
  combat::{
    CombatSystem, Direction, EQUIP_SLOTS_WIDTH, WeaponModule, WeaponModuleKind,
    distance_projection_screen, get_reticle_pos, get_slot_positions, weapon_module_from_kind,
  },
  controls::ControlsSystem,
  easing::{self, ease_out_cubic},
  ecs::{
    Activator, Damageable, Damager, Destroyed, Enemy, EntityHandle, GravitySource, Id,
    OnDestroyEffect, SimpleSprite, TouchSensor,
  },
  effects::{Effect, EffectKind},
  enemy::{EnemyAraneaState, EnemyImpState, EnemySniperState},
  graphics_utils::{draw_collider, draw_label},
  load_map::{ColliderLayer, MapSystem, physics_scalar_to_map, physics_translation_from_map},
  menu::{GameMenu, INVENTORY_WRAP_WIDTH, MainMenu, MenuSystem},
  physics::PhysicsSystem,
  save::SaveSystem,
  sprite::{self, SpriteToDraw, get_sprites_to_draw},
  system::System,
  units::{PhysicsScalar, PhysicsVector, ScreenVector, UnitConvert, UnitConvert2},
};

const RETICLE_SIZE: f32 = 3.0;

/* DEBUG OPTIONS */
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

#[derive(Clone)]
pub struct GraphicsSystem<Input> {
  effects: Vec<Effect>,
  _phantom_data: PhantomData<Input>,
}

const MINI_MAP_TILE_WIDTH: f32 = 2.0;
const MINI_MAP_TILE_HEIGHT: f32 = 2.0;

const TILEMAP_TILE_WIDTH: f32 = 8.0;
const TILEMAP_TILE_HEIGHT: f32 = 8.0;

impl<Input: Clone + Default + 'static> System for GraphicsSystem<Input> {
  type Input = Input;

  fn start(_: &crate::system::ProcessContext<Self::Input>) -> Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    Rc::new(GraphicsSystem {
      effects: vec![],
      _phantom_data: PhantomData,
    })
  }

  fn update(
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

      let tiles_texture = &ctx.input.textures.tiles_texture;

      // Draw tiles bg
      let tile_bg_layer = map_system.raw_map.tile_bg_layer();
      draw_tile_layer(tile_bg_layer, tiles_texture, &camera_system);

      let sorted_entities = physics_system
        .entities
        .iter()
        .sorted_by(
          |(handle_a, _), (handle_b, _)| match (*handle_a, *handle_b) {
            (EntityHandle::Collider(_), EntityHandle::RigidBody(_)) => Ordering::Less,
            (EntityHandle::RigidBody(_), EntityHandle::Collider(_)) => Ordering::Greater,
            (EntityHandle::Collider(handle_a), EntityHandle::Collider(handle_b)) => {
              if handle_a.0 < handle_b.0 {
                Ordering::Less
              } else {
                Ordering::Greater
              }
            }
            (EntityHandle::RigidBody(handle_a), EntityHandle::RigidBody(handle_b)) => {
              if handle_a.0 < handle_b.0 {
                Ordering::Less
              } else {
                Ordering::Greater
              }
            }
          },
        )
        .collect::<Vec<_>>();

      let new_effects: Vec<Effect> = sorted_entities
        .iter()
        .filter_map(|(handle, entity)| {
          let (physics_translation, rotation) = match *handle {
            EntityHandle::Collider(collider_handle) => {
              let collider = &physics_system.collider_set[*collider_handle];
              (collider.translation(), collider.rotation().angle())
            }
            EntityHandle::RigidBody(rigid_body_handle) => {
              let rigid_body = &physics_system.rigid_body_set[*rigid_body_handle];
              (rigid_body.translation(), rigid_body.rotation().angle())
            }
          };

          if let Some(damageable) = entity.components.get::<Damageable>()
            && (damageable.damaged
              || damageable.current_hitstun % 10.0 == 1.0
              || damageable.current_hitstun % 10.0 == 2.0)
            && (entity.components.get::<Destroyed>().is_none()
              || (entity.components.get::<Destroyed>().is_some()
                && entity.components.get::<OnDestroyEffect>().is_none()))
          {
            return None;
          }

          let translation =
            PhysicsVector::from_vec(*physics_translation).into_pos(camera_system.translation);

          let rotation = -(rotation * (16.0 / PI)).round() / (16.0 / PI);

          let sprites_override = if entity.components.get::<TouchSensor>().is_some()
            && let Some(activator) = entity.components.get::<Activator>()
          {
            Some(
              handle
                .colliders(&physics_system.rigid_body_set)
                .iter()
                .flat_map(|&&collider_handle| {
                  let collider = &physics_system.collider_set[collider_handle];
                  let dimensions = &PhysicsVector::from_vec(
                    collider.shape().as_cuboid().unwrap().half_extents * 2.0,
                  );

                  if activator.activation >= 0.5 {
                    sprite::touch_sensor_activated(dimensions, &ctx.input.textures)
                  } else {
                    sprite::touch_sensor_deactivated(dimensions, &ctx.input.textures)
                  }
                })
                .collect::<Vec<_>>(),
            )
          } else if let Some(enemy) = entity.components.get::<Enemy>() {
            match enemy.as_ref() {
              Enemy::Goblin(_goblin) => Some(sprite::goblin(
                (physics_system.frame_count / 15) as i32,
                &ctx.input.textures,
              )),
              Enemy::Imp(imp) => match imp.state {
                EnemyImpState::Moving(frames_left, _) => {
                  if frames_left
                    > BALANCING.enemies.imp.sprite_to_ball_frame_count
                      + BALANCING.enemies.imp.sprite_ball_frame_count
                  {
                    Some(sprite::imp(
                      (physics_system.frame_count / 15) as i32 % 2,
                      &ctx.input.textures,
                    ))
                  } else if frames_left > BALANCING.enemies.imp.sprite_ball_frame_count {
                    Some(sprite::imp(
                      ((frames_left - BALANCING.enemies.imp.sprite_ball_frame_count)
                        / BALANCING.enemies.imp.sprite_to_ball_frame_count)
                        % 2
                        + 2,
                      &ctx.input.textures,
                    ))
                  } else {
                    Some(sprite::imp(4, &ctx.input.textures))
                  }
                }
                EnemyImpState::ShootingCooldown(frames_left) => {
                  if BALANCING.enemies.imp.shooting_cooldown_initial_frames - frames_left < 7 {
                    Some(sprite::imp(5, &ctx.input.textures))
                  } else {
                    Some(sprite::imp(
                      (physics_system.frame_count / 15) as i32 % 2,
                      &ctx.input.textures,
                    ))
                  }
                }
                _ => Some(sprite::imp(
                  (physics_system.frame_count / 15) as i32 % 2,
                  &ctx.input.textures,
                )),
              },
              Enemy::Aranea(aranea) => {
                if let EnemyAraneaState::Idle = aranea.state {
                  Some(sprite::aranea(0, &ctx.input.textures))
                } else if let EnemyAraneaState::Detaching(frames_left, _, _) = aranea.state {
                  Some(sprite::aranea(
                    ((1 + BALANCING.enemies.aranea.detaching_initial_frames - frames_left) as f32
                      / (BALANCING.enemies.aranea.detaching_initial_frames as f32 / 5.0))
                      as i32,
                    &ctx.input.textures,
                  ))
                } else if let EnemyAraneaState::Cooldown(frames_left) = aranea.state
                  && (BALANCING.enemies.aranea.cooldown_initial_frames - frames_left) < 8
                {
                  Some(sprite::aranea(6, &ctx.input.textures))
                } else {
                  Some(sprite::aranea(
                    4 + ((physics_system.frame_count / 20) % 2) as i32,
                    &ctx.input.textures,
                  ))
                }
              }
              Enemy::Sniper(sniper) => {
                if let EnemySniperState::Cooldown(frames_left) = sniper.state {
                  Some(sprite::sniper(
                    1 + ((BALANCING.enemies.sniper.cooldown_initial_frames - frames_left)
                      / (BALANCING.enemies.sniper.cooldown_initial_frames / 3)),
                    &ctx.input.textures,
                  ))
                } else if let EnemySniperState::Shooting = sniper.state {
                  Some(sprite::sniper(5, &ctx.input.textures))
                } else {
                  Some(sprite::sniper(0, &ctx.input.textures))
                }
              }
              _ => None,
            }
          } else {
            None
          };

          let sprites_to_draw = sprites_override.or_else(|| {
            let sprite = entity.components.get::<SimpleSprite>()?;

            Some(get_sprites_to_draw(
              &sprite.kind,
              physics_system.frame_count,
              ctx.input.textures.as_ref(),
            ))
          });

          let sprites_to_draw = sprites_to_draw?;

          draw_sprites(&sprites_to_draw, translation, rotation);

          if entity.components.get::<Destroyed>().is_some()
            && let Some(destroy_effect) = entity.components.get::<OnDestroyEffect>()
          {
            let easing = easing::linear()
              .scale(destroy_effect.duration as f32)
              .offset(physics_system.frame_count as f32);

            Some(Effect {
              easing,
              kind: destroy_effect.effect_kind,
              sprites_to_draw: Rc::new(sprites_to_draw),
              translation: PhysicsVector::from_vec(*physics_translation),
              rotation,
            })
          } else {
            None
          }
        })
        .collect::<Vec<_>>();

      let effects = self
        .effects
        .iter()
        .cloned()
        .chain(new_effects)
        .filter(|effect| effect.easing.at(physics_system.frame_count as f32) <= 1.0)
        .collect::<Vec<_>>();

      effects.iter().for_each(|effect| {
        let material = match &effect.kind {
          EffectKind::NoiseDissolve => {
            let dissolve = ctx.input.materials.dissolve.clone();
            dissolve.set_uniform(
              "Progress",
              effect.easing.at(physics_system.frame_count as f32),
            );
            dissolve.set_texture(
              "NoiseTexture",
              ctx.input.textures.effect_textures.noise.weak_clone(),
            );
            dissolve
          }
        };

        let sprites_to_draw = effect
          .sprites_to_draw
          .iter()
          .map(|sprite_to_draw| {
            let material = material.clone();
            sprite_to_draw.with_material(Some(material))
          })
          .collect::<Vec<_>>();

        draw_sprites(
          &sprites_to_draw,
          effect.translation.into_pos(camera_system.translation),
          effect.rotation,
        );
      });

      let tile_layer = map_system.raw_map.tile_layer();
      draw_tile_layer(tile_layer, tiles_texture, &camera_system);

      /* Debug */
      if BALANCING.debug.show_colliders {
        sorted_entities.iter().for_each(|(handle, entity)| {
          if let Some(damageable) = entity.components.get::<Damageable>()
            && damageable.damaged
          {
            return;
          }

          if let Some(gravity_source) = entity.components.get::<GravitySource>()
            && let EntityHandle::Collider(handle) = entity.handle
            && let Some(ball) = physics_system.collider_set[handle].shape().as_ball()
          {
            let activation = if let Some(target_activator_id) = gravity_source.activator_id
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
              activator.activation
            } else {
              1.0
            };

            let gravity_strength = gravity_source.strength * activation;

            if gravity_strength == 0.0 {
              return;
            }

            let ball_radius_screen = *PhysicsScalar(ball.radius).convert();

            let period = (gravity_strength * ball_radius_screen * 30.0).abs();

            let ball_1_completion_ratio = (physics_system.frame_count as f32 % period) / period;

            let ball_1_completion_ratio = if gravity_strength > 0.0 {
              1.0 - ball_1_completion_ratio
            } else {
              ball_1_completion_ratio
            };

            let ball_1_radius = ball_1_completion_ratio * ball_radius_screen;

            let ball_2_completion_ratio =
              ((physics_system.frame_count as f32 + (period / 2.0)) % period) / period;

            let ball_2_completion_ratio = if gravity_strength > 0.0 {
              1.0 - ball_2_completion_ratio
            } else {
              ball_2_completion_ratio
            };

            let ball_2_radius = ball_2_completion_ratio * ball_radius_screen;

            let translation =
              PhysicsVector::from_vec(*physics_system.collider_set[handle].translation())
                .into_pos(camera_system.translation);

            let alpha_ease = ease_out_cubic() * 0.15;

            draw_circle(
              translation.x(),
              translation.y(),
              ball_1_radius,
              COLOR_3.with_alpha(alpha_ease.at(1.0 - ball_1_completion_ratio)),
            );
            draw_circle(
              translation.x(),
              translation.y(),
              ball_2_radius,
              COLOR_3.with_alpha(alpha_ease.at(1.0 - ball_2_completion_ratio)),
            );
            draw_circle(
              translation.x(),
              translation.y(),
              ball_radius_screen,
              COLOR_2.with_alpha(0.2),
            );

            return;
          }

          let color = if let Some(enemy) = entity.components.get::<Enemy>()
            && let Enemy::Sniper(sniper) = enemy.as_ref()
            && let EnemySniperState::Cooldown(frames_left) = sniper.state
          {
            let color_diff = COLOR_2.to_vec() - COLOR_4.to_vec();
            let color_ease = easing::ease_out_cubic() * color_diff;

            let current_color = COLOR_4.to_vec()
              + color_ease
                .at(frames_left as f32 / BALANCING.enemies.sniper.cooldown_initial_frames as f32);

            Some(Color::from_vec(current_color))
          } else if entity.components.get::<Damager>().is_some() {
            Some(COLOR_3)
          } else {
            None
          };

          handle
            .colliders(&physics_system.rigid_body_set)
            .iter()
            .for_each(|&&collider_handle| {
              let collider = &physics_system.collider_set[collider_handle];
              draw_collider(collider, camera_system.translation, None, color, None);
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
          .sorted_by(|(handle_a, _), (handle_b, _)| {
            if handle_a.0 > handle_b.0 {
              Ordering::Greater
            } else {
              Ordering::Less
            }
          })
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
            slot_screen_pos.into_vec() + distance_projection_screen(slot.angle, 9.0).into_vec(),
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
          player_damageable.health.round(),
          player_damageable.max_health.round()
        ),
        screen_width() * 0.01,
        screen_height() * 0.8,
        40.0,
        COLOR_4,
      );

      let ability_system = ctx.get::<AbilitySystem>().unwrap();

      let rechargeable_mana_percent_filled = ability_system.mana_tanks.rechargeable_mana_level
        / ability_system
          .mana_tanks
          .capacity
          .max_rechargeable_mana_level();

      let max_rechargeable_mana = ability_system
        .mana_tanks
        .capacity
        .max_rechargeable_mana_level()
        * 10.0;

      draw_arc(
        player_screen_pos.x(),
        player_screen_pos.y(),
        128,
        70.0,
        0.0,
        10.0,
        max_rechargeable_mana,
        COLOR_4.with_alpha(0.75),
      );
      draw_arc(
        player_screen_pos.x(),
        player_screen_pos.y(),
        128,
        70.0,
        0.0,
        10.0,
        rechargeable_mana_percent_filled * max_rechargeable_mana,
        COLOR_2.with_alpha(0.75),
      );

      let unrechargeable_mana_percent_filled =
        ability_system.mana_tanks.non_rechargeable_mana_level
          / ability_system
            .mana_tanks
            .capacity
            .max_non_rechargeable_mana_level();

      let max_unrechargeable_mana = ability_system
        .mana_tanks
        .capacity
        .max_non_rechargeable_mana_level()
        * 10.0;

      draw_arc(
        player_screen_pos.x(),
        player_screen_pos.y(),
        128,
        70.0,
        -unrechargeable_mana_percent_filled * max_unrechargeable_mana,
        10.0,
        unrechargeable_mana_percent_filled * max_unrechargeable_mana,
        COLOR_3,
      );

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

      return Rc::new(GraphicsSystem {
        effects,
        _phantom_data: PhantomData,
      });
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

    Rc::new(GraphicsSystem {
      effects: self.effects.clone(),
      _phantom_data: PhantomData,
    })
  }

  fn fixed_update(
    &self,
    _: &crate::system::ProcessContext<Self::Input>,
  ) -> Rc<dyn System<Input = Self::Input>> {
    Rc::new(self.clone())
  }
}

fn draw_tile_layer(
  layer: &ColliderLayer,
  target_texture: &Texture2D,
  camera_system: &Rc<CameraSystem>,
) {
  layer
    .data
    .iter()
    .enumerate()
    .for_each(|(index, &tile_data)| {
      if tile_data == 0 {
        return;
      }

      let map_x = index as i32 % layer.width;
      let map_y = index as i32 / layer.width;

      let physics_translation = PhysicsVector::from_vec(physics_translation_from_map(
        map_x as f32 * 8.0,
        map_y as f32 * 8.0,
        0.0,
        0.0,
        layer.height as f32 * 8.0,
      ));
      let screen_translation = physics_translation.into_pos(camera_system.translation);

      let actual_data = (tile_data - 4) as f32;

      let source = Rect {
        x: (actual_data % TILEMAP_TILE_WIDTH) * TILEMAP_TILE_WIDTH,
        y: (actual_data / TILEMAP_TILE_WIDTH).floor() * TILEMAP_TILE_HEIGHT,
        h: TILEMAP_TILE_WIDTH,
        w: TILEMAP_TILE_HEIGHT,
      };

      draw_texture_ex(
        target_texture,
        screen_translation.x(),
        screen_translation.y(),
        WHITE,
        DrawTextureParams {
          dest_size: Some(Vec2 {
            x: 64.0 * BALANCING.graphics_config.scaling_factor,
            y: 64.0 * BALANCING.graphics_config.scaling_factor,
          }),
          source: Some(source),
          rotation: 0.0,
          flip_x: false,
          flip_y: false,
          pivot: None,
        },
      );
    });
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

pub fn draw_sprites(sprites_to_draw: &[SpriteToDraw], translation: ScreenVector, rotation: f32) {
  sprites_to_draw.iter().for_each(|sprite_to_draw| {
    let adjusted_width = sprite_to_draw.source.w * 8.0 * BALANCING.graphics_config.scaling_factor;
    let adjusted_height = sprite_to_draw.source.h * 8.0 * BALANCING.graphics_config.scaling_factor;

    let offset_x = sprite_to_draw
      .offset
      .map(|offset| offset.x * 8.0 * BALANCING.graphics_config.scaling_factor)
      .unwrap_or(0.0);

    let offset_y = sprite_to_draw
      .offset
      .map(|offset| offset.y * 8.0 * BALANCING.graphics_config.scaling_factor)
      .unwrap_or(0.0);

    let new_offset_x = offset_x * rotation.cos() - offset_y * rotation.sin();
    let new_offset_y = offset_x * rotation.sin() + offset_y * rotation.cos();

    let dest_x = translation.x() - (adjusted_width / 2.0) + new_offset_x;

    let dest_y = translation.y() - (adjusted_height / 2.0) + new_offset_y;

    if let Some(material) = sprite_to_draw.material.as_ref() {
      material.set_uniform("PixelsX", sprite_to_draw.source.w);
      material.set_uniform("PixelsY", sprite_to_draw.source.h);
      material.set_uniform_array(
        "TextureOffset",
        &[sprite_to_draw.source.x, sprite_to_draw.source.y],
      );
      material.set_uniform_array(
        "TextureSize",
        &[
          sprite_to_draw.texture.size().x,
          sprite_to_draw.texture.size().y,
        ],
      );

      gl_use_material(material);
    }

    draw_texture_ex(
      sprite_to_draw.texture.as_ref(),
      dest_x,
      dest_y,
      WHITE,
      DrawTextureParams {
        dest_size: Some(Vec2 {
          x: adjusted_width,
          y: adjusted_height,
        }),
        source: Some(sprite_to_draw.source),
        rotation,
        flip_x: false,
        flip_y: false,
        pivot: None,
      },
    );

    gl_use_default_material();
  });
}
