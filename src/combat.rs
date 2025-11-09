use std::{
  any::Any,
  array::from_fn,
  collections::{HashMap, HashSet},
  f32::consts::PI,
  rc::Rc,
};

use crate::{
  controls::{ControlsSystem, angle_from_vec},
  f::Monad,
  load_map::{
    COLLISION_GROUP_ENEMY, COLLISION_GROUP_PLAYER_PROJECTILE, COLLISION_GROUP_WALL, MapSystem,
  },
  menu::MenuSystem,
  physics::PhysicsSystem,
  system::System,
  units::{PhysicsVector, ScreenVector, UnitConvert, UnitConvert2, vec_zero},
};
use rapier2d::{
  na::{ArrayStorage, Const, Matrix, Vector2},
  prelude::*,
};

pub fn distance_projection_physics(angle: f32, distance: f32) -> PhysicsVector {
  return PhysicsVector::from_vec(vector![
    angle.cos() * distance,
    -1.0 * angle.sin() * distance
  ]);
}

pub fn distance_projection_screen(angle: f32, distance: f32) -> ScreenVector {
  return ScreenVector::from_vec(vector![angle.cos() * distance, angle.sin() * distance]);
}

const RETICLE_DISTANCE_SCREEN: f32 = 20.0;

pub fn get_reticle_pos(angle: f32) -> ScreenVector {
  return distance_projection_screen(angle, RETICLE_DISTANCE_SCREEN);
}

pub struct Slot {
  pub offset: PhysicsVector,
  pub angle: f32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum SlotPosition {
  FrontAhead,
  FrontDoubleLeft,
  FrontDoubleRight,
  Front45Left,
  Front45Right,
  SideLeft,
  SideRight,
  BackAhead,
  BackDoubleLeft,
  BackDoubleRight,
  Back45Left,
  Back45Right,
}

pub type ProjectileSlots = HashMap<SlotPosition, Slot>; // 12

const SLOT_DISTANCE_PHYSICS: f32 = 1.0;

pub fn get_slot_positions(reticle_angle: f32) -> ProjectileSlots {
  let slot = |position_angle_offset: f32, shoot_direction_angle_offset: f32| {
    return Slot {
      offset: distance_projection_physics(
        reticle_angle + position_angle_offset,
        SLOT_DISTANCE_PHYSICS,
      ),
      angle: reticle_angle + shoot_direction_angle_offset,
    };
  };

  /* FRONT */

  let front_ahead = slot(0.0, 0.0);

  let front_double_left = slot(-PI / 8.0, 0.0);
  let front_double_right = slot(PI / 8.0, 0.0);

  let front_45_left = slot(-PI / 4.0, -PI / 4.0);
  let front_45_right = slot(PI / 4.0, PI / 4.0);

  /* SIDE */

  let side_left = slot(-PI / 2.0, -PI / 2.0);
  let side_right = slot(PI / 2.0, PI / 2.0);

  /* BACK */

  let back_ahead = slot(PI, PI);

  let back_double_left = slot(PI - PI / 8.0, PI);
  let back_double_right = slot(PI + PI / 8.0, PI);

  let back_45_left = slot(PI - PI / 4.0, PI - PI / 4.0);
  let back_45_right = slot(PI + PI / 4.0, PI + PI / 4.0);

  return HashMap::from([
    (SlotPosition::FrontAhead, front_ahead),
    (SlotPosition::FrontDoubleLeft, front_double_left),
    (SlotPosition::FrontDoubleRight, front_double_right),
    (SlotPosition::Front45Left, front_45_left),
    (SlotPosition::Front45Right, front_45_right),
    (SlotPosition::SideLeft, side_left),
    (SlotPosition::SideRight, side_right),
    (SlotPosition::BackAhead, back_ahead),
    (SlotPosition::BackDoubleLeft, back_double_left),
    (SlotPosition::BackDoubleRight, back_double_right),
    (SlotPosition::Back45Left, back_45_left),
    (SlotPosition::Back45Right, back_45_right),
  ]);

  /*  */
}

#[derive(Clone)]
pub struct Projectile {
  pub collider: Collider,
  pub offset: PhysicsVector,
  pub initial_force: PhysicsVector,
  pub damage: f32,
}

#[derive(Clone, Copy)]
enum ProjectileType {
  Plasma,
  Missle,
  Laser,
}

#[derive(Clone)]
pub struct Weapon {
  projectile_type: ProjectileType,
  slot_positions: HashSet<SlotPosition>,
  damage_mod: f32,
  velocity_mod: f32,
  current_cooldown: f32,
  max_cooldown: f32,
}

impl Weapon {
  pub fn reduce_cooldown(&self) -> Self {
    let current_cooldown = if self.current_cooldown > 0.0 {
      self.current_cooldown - 1.0
    } else {
      self.current_cooldown
    };

    return Self {
      projectile_type: self.projectile_type,
      slot_positions: self.slot_positions.clone(),
      damage_mod: self.damage_mod,
      velocity_mod: self.velocity_mod,
      max_cooldown: self.max_cooldown,
      current_cooldown,
    };
  }

  pub fn fire_if_ready(&self, available_slots: ProjectileSlots) -> (Self, Vec<Projectile>) {
    if self.current_cooldown > 0.0 {
      return (self.clone(), Vec::new());
    }

    let slot_positions = if self.slot_positions.len() == 0 {
      &HashSet::from([SlotPosition::FrontAhead])
    } else {
      &self.slot_positions
    };

    let mut new_weapon = self.clone();
    new_weapon.current_cooldown = new_weapon.max_cooldown;

    return (
      new_weapon,
      slot_positions
        .iter()
        .map(|slot_position| {
          let base_projectile = base_projectile_from_weapon_type(self.projectile_type);

          let slot = available_slots.get(slot_position).unwrap();

          let initial_force = distance_projection_physics(
            slot.angle,
            base_speed_from_projectile_type(self.projectile_type) * self.velocity_mod,
          );

          return Projectile {
            collider: base_projectile.collider,
            damage: base_projectile.damage * self.damage_mod,
            offset: slot.offset,
            initial_force,
          };
        })
        .collect(),
    );
  }
}

fn base_projectile_from_weapon_type(projectile_type: ProjectileType) -> Projectile {
  let collision_groups = InteractionGroups {
    memberships: COLLISION_GROUP_PLAYER_PROJECTILE,
    filter: COLLISION_GROUP_ENEMY.union(COLLISION_GROUP_WALL),
  };

  match projectile_type {
    ProjectileType::Plasma => Projectile {
      collider: ColliderBuilder::ball(0.15)
        .collision_groups(collision_groups)
        .build(),
      damage: 10.0,
      initial_force: PhysicsVector::zero(),
      offset: PhysicsVector::zero(),
    },
    ProjectileType::Missle => Projectile {
      collider: ColliderBuilder::ball(0.15)
        .collision_groups(collision_groups)
        .build(),
      damage: 10.0,
      initial_force: PhysicsVector::zero(),
      offset: PhysicsVector::zero(),
    },
    ProjectileType::Laser => Projectile {
      collider: ColliderBuilder::ball(0.15)
        .collision_groups(collision_groups)
        .build(),
      damage: 10.0,
      initial_force: PhysicsVector::zero(),
      offset: PhysicsVector::zero(),
    },
  }
}

fn base_speed_from_projectile_type(projectile_type: ProjectileType) -> f32 {
  return match projectile_type {
    ProjectileType::Plasma => 1.0,
    ProjectileType::Missle => 1.0,
    ProjectileType::Laser => 1.0,
  };
}

pub trait WeaponGenerator: Any {
  fn generate(&self) -> Weapon;
}

pub trait WeaponModulator {
  fn modulate(&self, weapon: &Weapon) -> Vec<Weapon>;
}

enum ConnectedModule {
  Generator(Rc<dyn WeaponGenerator>),
  Modulator(Rc<dyn WeaponModulator>, Rc<ConnectedModule>),
}

impl ConnectedModule {
  fn build(&self) -> Vec<Weapon> {
    return match self {
      Self::Generator(generator) => Vec::from([generator.generate()]),
      Self::Modulator(modulator, next) => next
        .build()
        .iter()
        .flat_map(|weapon| modulator.modulate(weapon))
        .collect(),
    };
  }
}

fn weapon_with_defaults(projectile_type: ProjectileType, max_cooldown: f32) -> Weapon {
  return Weapon {
    projectile_type,
    max_cooldown,
    slot_positions: HashSet::from([]),
    current_cooldown: max_cooldown,
    damage_mod: 1.0,
    velocity_mod: 1.0,
  };
}

/* WeaponComponent Implementations */

// PLSM
fn plasma() -> Weapon {
  return weapon_with_defaults(ProjectileType::Plasma, 30.0);
}

// F2SL
fn front_2_slot(weapon: &Weapon) -> Weapon {
  let mut new_weapon = weapon.clone();
  new_weapon
    .slot_positions
    .insert(SlotPosition::FrontDoubleLeft);
  new_weapon
    .slot_positions
    .insert(SlotPosition::FrontDoubleRight);
  return new_weapon;
}

// PWUP
fn double_damage(weapon: &Weapon) -> Weapon {
  let mut new_weapon = weapon.clone();
  new_weapon.damage_mod *= 2.0;
  return new_weapon;
}

pub type UnequippedModules = Vec<WeaponModuleKind>;

pub const EQUIP_SLOTS_WIDTH: i32 = 4;
pub const EQUIP_SLOTS_HEIGHT: i32 = 4;

pub type EquippedModules = Matrix<
  Option<WeaponModuleKind>,
  Const<{ EQUIP_SLOTS_HEIGHT as usize }>,
  Const<{ EQUIP_SLOTS_WIDTH as usize }>,
  ArrayStorage<
    Option<WeaponModuleKind>,
    { EQUIP_SLOTS_HEIGHT as usize },
    { EQUIP_SLOTS_WIDTH as usize },
  >,
>;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum WeaponModuleKind {
  Plasma,
  Front2Slot,
  DoubleDamage,
}

type Generator = fn() -> Weapon;
type Modulator = fn(&Weapon) -> Weapon;
type RcModulator = Rc<dyn Fn(&Weapon) -> Weapon>;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Direction {
  Up,
  Down,
  Left,
  Right,
}
use Direction::*;
use serde::Deserialize;

#[derive(Clone)]
enum WeaponModule {
  Generator(Generator),
  Modulator(Rc<Modulator>, HashSet<Direction>),
}

fn weapon_module_from_kind(kind: &WeaponModuleKind) -> WeaponModule {
  match *kind {
    WeaponModuleKind::Plasma => WeaponModule::Generator(plasma),
    WeaponModuleKind::Front2Slot => {
      WeaponModule::Modulator(Rc::new(front_2_slot), HashSet::from([Down]))
    }
    WeaponModuleKind::DoubleDamage => {
      WeaponModule::Modulator(Rc::new(double_damage), HashSet::from([Left]))
    }
  }
}

fn build_adjacent_modules(
  equipped_modules: EquippedModules,
  current_module_position: Vector2<usize>,
) -> RcModulator {
  let module_left = if current_module_position.x <= 0 {
    None
  } else {
    let equipped_modules_clone = equipped_modules.clone();
    equipped_modules.data.0[current_module_position.y][current_module_position.x - 1]
      .clone()
      .bind(weapon_module_from_kind)
      .map(|weapon_module| match weapon_module {
        WeaponModule::Generator(_) => None,
        WeaponModule::Modulator(modulator, attachment_points) => {
          if attachment_points.contains(&Right) {
            Some(Rc::new(move |weapon: &Weapon| {
              build_adjacent_modules(
                equipped_modules_clone.clone(),
                vector![current_module_position.x - 1, current_module_position.y],
              )(&modulator(weapon))
            }) as RcModulator)
          } else {
            None
          }
        }
      })
      .flatten()
  };

  let module_right = if current_module_position.x >= (EQUIP_SLOTS_WIDTH - 1) as usize {
    None
  } else {
    let equipped_modules_clone = equipped_modules.clone();
    equipped_modules.clone().data.0[current_module_position.y][current_module_position.x + 1]
      .clone()
      .bind(weapon_module_from_kind)
      .map(|weapon_module| match weapon_module {
        WeaponModule::Generator(_) => None,
        WeaponModule::Modulator(modulator, attachment_points) => {
          if attachment_points.contains(&Left) {
            Some(Rc::new(move |weapon: &Weapon| {
              build_adjacent_modules(
                equipped_modules_clone.clone(),
                vector![current_module_position.x + 1, current_module_position.y],
              )(&modulator(weapon))
            }) as RcModulator)
          } else {
            None
          }
        }
      })
      .flatten()
  };

  let module_up = if current_module_position.y <= 0 {
    None
  } else {
    let equipped_modules_clone = equipped_modules.clone();
    equipped_modules.data.0[current_module_position.y - 1][current_module_position.x]
      .clone()
      .bind(weapon_module_from_kind)
      .map(|weapon_module| match weapon_module {
        WeaponModule::Generator(_) => None,
        WeaponModule::Modulator(modulator, attachment_points) => {
          if attachment_points.contains(&Down) {
            Some(Rc::new(move |weapon: &Weapon| {
              build_adjacent_modules(
                equipped_modules_clone.clone(),
                vector![current_module_position.x, current_module_position.y - 1],
              )(&modulator(weapon))
            }) as RcModulator)
          } else {
            None
          }
        }
      })
      .flatten()
  };

  let module_down = if current_module_position.y >= (EQUIP_SLOTS_HEIGHT - 1) as usize {
    None
  } else {
    let equipped_modules_clone = equipped_modules.clone();
    equipped_modules.data.0[current_module_position.y + 1][current_module_position.x]
      .clone()
      .bind(weapon_module_from_kind)
      .map(|weapon_module| match weapon_module {
        WeaponModule::Generator(_) => None,
        WeaponModule::Modulator(modulator, attachment_points) => {
          if attachment_points.contains(&Up) {
            Some(Rc::new(move |weapon: &Weapon| {
              build_adjacent_modules(
                equipped_modules_clone.clone(),
                vector![current_module_position.x, current_module_position.y + 1],
              )(&modulator(&weapon.clone()))
            }) as RcModulator)
          } else {
            None
          }
        }
      })
      .flatten()
  };

  [module_left, module_right, module_up, module_down]
    .iter()
    .flatten()
    .cloned()
    .fold(
      Rc::new(|weapon: &Weapon| weapon.clone()) as RcModulator,
      move |acc: Rc<dyn Fn(&Weapon) -> Weapon>, modulator: Rc<dyn Fn(&Weapon) -> Weapon>| {
        Rc::new(move |weapon: &Weapon| acc(&modulator(weapon))) as Rc<dyn Fn(&Weapon) -> Weapon>
      },
    )
}

fn build_weapons(equipped_modules: EquippedModules) -> Vec<Weapon> {
  equipped_modules
    .data
    .0
    .iter()
    .enumerate()
    .flat_map(|(y, row)| {
      row
        .clone()
        .iter()
        .enumerate()
        .map(|(x, value)| {
          value.clone().bind(|weapon_module_kind| {
            match weapon_module_from_kind(weapon_module_kind) {
              WeaponModule::Modulator(_, _) => None,
              WeaponModule::Generator(generator) => Some(build_adjacent_modules(
                equipped_modules.clone(),
                vector![x, y],
              )(&generator())),
            }
          })
        })
        .collect::<Vec<_>>()
    })
    .flatten()
    .flatten()
    .collect::<Vec<_>>()
}

/*

WeaponModuleKind: enum
WeaponModule: enum(fn)

UnequippedModules: Vec<WeaponModuleKind>
EquippedModules: Matrix<WeaponModuleKind>

build: WeaponModuleKind -> WeaponModule -> Weapon



*/

/* CombatSystem */

// UnequippedModules
// EquippedModules
// Weapon
// Projectile

#[derive(Clone)]
pub struct CombatSystem {
  pub unequipped_modules: UnequippedModules,
  pub equipped_modules: EquippedModules,
  pub current_weapons: Vec<Weapon>,
  pub new_projectiles: Vec<Projectile>,
  pub acquired_items: Vec<(String, i32)>,
  pub reticle_angle: f32,
}

impl System for CombatSystem {
  fn start(_: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    /* Initialize default inventory */
    let unequipped_modules = Vec::new();

    /* Initialize default equipped weapons */
    let equipped_modules = &EquippedModules::from_data(ArrayStorage([
      [Some(WeaponModuleKind::Plasma), None, None, None],
      [None, None, Some(WeaponModuleKind::DoubleDamage), None],
      [None, None, None, Some(WeaponModuleKind::Front2Slot)],
      [None, None, None, Some(WeaponModuleKind::Plasma)],
    ]));

    return Rc::new(Self {
      unequipped_modules,
      equipped_modules: equipped_modules.clone(),
      current_weapons: build_weapons(equipped_modules.clone()),
      new_projectiles: vec![],
      reticle_angle: 0.0,
      acquired_items: vec![],
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let menu_system = ctx.get::<MenuSystem>().unwrap();

    if menu_system.active_menus.len() > 0 {
      if let Some(inventory_update) = &menu_system.inventory_update {
        return Rc::new(Self {
          unequipped_modules: inventory_update.unequipped_modules.clone(),
          equipped_modules: inventory_update.equipped_modules.clone(),
          current_weapons: build_weapons(inventory_update.equipped_modules.clone()),
          new_projectiles: Vec::new(),
          reticle_angle: self.reticle_angle,
          acquired_items: self.acquired_items.clone(),
        });
      }

      return Rc::new(self.clone());
    }

    /* Add new unequipped modules from item pickups */
    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    let unequipped_modules = self
      .unequipped_modules
      .iter()
      .chain(
        physics_system
          .new_weapon_modules
          .iter()
          .map(|(_, module)| module),
      )
      .cloned()
      .collect();

    /* Mark new item pickups as acquired */
    let map_system = ctx.get::<MapSystem>().unwrap();

    let acquired_items = self
      .acquired_items
      .iter()
      .cloned()
      .chain(
        physics_system
          .new_weapon_modules
          .iter()
          .map(|(id, _)| (map_system.current_map_name.clone(), *id)),
      )
      .collect();

    /* Decrement cooldown for active weapons */
    let reduced_cooldown_weapons: Vec<Weapon> = self
      .current_weapons
      .iter()
      .map(Weapon::reduce_cooldown)
      .collect();

    let controls_system = ctx.get::<ControlsSystem>().unwrap();

    let reticle_angle = if controls_system.right_stick.into_vec() == vector![0.0, 0.0] {
      self.reticle_angle
    } else {
      angle_from_vec(controls_system.right_stick)
    };

    let weapons_firing: Vec<(Weapon, Vec<Projectile>)> = if controls_system.firing {
      reduced_cooldown_weapons
        .iter()
        .map(|weapon| weapon.fire_if_ready(get_slot_positions(reticle_angle)))
        .collect()
    } else {
      reduced_cooldown_weapons
        .iter()
        .map(|weapon| (weapon.clone(), Vec::new()))
        .collect()
    };

    let new_weapons = weapons_firing
      .iter()
      .map(|(weapon, _)| weapon.clone())
      .collect();

    let new_projectiles = weapons_firing
      .iter()
      .flat_map(|(_, projectiles)| projectiles.clone())
      .collect();

    return Rc::new(Self {
      unequipped_modules,
      equipped_modules: self.equipped_modules.clone(),
      current_weapons: new_weapons,
      new_projectiles,
      reticle_angle,
      acquired_items,
    });
  }
}
