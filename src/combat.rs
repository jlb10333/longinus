use std::{
  any::Any,
  collections::{HashMap, HashSet},
  f32::consts::PI,
  rc::Rc,
};

use crate::{
  system::System,
  units::{PhysicsVector, ScreenVector},
};
use rapier2d::{na::Vector2, prelude::*};

pub fn distance_projection(angle: f32, distance: f32) -> Vector2<f32> {
  return vector![angle.cos() * distance, angle.sin() * distance];
}

const RETICLE_DISTANCE_SCREEN: f32 = 20.0;

pub fn get_reticle_pos(angle: f32) -> ScreenVector {
  return ScreenVector::new(distance_projection(angle, RETICLE_DISTANCE_SCREEN));
}

pub struct Slot {
  pub offset: PhysicsVector,
  pub angle: f32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

const SLOT_DISTANCE_PHYSICS: f32 = 0.2;

pub fn get_slot_positions(reticle_angle: f32) -> ProjectileSlots {
  let slot = |position_angle_offset: f32, shoot_direction_angle_offset: f32| {
    return Slot {
      offset: PhysicsVector::new(distance_projection(
        reticle_angle + position_angle_offset,
        SLOT_DISTANCE_PHYSICS,
      )),
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

  pub fn fire_if_ready(&self, available_slots: ProjectileSlots) -> Vec<Projectile> {
    if self.current_cooldown > 0.0 {
      return Vec::new();
    }

    let slot_positions = if self.slot_positions.len() == 0 {
      &HashSet::from([SlotPosition::FrontAhead])
    } else {
      &self.slot_positions
    };

    return slot_positions
      .iter()
      .map(|slot_position| {
        let base_projectile = base_projectile_from_weapon_type(self.projectile_type);

        let slot_position = available_slots.get(slot_position).unwrap();

        let initial_force = PhysicsVector::new(distance_projection(
          slot_position.angle,
          base_speed_from_projectile_type(self.projectile_type) * self.velocity_mod,
        ));

        return Projectile {
          collider: base_projectile.collider,
          damage: base_projectile.damage * self.damage_mod,
          offset: slot_position.offset,
          initial_force,
        };
      })
      .collect();
  }
}

fn base_projectile_from_weapon_type(projectile_type: ProjectileType) -> Projectile {
  return match projectile_type {
    ProjectileType::Plasma => Projectile {
      collider: ColliderBuilder::ball(0.05).build(),
      damage: 10.0,
      initial_force: PhysicsVector::zero(),
      offset: PhysicsVector::zero(),
    },
    ProjectileType::Missle => Projectile {
      collider: ColliderBuilder::ball(0.05).build(),
      damage: 10.0,
      initial_force: PhysicsVector::zero(),
      offset: PhysicsVector::zero(),
    },
    ProjectileType::Laser => Projectile {
      collider: ColliderBuilder::ball(0.05).build(),
      damage: 10.0,
      initial_force: PhysicsVector::zero(),
      offset: PhysicsVector::zero(),
    },
  };
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

enum WeaponInventory {
  Generator(Rc<dyn WeaponGenerator>),
  Modulator(Rc<dyn WeaponModulator>, Rc<WeaponInventory>),
}

impl WeaponInventory {
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
    damage_mod: 0.0,
    velocity_mod: 0.0,
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

pub struct DoubleDamageWeaponModulator; // DDMG

impl WeaponModulator for DoubleDamageWeaponModulator {
  fn modulate(&self, weapon: &Weapon) -> Vec<Weapon> {
    let mut new_weapon = weapon.clone();
    new_weapon.damage_mod *= 2.0;
    return Vec::from([new_weapon]);
  }
}

/* CombatSystem */

#[derive(Clone)]
pub struct CombatSystem {
  pub inventory: Rc<WeaponInventory>,
  pub current_weapons: Vec<Weapon>,
}

impl System for CombatSystem {
  fn start(_: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    /* Initialize default inventory */
    let inventory = &Rc::new(WeaponInventory::Generator(Rc::new(PlasmaWeaponGenerator)));

    return Rc::new(Self {
      inventory: Rc::clone(inventory),
      current_weapons: inventory.build(),
    });
  }

  fn run(&self, _: &crate::system::Context) -> Rc<dyn System> {
    /* Decrement cooldown for active weapons */
    let new_weapons: Vec<Weapon> = self
      .current_weapons
      .iter()
      .map(Weapon::reduce_cooldown)
      .collect();

    return Rc::new(Self {
      inventory: Rc::clone(&self.inventory),
      current_weapons: new_weapons,
    });
  }
}
