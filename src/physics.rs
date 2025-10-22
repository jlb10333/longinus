use rapier2d::prelude::*;
use std::{cell::RefCell, rc::Rc};

use crate::{
  combat::{CombatSystem, Projectile, get_slot_positions},
  controls::ControlsSystem,
  load_map::{COLLISION_GROUP_PLAYER, COLLISION_GROUP_WALL, MapSystem, MapTile},
  system::System,
};

// pub struct PhysicsSystem {
//   rigid_body_set: Rc<RigidBodySet>,
//   collider_set: Rc<ColliderSet>,
//   integration_parameters: Rc<IntegrationParameters>,
//   physics_pipeline: Rc<PhysicsPipeline>,
//   island_manager: Rc<IslandManager>,
//   broad_phase: Rc<DefaultBroadPhase>,
//   narrow_phase: Rc<NarrowPhase>,
//   impulse_joint_set: Rc<ImpulseJointSet>,
//   multibody_joint_set: Rc<MultibodyJointSet>,
//   ccd_solver: Rc<CCDSolver>,
// }

pub struct PhysicsSystem {
  pub rigid_body_set: RigidBodySet,
  pub collider_set: ColliderSet,
  pub integration_parameters: IntegrationParameters,
  pub physics_pipeline: Rc<RefCell<PhysicsPipeline>>,
  pub island_manager: IslandManager,
  pub broad_phase: DefaultBroadPhase,
  pub narrow_phase: NarrowPhase,
  pub impulse_joint_set: ImpulseJointSet,
  pub multibody_joint_set: MultibodyJointSet,
  pub ccd_solver: CCDSolver,
  pub player_handle: RigidBodyHandle,
}

impl System for PhysicsSystem {
  fn start(ctx: crate::system::Context) -> Rc<dyn System>
  where
    Self: Sized,
  {
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();

    /* Create the player. */
    let player_rigid_body = RigidBodyBuilder::dynamic()
      .translation(vector![0.0, 0.0])
      .build();
    let player_collider = ColliderBuilder::ball(0.25)
      .restitution(0.7)
      .collision_groups(InteractionGroups {
        memberships: COLLISION_GROUP_PLAYER,
        filter: COLLISION_GROUP_WALL,
      })
      .build();
    let player_handle = rigid_body_set.insert(player_rigid_body);
    collider_set.insert_with_parent(player_collider.clone(), player_handle, &mut rigid_body_set);

    /* Create the map colliders. */
    let map_system = ctx.get::<MapSystem>().unwrap();

    map_system.map.colliders.iter().for_each(|map_tile| {
      match map_tile {
        MapTile::Wall(wall) => collider_set.insert(wall.collider.clone()),
      };
    });

    /* Create other structures necessary for the simulation. */
    let integration_parameters = IntegrationParameters::default();
    let physics_pipeline = Rc::new(RefCell::new(PhysicsPipeline::new()));
    let island_manager = IslandManager::new();
    let broad_phase = DefaultBroadPhase::new();
    let narrow_phase = NarrowPhase::new();
    let impulse_joint_set = ImpulseJointSet::new();
    let multibody_joint_set = MultibodyJointSet::new();
    let ccd_solver = CCDSolver::new();

    return Rc::new(Self {
      rigid_body_set,
      collider_set,
      integration_parameters,
      physics_pipeline,
      island_manager,
      broad_phase,
      narrow_phase,
      impulse_joint_set,
      multibody_joint_set,
      ccd_solver,
      player_handle,
    });
  }

  fn run(&self, ctx: &crate::system::Context) -> Rc<dyn System> {
    let mut physics_pipeline = self.physics_pipeline.as_ref().borrow_mut();
    let mut island_manager = self.island_manager.clone();
    let mut broad_phase = self.broad_phase.clone();
    let mut narrow_phase = self.narrow_phase.clone();
    let mut impulse_joint_set = self.impulse_joint_set.clone();
    let mut multibody_joint_set = self.multibody_joint_set.clone();
    let mut ccd_solver = self.ccd_solver.clone();
    let mut rigid_body_set = &mut self.rigid_body_set.clone();
    let mut collider_set = self.collider_set.clone();

    /* Move the player */
    let controls_system = ctx.get::<ControlsSystem>().unwrap();

    rigid_body_set[self.player_handle].apply_impulse(*controls_system.movement_direction, true);

    /* Fire all weapons */
    let combat_system = ctx.get::<CombatSystem>().unwrap();

    let new_projectiles: Vec<Projectile> = combat_system
      .current_weapons
      .iter()
      .flat_map(|weapon| weapon.fire_if_ready(get_slot_positions(controls_system.reticle_angle)))
      .collect();

    if (new_projectiles.len() > 0) {
      println!("firing {}", new_projectiles.len());
    }

    new_projectiles.iter().for_each(|projectile| {
      let handle = rigid_body_set.insert(
        RigidBodyBuilder::dynamic()
          .translation(*rigid_body_set[self.player_handle].translation() + *projectile.offset),
      );
      rigid_body_set[handle].apply_impulse(*projectile.initial_force, true);
      collider_set.insert_with_parent(projectile.collider.clone(), handle, rigid_body_set);
    });

    /* Step physics */
    physics_pipeline.step(
      &vector![0.0, 0.0],
      &self.integration_parameters,
      &mut island_manager,
      &mut broad_phase,
      &mut narrow_phase,
      &mut rigid_body_set,
      &mut collider_set,
      &mut impulse_joint_set,
      &mut multibody_joint_set,
      &mut ccd_solver,
      &(),
      &(),
    );

    return Rc::new(Self {
      rigid_body_set: rigid_body_set.clone(),
      collider_set: collider_set,
      integration_parameters: self.integration_parameters,
      physics_pipeline: Rc::clone(&self.physics_pipeline),
      island_manager: island_manager,
      broad_phase: broad_phase,
      narrow_phase: narrow_phase,
      impulse_joint_set: impulse_joint_set,
      multibody_joint_set: multibody_joint_set,
      ccd_solver: ccd_solver,
      player_handle: self.player_handle,
    });
  }
}
