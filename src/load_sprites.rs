use crate::load_sprites_utils::load_texture_with_filter;
use macroquad::prelude::Texture2D;
pub struct SpriteTextures {
  abilities: AbilitiesTextures,
  activators: ActivatorsTextures,
  blocks: BlocksTextures,
  breakable_tile_texture: Texture2D,
  effects: EffectsTextures,
  enemies: EnemiesTextures,
  noise_texture: Texture2D,
  pickups: PickupsTextures,
  player_texture: Texture2D,
  projectiles: ProjectilesTextures,
  save_point_texture: Texture2D,
  ui: UiTextures,
}
pub struct AbilitiesTextures {
  chain_mount_point_selection_texture: Texture2D,
  chain_texture: Texture2D,
}
pub struct ActivatorsTextures {
  touch_sensor_activated_texture: Texture2D,
  touch_sensor_deactivated_texture: Texture2D,
}
pub struct BlocksTextures {
  angelic_block_texture: Texture2D,
  block_texture: Texture2D,
}
pub struct EffectsTextures {
  explosion_texture: Texture2D,
  gravity_particle_texture: Texture2D,
}
pub struct EnemiesTextures {
  aranea_egg_texture: Texture2D,
  aranea_texture: Texture2D,
  aranea_queen_texture: Texture2D,
  defender_texture: Texture2D,
  defender_prime_texture: Texture2D,
  goblin_texture: Texture2D,
  imp_texture: Texture2D,
  laser_gate_texture: Texture2D,
  seeker_texture: Texture2D,
  sniper_texture: Texture2D,
}
pub struct PickupsTextures {
  health_pickup_texture: Texture2D,
  health_tank_texture: Texture2D,
  mana_pickup_texture: Texture2D,
  mana_tank_texture: Texture2D,
  weapon_module_texture: Texture2D,
}
pub struct ProjectilesTextures {
  aranea_queen_projectile_texture: Texture2D,
  beam_texture: Texture2D,
  imp_projectile_texture: Texture2D,
  missile_texture: Texture2D,
  plasma_texture: Texture2D,
  sniper_projectile_texture: Texture2D,
}
pub struct UiTextures {
  enemy_offscreen_texture: Texture2D,
  menu_texture: Texture2D,
  modules: ModulesTextures,
  text_font_texture: Texture2D,
}
pub struct ModulesTextures {
  module_double_damage_75_freq_texture: Texture2D,
  module_double_freq_75_damage_texture: Texture2D,
  module_empty_texture: Texture2D,
  module_fortyfive_slot_texture: Texture2D,
  module_front_2_slot_texture: Texture2D,
  module_mana_cost_texture: Texture2D,
  module_mana_free_texture: Texture2D,
  module_mirror_slot_texture: Texture2D,
  module_missile_texture: Texture2D,
  module_plasma_texture: Texture2D,
  module_side_slot_texture: Texture2D,
  module_status_deteriorate_texture: Texture2D,
  module_status_vulnerable_texture: Texture2D,
  module_status_weakness_texture: Texture2D,
}
pub async fn load_game_textures() -> SpriteTextures {
  let mana_pickup_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickups/mana_pickup.png").await;
  let mana_tank_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickups/mana_tank.png").await;
  let health_tank_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickups/health_tank.png").await;
  let health_pickup_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickups/health_pickup.png").await;
  let weapon_module_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickups/weapon_module.png").await;
  let breakable_tile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/breakable_tile.png").await;
  let save_point_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/save_point.png").await;
  let touch_sensor_deactivated_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/activators/touch_sensor_deactivated.png").await;
  let touch_sensor_activated_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/activators/touch_sensor_activated.png").await;
  let module_fortyfive_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_fortyfive_slot.png").await;
  let module_front_2_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_front_2_slot.png").await;
  let module_plasma_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_plasma.png").await;
  let module_double_freq_75_damage_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_double_freq_75_damage.png").await;
  let module_status_weakness_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_status_weakness.png").await;
  let module_status_deteriorate_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_status_deteriorate.png").await;
  let module_side_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_side_slot.png").await;
  let module_missile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_missile.png").await;
  let module_empty_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_empty.png").await;
  let module_mana_free_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_mana_free.png").await;
  let module_status_vulnerable_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_status_vulnerable.png").await;
  let module_mirror_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_mirror_slot.png").await;
  let module_mana_cost_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_mana_cost.png").await;
  let module_double_damage_75_freq_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/modules/module_double_damage_75_freq.png").await;
  let text_font_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/text_font.png").await;
  let enemy_offscreen_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/enemy_offscreen.png").await;
  let menu_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/menu.png").await;
  let explosion_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/effects/explosion.png").await;
  let gravity_particle_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/effects/gravity_particle.png").await;
  let seeker_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/seeker.png").await;
  let aranea_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/aranea.png").await;
  let defender_prime_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/defender_prime.png").await;
  let laser_gate_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/laser_gate.png").await;
  let defender_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/defender.png").await;
  let aranea_queen_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/aranea_queen.png").await;
  let sniper_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/sniper.png").await;
  let aranea_egg_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/aranea_egg.png").await;
  let imp_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/imp.png").await;
  let goblin_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemies/goblin.png").await;
  let chain_mount_point_selection_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/abilities/chain_mount_point_selection.png").await;
  let chain_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/abilities/chain.png").await;
  let imp_projectile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectiles/imp_projectile.png").await;
  let aranea_queen_projectile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectiles/aranea_queen_projectile.png").await;
  let sniper_projectile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectiles/sniper_projectile.png").await;
  let plasma_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectiles/plasma.png").await;
  let beam_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectiles/beam.png").await;
  let missile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectiles/missile.png").await;
  let block_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/blocks/block.png").await;
  let angelic_block_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/blocks/angelic_block.png").await;
  let noise_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/noise.png").await;
  let player_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/player.png").await;
  SpriteTextures {
    abilities: AbilitiesTextures {
      chain_mount_point_selection_texture,
      chain_texture,
    },
    activators: ActivatorsTextures {
      touch_sensor_activated_texture,
      touch_sensor_deactivated_texture,
    },
    blocks: BlocksTextures {
      angelic_block_texture,
      block_texture,
    },
    breakable_tile_texture,
    effects: EffectsTextures {
      explosion_texture,
      gravity_particle_texture,
    },
    enemies: EnemiesTextures {
      aranea_egg_texture,
      aranea_texture,
      aranea_queen_texture,
      defender_texture,
      defender_prime_texture,
      goblin_texture,
      imp_texture,
      laser_gate_texture,
      seeker_texture,
      sniper_texture,
    },
    noise_texture,
    pickups: PickupsTextures {
      health_pickup_texture,
      health_tank_texture,
      mana_pickup_texture,
      mana_tank_texture,
      weapon_module_texture,
    },
    player_texture,
    projectiles: ProjectilesTextures {
      aranea_queen_projectile_texture,
      beam_texture,
      imp_projectile_texture,
      missile_texture,
      plasma_texture,
      sniper_projectile_texture,
    },
    save_point_texture,
    ui: UiTextures {
      enemy_offscreen_texture,
      menu_texture,
      modules: ModulesTextures {
        module_double_damage_75_freq_texture,
        module_double_freq_75_damage_texture,
        module_empty_texture,
        module_fortyfive_slot_texture,
        module_front_2_slot_texture,
        module_mana_cost_texture,
        module_mana_free_texture,
        module_mirror_slot_texture,
        module_missile_texture,
        module_plasma_texture,
        module_side_slot_texture,
        module_status_deteriorate_texture,
        module_status_vulnerable_texture,
        module_status_weakness_texture,
      },
      text_font_texture,
    },
  }
}
