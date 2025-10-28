use std::{
  any::Any,
  array::from_fn,
  collections::{HashMap, HashSet},
  f32::consts::PI,
  rc::Rc,
};

use crate::{
  controls::{ControlsSystem, angle_from_vec},
  load_map::{COLLISION_GROUP_ENEMY, COLLISION_GROUP_PLAYER_PROJECTILE, COLLISION_GROUP_WALL},
  system::System,
  units::{PhysicsVector, ScreenVector, UnitConvert, UnitConvert2, vec_zero},
};
use rapier2d::{
  na::{ArrayStorage, Const, Matrix},
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

pub struct PlasmaWeaponGenerator; // PLSM

impl WeaponGenerator for PlasmaWeaponGenerator {
  fn generate(&self) -> Weapon {
    return weapon_with_defaults(ProjectileType::Plasma, 30.0);
  }
}

pub struct FrontTwoSlotWeaponModulator; // F2SL

impl WeaponModulator for FrontTwoSlotWeaponModulator {
  fn modulate(&self, weapon: &Weapon) -> Vec<Weapon> {
    let mut new_weapon = weapon.clone();
    new_weapon
      .slot_positions
      .insert(SlotPosition::FrontDoubleLeft);
    new_weapon
      .slot_positions
      .insert(SlotPosition::FrontDoubleRight);
    return Vec::from([new_weapon]);
  }
}

pub struct DoubleDamageWeaponModulator; // PWUP

impl WeaponModulator for DoubleDamageWeaponModulator {
  fn modulate(&self, weapon: &Weapon) -> Vec<Weapon> {
    let mut new_weapon = weapon.clone();
    new_weapon.damage_mod *= 2.0;
    return Vec::from([new_weapon]);
  }
}

pub type UnequippedModules = Vec<WeaponModule>;
pub type EquippedModules =
  Matrix<Option<WeaponModule>, Const<4>, Const<4>, ArrayStorage<Option<WeaponModule>, 4, 4>>;

/* CombatSystem */

// UnequippedModule
// EquippedModule
// ConnectedModule
// Weapon
// Projectile

#[derive(Clone)]
pub struct CombatSystem {
  pub inventory: UnequippedModules,
  pub equipped_weapons: EquippedModules,
  pub tree_weapons: Rc<ConnectedModule>, // get rid of this
  pub current_weapons: Vec<Weapon>,
  pub new_projectiles: Vec<Projectile>,
  pub reticle_angle: f32,
}

impl System for CombatSystem {
  fn start(_: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    /* Initialize default inventory */
    let inventory = Vec::new();

    /* Initialize default equipped weapons */
    let equipped_weapons = EquippedModules::from_data(ArrayStorage(from_fn(|_| from_fn(|_| None))));

    /* TODO: Get rid of */
    let tree_weapons = &Rc::new(ConnectedModule::Modulator(
      Rc::new(FrontTwoSlotWeaponModulator),
      Rc::new(ConnectedModule::Generator(Rc::new(PlasmaWeaponGenerator))),
    ));

    return Rc::new(Self {
      inventory,
      equipped_weapons,
      tree_weapons: Rc::clone(tree_weapons),
      current_weapons: tree_weapons.build(),
      new_projectiles: Vec::new(),
      reticle_angle: 0.0,
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
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
      inventory: self.inventory.clone(),
      equipped_weapons: self.equipped_weapons.clone(),
      tree_weapons: Rc::clone(&self.tree_weapons),
      current_weapons: new_weapons,
      new_projectiles,
      reticle_angle,
    });
  }
}

#[derive(Clone)]
pub enum WeaponModule {
  Generator(Rc<dyn WeaponGenerator>),
  Modulator(Rc<dyn WeaponModulator>),
}
