 use std::rc::Rc;

use rapier2d::na::Vector2;

use crate::{
  controls::ControlsSystem,
  save::SaveData,
  system::System,
  units::{PhysicsVector, UnitConvert, UnitConvert2},
};

const BOOST_MOD: f32 = 5.0;

pub struct AbilitySystem {
  pub acquired_boost: bool,
  pub boost_force: Option<Vector2<f32>>,
  pub current_boost_cooldown: f32,
  pub max_boost_cooldown: f32,
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
      acquired_boost: ctx.input.acquired_boost, // TODO: load this from save data
      boost_force: None,
      current_boost_cooldown: 60.0,
      max_boost_cooldown: 60.0,
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

    Rc::new(AbilitySystem {
      acquired_boost: self.acquired_boost,
      boost_force,
      current_boost_cooldown,
      max_boost_cooldown: self.max_boost_cooldown,
    })
  }
}
