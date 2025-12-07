use std::{any::Any, rc::Rc, time::Instant};

use rapier2d::{
  na::Vector2,
  prelude::{ColliderHandle, ColliderSet, RigidBodyHandle, RigidBodySet},
};

use crate::{
  combat::WeaponModuleKind,
  enemy::{EnemyDefender, EnemySeeker, EnemySeekerGenerator},
  load_map::{MapAbilityType, MapEnemyName, MapGateState},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityHandle {
  RigidBody(RigidBodyHandle),
  Collider(ColliderHandle),
}

impl EntityHandle {
  pub fn colliders<'a>(&'a self, rigid_body_set: &'a RigidBodySet) -> Vec<&'a ColliderHandle> {
    match self {
      EntityHandle::Collider(collider_handle) => vec![collider_handle],
      EntityHandle::RigidBody(rigid_body_handle) => rigid_body_set[*rigid_body_handle]
        .colliders()
        .iter()
        .collect(),
    }
  }

  pub fn translation<'a>(
    &'a self,
    rigid_body_set: &'a RigidBodySet,
    collider_set: &'a ColliderSet,
  ) -> &'a Vector2<f32> {
    match self {
      EntityHandle::Collider(collider_handle) => collider_set[*collider_handle].translation(),
      EntityHandle::RigidBody(rigid_body_handle) => {
        rigid_body_set[*rigid_body_handle].translation()
      }
    }
  }
}

#[derive(Clone)]
pub struct Entity {
  pub handle: EntityHandle,
  pub components: ComponentSet,
  pub label: String,
}

#[derive(Clone)]
pub struct ComponentSet {
  components: Vec<Rc<dyn Component>>,
}

impl ComponentSet {
  pub fn new() -> Self {
    ComponentSet {
      components: Vec::new(),
    }
  }

  pub fn insert<Item>(&self, item: Item) -> Self
  where
    Item: Component,
  {
    if self.components.iter().any(|component| {
      (Rc::clone(component) as Rc<dyn Any>)
        .downcast::<Item>()
        .is_ok()
    }) {
      return self.clone();
    }
    return Self {
      components: self
        .components
        .iter()
        .cloned()
        .chain([Rc::new(item) as Rc<dyn Component>])
        .collect(),
    };
  }

  pub fn with<Item>(&self, item: Item) -> Self
  where
    Item: Component,
  {
    let components: Vec<_> = self
      .components
      .iter()
      .cloned()
      .filter(|component| {
        (Rc::clone(component) as Rc<dyn Any>)
          .downcast::<Item>()
          .is_err()
      })
      .collect();

    return Self { components }.insert(item);
  }

  pub fn get<Item>(&self) -> Option<Rc<Item>>
  where
    Item: Component,
  {
    self
      .components
      .iter()
      .find(|component| {
        (Rc::clone(component) as Rc<dyn Any>)
          .downcast::<Item>()
          .is_ok()
      })
      .and_then(|component| {
        (Rc::clone(component) as Rc<dyn Any>)
          .downcast::<Item>()
          .ok()
      })
  }
}

pub trait Component: Any {}

pub struct Damageable {
  pub health: f32,
  pub max_health: f32,
  pub destroy_on_zero_health: bool,
  pub current_hitstun: f32,
  pub max_hitstun: f32,
}
impl Component for Damageable {}

pub struct Damager {
  pub damage: f32,
}
impl Component for Damager {}

pub struct DestroyOnCollision;
impl Component for DestroyOnCollision {}

#[derive(Clone)]
pub enum Enemy {
  Defender(EnemyDefender),
  Seeker(EnemySeeker),
  SeekerGenerator(EnemySeekerGenerator),
}
impl Enemy {
  pub fn default_from_map(map_enemy: MapEnemyName) -> Enemy {
    match map_enemy {
      MapEnemyName::Defender => Self::Defender(EnemyDefender { cooldown: 0 }),
      MapEnemyName::Seeker => Self::Seeker(EnemySeeker),
      MapEnemyName::SeekerGenerator => Self::SeekerGenerator(EnemySeekerGenerator { cooldown: 0 }),
    }
  }
}
impl Component for Enemy {}

pub struct GivesItemOnCollision {
  pub id: i32,
  pub weapon_module_kind: WeaponModuleKind,
}
impl Component for GivesItemOnCollision {}

pub struct MapTransitionOnCollision {
  pub map_name: String,
  pub target_player_spawn_id: i32,
}
impl Component for MapTransitionOnCollision {}

pub struct SaveMenuOnCollision {
  pub id: i32,
}
impl Component for SaveMenuOnCollision {}

pub struct DropHealthOnDestroy {
  pub amount: f32,
  pub chance: f32,
}
impl Component for DropHealthOnDestroy {}

pub struct HealOnCollision {
  pub amount: f32,
}
impl Component for HealOnCollision {}

pub struct Gate {
  pub id: i32,
}
impl Component for Gate {}

pub struct GateTrigger {
  pub gate_id: i32,
  pub action: MapGateState,
}
impl Component for GateTrigger {}

pub struct GravitySource {
  pub strength: f32,
}
impl Component for GravitySource {}

pub struct Destroyed;
impl Component for Destroyed {}

pub struct GiveAbilityOnCollision {
  pub ability_type: MapAbilityType,
}
impl Component for GiveAbilityOnCollision {}

pub struct ChainMountActivation {
  pub target_mount_body: RigidBodyHandle,
}
impl Component for ChainMountActivation {}

pub struct Switch {
  pub activation: f32,
}
impl Component for Switch {}

pub struct ChainSegment;
impl Component for ChainSegment {}
