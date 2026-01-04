use std::rc::Rc;

use rapier2d::{na::Vector2, prelude::RigidBodyHandle};
use serde::{Deserialize, Serialize};

use crate::{
  controls::ControlsSystem,
  load_map::MapAbilityType,
  menu::MenuSystem,
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert, UnitConvert2},
};

const MANA_TANK_CAPACITY: f32 = 3.0;
const MANA_TANK_RECHARGE_RATE: f32 = 1.0 / 60.0;
const BOOST_MOD: f32 = 5.5;
const BOOST_MANA_USE: f32 = 3.0;
const BOOST_MAX_COOLDOWN: f32 = 10.0;

#[derive(Serialize, Deserialize, Clone, Copy, Default)]
pub struct ManaTanksCapacityInfo {
  pub auto_recharge_mana_tanks: i32,
  pub non_recharge_mana_tanks: i32,
}

impl ManaTanksCapacityInfo {
  pub fn max_non_rechargeable_mana_level(&self) -> f32 {
    self.non_recharge_mana_tanks as f32 * MANA_TANK_CAPACITY
  }

  pub fn max_rechargeable_mana_level(&self) -> f32 {
    self.auto_recharge_mana_tanks as f32 * MANA_TANK_CAPACITY
  }
}

#[derive(Clone, Copy)]
pub struct ManaTanksActiveInfo {
  pub rechargeable_mana_level: f32,
  pub non_rechargeable_mana_level: f32,
  pub capacity: ManaTanksCapacityInfo,
}

impl ManaTanksActiveInfo {
  pub fn recharge(&self) -> Self {
    Self {
      rechargeable_mana_level: (self.rechargeable_mana_level + MANA_TANK_RECHARGE_RATE)
        .min(self.capacity.max_non_rechargeable_mana_level()),
      ..*self
    }
  }

  pub fn with(&self, amount: f32) -> Self {
    let attempted_non_recharge_level = self.non_rechargeable_mana_level + amount;
    let rechargeable_difference =
      (attempted_non_recharge_level - self.capacity.max_non_rechargeable_mana_level()).max(0.0);

    Self {
      non_rechargeable_mana_level: attempted_non_recharge_level
        .min(self.capacity.max_non_rechargeable_mana_level()),
      rechargeable_mana_level: (self.rechargeable_mana_level + rechargeable_difference)
        .min(self.capacity.max_rechargeable_mana_level()),
      ..*self
    }
  }

  pub fn total_mana_level(&self) -> f32 {
    self.non_rechargeable_mana_level + self.rechargeable_mana_level
  }

  pub fn without(&self, amount: f32) -> Option<Self> {
    if amount > self.total_mana_level() {
      None
    } else {
      let attempted_rechargeable_level = self.rechargeable_mana_level - amount;
      let non_rechargeable_difference = attempted_rechargeable_level.min(0.0);

      Some(Self {
        rechargeable_mana_level: attempted_rechargeable_level.max(0.0),
        non_rechargeable_mana_level: self.non_rechargeable_mana_level + non_rechargeable_difference,
        ..*self
      })
    }
  }
}

pub struct AbilitySystem {
  pub acquired_boost: bool,
  pub acquired_chain: bool,
  pub boost_force: Option<Vector2<f32>>,
  pub current_boost_cooldown: f32,
  pub max_boost_cooldown: f32,
  pub chain_to_mount_point: Option<RigidBodyHandle>,
  pub chain_activated: bool,
  pub kill_chain: bool,
  pub mana_tanks: ManaTanksActiveInfo,
}

impl System for AbilitySystem {
  type Input = SaveData;

  fn start(
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>>
  where
    Self: Sized,
  {
    Rc::new(AbilitySystem {
      acquired_boost: ctx.input.acquired_boost,
      acquired_chain: ctx.input.acquired_chain,
      boost_force: None,
      current_boost_cooldown: BOOST_MAX_COOLDOWN,
      max_boost_cooldown: BOOST_MAX_COOLDOWN,
      chain_to_mount_point: None,
      chain_activated: false,
      kill_chain: false,
      mana_tanks: ManaTanksActiveInfo {
        rechargeable_mana_level: ctx.input.mana_tanks_capacity.max_rechargeable_mana_level(),
        non_rechargeable_mana_level: ctx
          .input
          .mana_tanks_capacity
          .max_non_rechargeable_mana_level(),
        capacity: ctx.input.mana_tanks_capacity,
      },
    })
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>> {
    let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

    let (boost_force, current_boost_cooldown, mana_tanks) = if controls_system.boost
      && !controls_system
        .last_frame
        .as_ref()
        .map(|last_frame| last_frame.boost)
        .unwrap_or(false)
      && controls_system.left_stick != PhysicsVector::zero()
      && self.acquired_boost
      && self.current_boost_cooldown == 0.0
      && let Some(mana_tanks) = self.mana_tanks.without(BOOST_MANA_USE)
    {
      (
        Some(controls_system.left_stick.into_vec().normalize() * BOOST_MOD),
        self.max_boost_cooldown,
        mana_tanks,
      )
    } else {
      (
        None,
        (self.current_boost_cooldown - 1.0).max(0.0),
        self.mana_tanks,
      )
    };

    let physics_system = ctx.get::<PhysicsSystem>().unwrap();

    let acquired_boost = self.acquired_boost
      || physics_system
        .new_abilities
        .iter()
        .any(|new_ability| matches!(new_ability, MapAbilityType::Boost));

    let acquired_chain = self.acquired_chain
      || physics_system
        .new_abilities
        .iter()
        .any(|new_ability| matches!(new_ability, MapAbilityType::Chain));

    let kill_chain = self.chain_activated
      && controls_system.chain
      && !controls_system.last_frame.as_ref().unwrap().chain;

    let chain_to_mount_point = if self.acquired_chain
      && !self.chain_activated
      && controls_system.chain
      && !controls_system.last_frame.as_ref().unwrap().chain
    {
      physics_system
        .mount_points_in_range
        .iter()
        .reduce(|mount_a, mount_b| {
          let mount_a_translation = physics_system.rigid_body_set[*mount_a].translation();
          let mount_b_translation = physics_system.rigid_body_set[*mount_b].translation();

          let player = physics_system.rigid_body_set[physics_system.player_handle].translation();

          let distance_a = (mount_a_translation - player).magnitude();

          let distance_b = (mount_b_translation - player).magnitude();

          if distance_a < distance_b {
            mount_a
          } else {
            mount_b
          }
        })
        .cloned()
    } else {
      None
    };

    let chain_activated = (self.chain_activated || chain_to_mount_point.is_some()) && !kill_chain;

    let menu_system = ctx.get::<MenuSystem<_>>().unwrap();
    let mana_tanks = if menu_system.active_menus.is_empty() {
      let mana_tanks = mana_tanks.recharge();
      mana_tanks.with(physics_system.incoming_mana)
    } else {
      mana_tanks
    };
    let mana_tanks = mana_tanks.recharge();

    Rc::new(AbilitySystem {
      acquired_boost,
      acquired_chain,
      boost_force,
      current_boost_cooldown,
      max_boost_cooldown: self.max_boost_cooldown,
      chain_to_mount_point,
      chain_activated,
      kill_chain,
      mana_tanks,
    })
  }
}
