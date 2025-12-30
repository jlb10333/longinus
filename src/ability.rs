use std::rc::Rc;

use rapier2d::{na::Vector2, prelude::RigidBodyHandle};

use crate::{
  controls::ControlsSystem,
  load_map::MapAbilityType,
  physics::PhysicsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert, UnitConvert2},
};

const BOOST_MOD: f32 = 5.5;

pub struct AbilitySystem {
  pub acquired_boost: bool,
  pub acquired_chain: bool,
  pub boost_force: Option<Vector2<f32>>,
  pub current_boost_cooldown: f32,
  pub max_boost_cooldown: f32,
  pub chain_to_mount_point: Option<RigidBodyHandle>,
  pub chain_activated: bool,
  pub kill_chain: bool,
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
      current_boost_cooldown: 240.0, // TODO: Load from save data
      max_boost_cooldown: 240.0,
      chain_to_mount_point: None,
      chain_activated: false,
      kill_chain: false,
    })
  }

  fn run(
    &self,
    ctx: &crate::system::ProcessContext<Self::Input>,
  ) -> std::rc::Rc<dyn System<Input = Self::Input>> {
    let controls_system = ctx.get::<ControlsSystem<_>>().unwrap();

    let (boost_force, current_boost_cooldown) = if controls_system.boost
      && controls_system.left_stick != PhysicsVector::zero()
      && self.acquired_boost
      && self.current_boost_cooldown == 0.0
    {
      (
        Some(controls_system.left_stick.into_vec().normalize() * BOOST_MOD),
        self.max_boost_cooldown,
      )
    } else {
      (None, (self.current_boost_cooldown - 1.0).max(0.0))
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

    Rc::new(AbilitySystem {
      acquired_boost,
      acquired_chain,
      boost_force,
      current_boost_cooldown,
      max_boost_cooldown: self.max_boost_cooldown,
      chain_to_mount_point,
      chain_activated,
      kill_chain,
    })
  }
}
