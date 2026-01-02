use std::{any::Any, rc::Rc};

use rapier2d::{
  na::Vector2,
  prelude::{
    ColliderHandle, ColliderSet, ImpulseJointHandle, InteractionGroups, NarrowPhase,
    RigidBodyHandle, RigidBodySet,
  },
};
use rpds::{HashTrieSet, List};

use crate::{
  combat::WeaponModuleKind,
  enemy::{EnemyDefender, EnemyGoblin, EnemyGoblinState, EnemySeeker, EnemySeekerGenerator},
  load_map::{MapAbilityType, MapEnemyName},
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
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

  pub fn intersecting_with_colliders(
    &self,
    rigid_body_set: &RigidBodySet,
    narrow_phase: &NarrowPhase,
  ) -> List<ColliderHandle> {
    self
      .colliders(rigid_body_set)
      .iter()
      .flat_map(|&&collider_handle| {
        narrow_phase
          .contact_pairs_with(collider_handle)
          .flat_map(move |contact_pair| {
            if contact_pair.has_any_active_contact {
              [contact_pair.collider1, contact_pair.collider2]
                .into_iter()
                .filter(|&other_handle| other_handle != collider_handle)
                .collect::<Vec<_>>()
            } else {
              vec![]
            }
          })
          .chain(
            narrow_phase
              .intersection_pairs_with(collider_handle)
              .flat_map(move |(collider1, collider2, colliding)| {
                if !colliding {
                  return vec![];
                }
                [collider1, collider2]
                  .into_iter()
                  .filter(|&other_handle| other_handle != collider_handle)
                  .collect::<Vec<_>>()
              }),
          )
      })
      .collect::<List<_>>()
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
  /* Dragonspawn */
  Goblin(EnemyGoblin),
  /* Angelic Constructs */
  Defender(EnemyDefender),
  Seeker(EnemySeeker),
  SeekerGenerator(EnemySeekerGenerator),
}
impl Enemy {
  pub fn default_from_map(map_enemy: MapEnemyName) -> Enemy {
    match map_enemy {
      MapEnemyName::Goblin => Self::Goblin(EnemyGoblin {
        state: EnemyGoblinState::initial(),
      }),
      MapEnemyName::Defender => Self::Defender(EnemyDefender { cooldown: 0 }),
      MapEnemyName::Seeker => Self::Seeker(EnemySeeker),
      MapEnemyName::SeekerGenerator => Self::SeekerGenerator(EnemySeekerGenerator { cooldown: 0 }),
    }
  }
}
impl Component for Enemy {}

pub struct GivesItemOnCollision {
  pub weapon_module_kind: WeaponModuleKind,
}
impl Component for GivesItemOnCollision {}

pub struct MapTransitionOnCollision {
  pub map_name: String,
  pub target_player_spawn_id: i32,
}
impl Component for MapTransitionOnCollision {}

pub struct SaveMenuOnCollision;
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

pub struct TouchSensor {
  pub target_activation: f32,
}
impl Component for TouchSensor {}

pub struct GravitySource {
  pub strength: f32,
  pub activator_id: Option<i32>,
}
impl Component for GravitySource {}

pub struct Destroyed;
impl Component for Destroyed {}

pub struct GiveAbilityOnCollision {
  pub ability_type: MapAbilityType,
}
impl Component for GiveAbilityOnCollision {}

pub struct ChainMountArea {
  pub target_mount_body: RigidBodyHandle,
}
impl Component for ChainMountArea {}

pub struct Switch {
  pub joint: ImpulseJointHandle,
}
impl Component for Switch {}

pub struct Locomotor {
  pub joint: ImpulseJointHandle,
  pub reverse_direction: bool,
}
impl Component for Locomotor {}

pub struct ChainSegment;
impl Component for ChainSegment {}

pub struct SimpleActivatable {
  pub activation: f32,
  pub activator_id: i32,
}
impl Component for SimpleActivatable {}

pub struct Activator {
  pub activation: f32,
}
impl Component for Activator {}

pub struct ExplodeOnCollision {
  pub strength: f32,
  pub radius: f32,
  pub damage: f32,
  pub interaction_groups: InteractionGroups,
}
impl Component for ExplodeOnCollision {}

pub struct DestroyAfterFrames {
  pub frames: i32,
}
impl Component for DestroyAfterFrames {}

pub struct And {
  pub activator_ids: (i32, i32),
}
impl Component for And {}

pub struct Or {
  pub activator_ids: (i32, i32),
}
impl Component for Or {}

pub struct Gate {
  pub activator_id: i32,
  pub highest_historical_activation: f32,
}
impl Component for Gate {}

pub struct Engine {
  pub activator_id: Option<i32>,
  pub currently_increasing: bool,
}
impl Component for Engine {}

pub struct Terminal {
  pub content: String,
  pub created_at: String,
}
impl Component for Terminal {}

pub struct Id {
  pub id: i32,
}
impl Component for Id {}
