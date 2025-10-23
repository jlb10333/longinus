use std::{any::Any, rc::Rc};

use rapier2d::prelude::RigidBodyHandle;

#[derive(Clone)]
pub struct Entity {
  pub handle: RigidBodyHandle,
  pub components: ComponentSet,
}

#[derive(Clone)]
pub struct ComponentSet {
  components: Vec<Rc<dyn Component>>,
}

impl ComponentSet {
  pub fn new() -> Self {
    return ComponentSet {
      components: Vec::new(),
    };
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

  pub fn get<Item>(&self) -> Option<Rc<Item>>
  where
    Item: Component,
  {
    return match self.components.iter().find(|component| {
      (Rc::clone(component) as Rc<dyn Any>)
        .downcast::<Item>()
        .is_ok()
    }) {
      Some(component) => Some(
        (Rc::clone(component) as Rc<dyn Any>)
          .downcast::<Item>()
          .unwrap(),
      ),
      None => None,
    };
  }
}

pub trait Component: Any {}

pub struct Damageable {
  pub health: i32,
}
impl Component for Damageable {}

pub struct Damager {
  pub damage: i32,
}
impl Component for Damager {}

pub struct DestroyOnCollision;
impl Component for DestroyOnCollision {}
