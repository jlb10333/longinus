use std::rc::Rc;

use balance::BALANCING;
use macroquad::prelude::*;
use shaders::{DISSOLVE_FRAGMENT_SHADER, IDENTITY_VERTEX_SHADER};
use system::ProcessContextOptions;

use crate::ability::AbilitySystem;
use crate::camera::CameraSystem;
use crate::combat::CombatSystem;
use crate::controls::ControlsSystem;
use crate::enemy::EnemySystem;
use crate::graphics::GraphicsSystem;
use crate::load_map::MapSystem;
use crate::menu::{MenuSystem, QuitDecision};
use crate::physics::PhysicsSystem;
use crate::save::{SaveData, SaveSystem, load_save};
use crate::system::{Process, System};

mod ability;
mod balance;
mod camera;
mod combat;
mod controls;
mod easing;
mod ecs;
mod effects;
mod enemy;
mod f;
mod graphics;
mod graphics_utils;
mod load_map;
mod menu;
mod physics;
mod save;
mod shaders;
mod sprite;
mod system;
mod units;

#[derive(Clone, Default)]
pub struct Start;

pub struct ProjectileTextures {
  pub plasma: Texture2D,
  pub missile: Texture2D,
  pub imp: Texture2D,
  pub beam: Texture2D,
  pub sniper: Texture2D,
}

pub struct PickupTextures {
  pub health_tank: Texture2D,
  pub weapon_module: Texture2D,
  pub health: Texture2D,
  pub mana: Texture2D,
}

pub struct BlockTextures {
  pub block: Texture2D,
}

pub struct ActivatorTextures {
  pub touch_sensor_activated: Texture2D,
  pub touch_sensor_deactivated: Texture2D,
}

pub struct EnemyTextures {
  pub goblin: Texture2D,
  pub imp: Texture2D,
  pub aranea: Texture2D,
  pub aranea_egg: Texture2D,
  pub sniper: Texture2D,
  pub laser_gate: Texture2D,
}

pub struct EffectTextures {
  pub noise: Texture2D,
  pub gravity_particle: Texture2D,
  pub explosion: Texture2D,
}

pub struct GameTextures {
  pub tiles_texture: Texture2D,
  pub player_texture: Texture2D,
  pub breakable_tile_texture: Texture2D,
  pub save_point_texture: Texture2D,
  pub projectile_textures: ProjectileTextures,
  pub pickup_textures: PickupTextures,
  pub block_textures: BlockTextures,
  pub activator_textures: ActivatorTextures,
  pub enemy_textures: EnemyTextures,
  pub effect_textures: EffectTextures,
}

impl Default for GameTextures {
  fn default() -> Self {
    panic!()
  }
}

pub struct GameMaterials {
  pub dissolve: Material,
}

impl Default for GameMaterials {
  fn default() -> Self {
    panic!()
  }
}

#[derive(Clone, Default)]
pub struct GameInput {
  pub save_data: SaveData,
  pub textures: Rc<GameTextures>,
  pub materials: Rc<GameMaterials>,
}

enum State {
  MainMenu,
  Game(SaveData),
  Exit,
}

fn window_conf() -> Conf {
  Conf {
    window_title: "Longinus".to_string(),
    window_width: 1920,
    window_height: 1080,
    ..Default::default()
  }
}

async fn load_texture_with_filter(path: &'static str) -> Texture2D {
  let texture = load_texture(path).await.unwrap();
  texture.set_filter(FilterMode::Nearest);
  texture
}

async fn load_game_textures() -> GameTextures {
  let tiles_texture = load_texture_with_filter("./assets/maps/tilesets/tiles.png").await;
  let player_texture = load_texture_with_filter("./assets/sprites/player.png").await;
  let plasma_texture = load_texture_with_filter("./assets/sprites/projectiles/plasma.png").await;
  let missile_texture = load_texture_with_filter("./assets/sprites/projectiles/missile.png").await;
  let imp_projectile_texture =
    load_texture_with_filter("./assets/sprites/projectiles/imp_projectile.png").await;
  let beam_texture = load_texture_with_filter("./assets/sprites/projectiles/beam.png").await;
  let sniper_projectile_texture =
    load_texture_with_filter("./assets/sprites/projectiles/sniper_projectile.png").await;
  let health_tank_texture =
    load_texture_with_filter("./assets/sprites/pickups/health_tank.png").await;
  let weapon_module_texture =
    load_texture_with_filter("./assets/sprites/pickups/weapon_module.png").await;
  let health_pickup_texture =
    load_texture_with_filter("./assets/sprites/pickups/health_pickup.png").await;
  let mana_pickup_texture =
    load_texture_with_filter("./assets/sprites/pickups/mana_pickup.png").await;
  let breakable_tile_texture =
    load_texture_with_filter("./assets/sprites/breakable_tile.png").await;
  let block_texture = load_texture_with_filter("./assets/sprites/blocks/block.png").await;
  let touch_sensor_deactivated_texture =
    load_texture_with_filter("./assets/sprites/activators/touch_sensor_deactivated.png").await;
  let touch_sensor_activated_texture =
    load_texture_with_filter("./assets/sprites/activators/touch_sensor_activated.png").await;
  let goblin_texture = load_texture_with_filter("./assets/sprites/enemies/goblin.png").await;
  let imp_texture = load_texture_with_filter("./assets/sprites/enemies/imp.png").await;
  let aranea_texture = load_texture_with_filter("./assets/sprites/enemies/aranea.png").await;
  let aranea_egg_texture =
    load_texture_with_filter("./assets/sprites/enemies/aranea_egg.png").await;
  let sniper_texture = load_texture_with_filter("./assets/sprites/enemies/sniper.png").await;
  let laser_gate_texture =
    load_texture_with_filter("./assets/sprites/enemies/laser_gate.png").await;
  let noise_texture = load_texture_with_filter("./assets/sprites/noise.png").await;
  let gravity_particle_texture =
    load_texture_with_filter("./assets/sprites/effects/gravity_particle.png").await;
  let explosion_texture = load_texture_with_filter("./assets/sprites/effects/explosion.png").await;
  let save_point_texture = load_texture_with_filter("./assets/sprites/save_point.png").await;

  GameTextures {
    tiles_texture,
    player_texture,
    breakable_tile_texture,
    save_point_texture,
    projectile_textures: ProjectileTextures {
      plasma: plasma_texture,
      missile: missile_texture,
      imp: imp_projectile_texture,
      beam: beam_texture,
      sniper: sniper_projectile_texture,
    },
    pickup_textures: PickupTextures {
      health_tank: health_tank_texture,
      weapon_module: weapon_module_texture,
      health: health_pickup_texture,
      mana: mana_pickup_texture,
    },
    block_textures: BlockTextures {
      block: block_texture,
    },
    activator_textures: ActivatorTextures {
      touch_sensor_activated: touch_sensor_activated_texture,
      touch_sensor_deactivated: touch_sensor_deactivated_texture,
    },
    enemy_textures: EnemyTextures {
      goblin: goblin_texture,
      imp: imp_texture,
      aranea: aranea_texture,
      aranea_egg: aranea_egg_texture,
      sniper: sniper_texture,
      laser_gate: laser_gate_texture,
    },
    effect_textures: EffectTextures {
      noise: noise_texture,
      gravity_particle: gravity_particle_texture,
      explosion: explosion_texture,
    },
  }
}

fn load_game_materials() -> GameMaterials {
  let dissolve_material = load_material(
    ShaderSource::Glsl {
      vertex: IDENTITY_VERTEX_SHADER,
      fragment: DISSOLVE_FRAGMENT_SHADER,
    },
    MaterialParams {
      uniforms: vec![
        UniformDesc::new("Progress", UniformType::Float1),
        UniformDesc::new("PixelsX", UniformType::Float1),
        UniformDesc::new("PixelsY", UniformType::Float1),
        UniformDesc::new("TextureOffset", UniformType::Float2),
        UniformDesc::new("TextureSize", UniformType::Float2),
      ],
      textures: vec!["NoiseTexture".to_string()],
      ..Default::default()
    },
  )
  .unwrap();

  GameMaterials {
    dissolve: dissolve_material,
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  // Load textures async
  let textures = Rc::new(load_game_textures().await);
  let materials = Rc::new(load_game_materials());

  let mut state = State::MainMenu;

  loop {
    state = match state {
      State::MainMenu => {
        let save_data = Process::new(&Start)
          .add_system(ControlsSystem::start)
          .add_system(SaveSystem::start)
          .add_system(MenuSystem::start)
          .add_system(GraphicsSystem::start)
          .start(None)
          .run(|ctx| {
            ctx
              .get::<MenuSystem<_>>()
              .unwrap()
              .save_to_load
              .as_ref()
              .map(load_save)
          })
          .await;
        State::Game(save_data)
      }
      State::Game(save_data) => {
        let game_input = GameInput {
          save_data,
          textures: Rc::clone(&textures),
          materials: Rc::clone(&materials),
        };

        let quit_decision = Process::new(&game_input)
          .add_system(SaveSystem::start)
          .add_system(CombatSystem::start)
          .add_system(MapSystem::start)
          .add_system(CameraSystem::start)
          .add_system(AbilitySystem::start)
          .add_system(PhysicsSystem::start)
          .add_system(ControlsSystem::start)
          .add_system(MenuSystem::start)
          .add_system(EnemySystem::start)
          .add_system(GraphicsSystem::start)
          .start(Some(ProcessContextOptions {
            should_freeze_fixed: Some(|ctx| {
              !ctx.get::<MenuSystem<_>>().unwrap().active_menus.is_empty()
                || (BALANCING.graphics_config.hitstop_enabled
                  && ctx.get::<PhysicsSystem>().unwrap().hitstop_frames_left > 0)
            }),
            ..Default::default()
          }))
          .run(|ctx| ctx.get::<MenuSystem<_>>().unwrap().quit_decision.clone())
          .await;
        match quit_decision {
          QuitDecision::LoadSave(save_to_load) => {
            State::Game(load_save(&menu::SaveToLoad::SaveData(save_to_load.clone())))
          }
          QuitDecision::ToMainMenu => State::MainMenu,
          QuitDecision::ToDesktop => State::Exit,
        }
      }
      State::Exit => break,
    };
  }
}
