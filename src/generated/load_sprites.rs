//b5f735e1a92fa59a99556b495eb1cc7033462d0fbe0ffcfd2f7b4b8abfebe56f  -
use crate::load_sprites_utils::load_texture_with_filter;
use macroquad::prelude::Texture2D;
pub struct SpriteTextures {
  pub ability_textures: AbilityTextures,
  pub activator_textures: ActivatorTextures,
  pub block_textures: BlockTextures,
  pub breakable_tile_texture: Texture2D,
  pub effect_textures: EffectTextures,
  pub enemy_textures: EnemyTextures,
  pub pickup_textures: PickupTextures,
  pub player_texture: Texture2D,
  pub projectile_textures: ProjectileTextures,
  pub save_point_texture: Texture2D,
  pub tileset_textures: TilesetTextures,
  pub ui_textures: UiTextures,
}
pub struct AbilityTextures {
  pub chain_mount_point_selection_texture: Texture2D,
  pub chain_texture: Texture2D,
}
pub struct ActivatorTextures {
  pub touch_sensor_activated_texture: Texture2D,
  pub touch_sensor_deactivated_texture: Texture2D,
}
pub struct BlockTextures {
  pub angelic_block_texture: Texture2D,
  pub block_texture: Texture2D,
}
pub struct EffectTextures {
  pub explosion_texture: Texture2D,
  pub gravity_particle_texture: Texture2D,
  pub noise_texture: Texture2D,
}
pub struct EnemyTextures {
  pub aranea_egg_texture: Texture2D,
  pub aranea_texture: Texture2D,
  pub aranea_queen_texture: Texture2D,
  pub defender_texture: Texture2D,
  pub defender_prime_texture: Texture2D,
  pub goblin_texture: Texture2D,
  pub imp_texture: Texture2D,
  pub laser_gate_texture: Texture2D,
  pub seeker_texture: Texture2D,
  pub sniper_texture: Texture2D,
}
pub struct PickupTextures {
  pub health_pickup_texture: Texture2D,
  pub health_tank_texture: Texture2D,
  pub mana_pickup_texture: Texture2D,
  pub mana_tank_texture: Texture2D,
  pub weapon_module_texture: Texture2D,
}
pub struct ProjectileTextures {
  pub aranea_queen_projectile_texture: Texture2D,
  pub beam_texture: Texture2D,
  pub imp_projectile_texture: Texture2D,
  pub missile_texture: Texture2D,
  pub plasma_texture: Texture2D,
  pub sniper_projectile_texture: Texture2D,
}
pub struct TilesetTextures {
  pub tiles_texture: Texture2D,
}
pub struct UiTextures {
  pub enemy_offscreen_texture: Texture2D,
  pub menu_texture: Texture2D,
  pub module_textures: ModuleTextures,
  pub text_font_texture: Texture2D,
}
pub struct ModuleTextures {
  pub module_cursor_texture: Texture2D,
  pub module_double_damage_75_freq_texture: Texture2D,
  pub module_double_freq_75_damage_texture: Texture2D,
  pub module_empty_texture: Texture2D,
  pub module_fortyfive_slot_texture: Texture2D,
  pub module_front_2_slot_texture: Texture2D,
  pub module_mana_cost_texture: Texture2D,
  pub module_mana_free_texture: Texture2D,
  pub module_mirror_slot_texture: Texture2D,
  pub module_missile_texture: Texture2D,
  pub module_plasma_texture: Texture2D,
  pub module_side_slot_texture: Texture2D,
  pub module_status_deteriorate_texture: Texture2D,
  pub module_status_vulnerable_texture: Texture2D,
  pub module_status_weakness_texture: Texture2D,
}
pub async fn load_game_textures() -> SpriteTextures {
  let chain_mount_point_selection_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ability/chain_mount_point_selection.png").await;
  let chain_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ability/chain.png").await;
  let block_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/block/block.png").await;
  let angelic_block_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/block/angelic_block.png").await;
  let breakable_tile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/breakable_tile.png").await;
  let save_point_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/save_point.png").await;
  let explosion_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/effect/explosion.png").await;
  let gravity_particle_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/effect/gravity_particle.png").await;
  let noise_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/effect/noise.png").await;
  let mana_pickup_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickup/mana_pickup.png").await;
  let mana_tank_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickup/mana_tank.png").await;
  let health_tank_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickup/health_tank.png").await;
  let health_pickup_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickup/health_pickup.png").await;
  let weapon_module_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/pickup/weapon_module.png").await;
  let text_font_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/text_font.png").await;
  let enemy_offscreen_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/enemy_offscreen.png").await;
  let module_fortyfive_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_fortyfive_slot.png").await;
  let module_front_2_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_front_2_slot.png").await;
  let module_plasma_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_plasma.png").await;
  let module_double_freq_75_damage_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_double_freq_75_damage.png").await;
  let module_status_weakness_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_status_weakness.png").await;
  let module_status_deteriorate_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_status_deteriorate.png").await;
  let module_side_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_side_slot.png").await;
  let module_missile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_missile.png").await;
  let module_empty_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_empty.png").await;
  let module_mana_free_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_mana_free.png").await;
  let module_status_vulnerable_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_status_vulnerable.png").await;
  let module_mirror_slot_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_mirror_slot.png").await;
  let module_cursor_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_cursor.png").await;
  let module_mana_cost_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_mana_cost.png").await;
  let module_double_damage_75_freq_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/module/module_double_damage_75_freq.png").await;
  let menu_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/ui/menu.png").await;
  let seeker_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/seeker.png").await;
  let aranea_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/aranea.png").await;
  let defender_prime_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/defender_prime.png").await;
  let laser_gate_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/laser_gate.png").await;
  let defender_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/defender.png").await;
  let aranea_queen_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/aranea_queen.png").await;
  let sniper_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/sniper.png").await;
  let aranea_egg_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/aranea_egg.png").await;
  let imp_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/imp.png").await;
  let goblin_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/enemy/goblin.png").await;
  let touch_sensor_deactivated_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/activator/touch_sensor_deactivated.png").await;
  let touch_sensor_activated_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/activator/touch_sensor_activated.png").await;
  let tiles_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/tileset/tiles.png").await;
  let player_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/player.png").await;
  let imp_projectile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectile/imp_projectile.png").await;
  let aranea_queen_projectile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectile/aranea_queen_projectile.png").await;
  let sniper_projectile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectile/sniper_projectile.png").await;
  let plasma_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectile/plasma.png").await;
  let beam_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectile/beam.png").await;
  let missile_texture = load_texture_with_filter("/home/jack/longinus/assets/sprite/projectile/missile.png").await;
  SpriteTextures {
    ability_textures: AbilityTextures {
      chain_mount_point_selection_texture,
      chain_texture,
    },
    activator_textures: ActivatorTextures {
      touch_sensor_activated_texture,
      touch_sensor_deactivated_texture,
    },
    block_textures: BlockTextures {
      angelic_block_texture,
      block_texture,
    },
    breakable_tile_texture,
    effect_textures: EffectTextures {
      explosion_texture,
      gravity_particle_texture,
      noise_texture,
    },
    enemy_textures: EnemyTextures {
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
    pickup_textures: PickupTextures {
      health_pickup_texture,
      health_tank_texture,
      mana_pickup_texture,
      mana_tank_texture,
      weapon_module_texture,
    },
    player_texture,
    projectile_textures: ProjectileTextures {
      aranea_queen_projectile_texture,
      beam_texture,
      imp_projectile_texture,
      missile_texture,
      plasma_texture,
      sniper_projectile_texture,
    },
    save_point_texture,
    tileset_textures: TilesetTextures {
      tiles_texture,
    },
    ui_textures: UiTextures {
      enemy_offscreen_texture,
      menu_texture,
      module_textures: ModuleTextures {
        module_cursor_texture,
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
