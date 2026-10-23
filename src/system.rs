use std::{
  any::Any,
  marker::PhantomData,
  rc::Rc,
  thread::sleep,
  time::{Duration, Instant},
};

use frunk::{
  HCons, HList,
  hlist::{self, *},
  indices::Here,
};
use macroquad::window::next_frame;

pub trait System: Any {
  type Input: Clone + 'static;
  type Deps: HList = HNil;
  type LDeps: HList = HNil;

  fn start<ProcessSystems: Clone + HList + Sculptor<Self::Deps, Indices> + 'static, Indices>(
    ctx: Rc<ProcessContext<Self::Input, ProcessSystems>>,
  ) -> Rc<Self>
  where
    Self: Sized;

  fn update<
    ProcessSystems: Clone + HList + Sculptor<Self::Deps, Indices> + Sculptor<Self::LDeps, LIndices> + 'static,
    Indices,
    LIndices,
  >(
    self: Rc<Self>,
    _ctx: Rc<ProcessContext<Self::Input, ProcessSystems>>,
  ) -> Rc<Self> {
    self
  }

  fn fixed_update<
    ProcessSystems: Clone + HList + Sculptor<Self::Deps, Indices> + Sculptor<Self::LDeps, LIndices> + 'static,
    Indices,
    LIndices,
  >(
    self: Rc<Self>,
    _ctx: Rc<ProcessContext<Self::Input, ProcessSystems>>,
  ) -> Rc<Self> {
    self
  }
}

#[derive(Clone)]
pub struct ProcessContextOptions {
  /// Interval to target for fixed_update calls, in seconds
  pub fixed_time_interval: f32,
  pub target_fps: i32,
}

impl Default for ProcessContextOptions {
  fn default() -> Self {
    Self {
      fixed_time_interval: 1.0 / 60.0,
      target_fps: 60,
    }
  }
}

trait RunSystemHList<Input: Clone + 'static, Indices, LIndices, Systems: HList + 'static>:
  Sized
{
  fn update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self;

  fn fixed_update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self;
}

impl<Input: Clone + 'static, Systems: HList + 'static> RunSystemHList<Input, HNil, HNil, Systems>
  for HNil
{
  fn update(self, _: Rc<ProcessContext<Input, Systems>>) -> Self {
    self
  }

  fn fixed_update(self, _: Rc<ProcessContext<Input, Systems>>) -> Self {
    self
  }
}

impl<
  Input: Clone + 'static,
  Deps: HList,
  LDeps: HList,
  FirstIndices,
  RestIndices,
  FirstLIndices,
  RestLIndices,
  Systems: HList + Sculptor<Deps, FirstIndices> + Sculptor<LDeps, FirstLIndices> + Clone + 'static,
  H: System<Input = Input, Deps = Deps, LDeps = LDeps>,
  T: HList + RunSystemHList<Input, RestIndices, RestLIndices, Systems>,
> RunSystemHList<Input, (FirstIndices, RestIndices), (FirstLIndices, RestLIndices), Systems>
  for HCons<Rc<H>, T>
{
  fn update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self {
    let (h, t) = self.pop();

    let new_h = h.update(ctx.clone());

    let new_t = t.update(ctx);

    h_cons(new_h, new_t)
  }

  fn fixed_update(self, ctx: Rc<ProcessContext<Input, Systems>>) -> Self {
    let (h, t) = self.pop();

    let new_h = h.fixed_update(ctx.clone());

    let new_t = t.fixed_update(ctx);

    h_cons(new_h, new_t)
  }
}

pub struct ProcessContext<Input: Clone + 'static, Systems: HList> {
  pub systems: Systems,
  pub input: Input,
  options: ProcessContextOptions,
}

impl<Input: Clone + 'static, Systems: HList + Clone> Clone for ProcessContext<Input, Systems> {
  fn clone(&self) -> Self {
    Self {
      systems: self.systems.clone(),
      input: self.input.clone(),
      options: self.options.clone(),
    }
  }
}

impl<Input: Clone + 'static, Systems: HList + 'static> ProcessContext<Input, Systems> {
  pub fn downcast<Target: Clone + 'static>(&self) -> Option<&ProcessContext<Target, Systems>> {
    (self as &dyn Any).downcast_ref::<ProcessContext<Target, Systems>>()
  }

  pub fn get_deps<'a, Deps: HList, Index>(self: &Rc<Self>) -> Deps
  where
    Systems: Sculptor<Deps, Index> + Clone,
  {
    self.systems.clone().sculpt().0
  }

  pub fn get<'a, SingleDep, Index>(self: &Rc<Self>) -> Rc<SingleDep>
  where
    Systems: Plucker<Rc<SingleDep>, Index> + Clone,
  {
    self.systems.clone().pluck().0
  }

  pub fn add_system<NewSystem: System<Input = Input>, Indices>(
    self: Rc<Self>,
    new_system_initializer: fn(Rc<Self>) -> Rc<NewSystem>,
  ) -> Rc<ProcessContext<Input, HCons<Rc<NewSystem>, Systems>>>
  where
    Systems: Sculptor<NewSystem::Deps, Indices> + Clone,
  {
    let new_system = new_system_initializer(self.clone());

    ProcessContext::<Input, HCons<Rc<NewSystem>, Systems>> {
      input: self.input.clone(),
      systems: h_cons(new_system, self.systems.clone()),
      options: self.options.clone(),
    }
    .into()
  }

  fn with_systems(&self, new_systems: Systems) -> Self {
    Self {
      systems: new_systems,
      input: self.input.clone(),
      options: self.options.clone(),
    }
  }

  pub async fn run<'a, Output, Terminator, ShouldFreeze, Indices, LIndices>(
    self: Rc<Self>,
    terminator: Terminator,
    should_freeze_fixed: ShouldFreeze,
  ) -> Output
  where
    Terminator: Fn(Rc<ProcessContext<Input, Systems>>) -> Option<Output>,
    ShouldFreeze: Fn(Rc<ProcessContext<Input, Systems>>) -> bool,
    Systems: RunSystemHList<Input, Indices, LIndices, Systems> + Clone,
  {
    let mut game_state = self.clone();
    let mut acc_time = 0.0;

    loop {
      let result = terminator(game_state.clone());

      if let Some(output) = result {
        return output;
      }

      let now = Instant::now();

      let new_systems = game_state.systems.clone().update(game_state.clone());
      game_state = game_state.with_systems(new_systems.into()).into();

      next_frame().await;

      let skipping_fixed = should_freeze_fixed(game_state.clone());

      if !skipping_fixed {
        let fixed_update_count =
          (acc_time / (game_state.options.fixed_time_interval * 1_000_000.0)).floor() as i32;

        acc_time -=
          game_state.options.fixed_time_interval * 1_000_000.0 * fixed_update_count as f32;

        let new_systems = game_state.systems.clone().fixed_update(game_state.clone());
        game_state = game_state.with_systems(new_systems.into()).into();
      }

      let frame_time = now.elapsed().as_millis() as f32;

      let min_frame_time = 1000.0 / game_state.options.target_fps as f32;

      if frame_time < min_frame_time {
        let time_to_sleep = min_frame_time - frame_time;

        sleep(Duration::from_millis(time_to_sleep as u64));
      }

      if !skipping_fixed {
        acc_time += now.elapsed().as_micros() as f32;
      }
    }
  }
}

pub fn new_process<Input: Clone + 'static>(
  input: &Input,
  options: Option<ProcessContextOptions>,
) -> Rc<ProcessContext<Input, HNil>> {
  ProcessContext::<Input, HNil> {
    input: input.clone(),
    systems: HNil,
    options: options.unwrap_or_default(),
  }
  .into()
}

trait HConsMarker {}
impl<H, T> HConsMarker for HCons<H, T> {}

pub trait GetDeps<Input: Clone + 'static>: HList {
  type Output;

  fn get_deps<Indices, Systems: HList + Sculptor<Self, Indices> + Clone + 'static>(
    ctx: &Rc<ProcessContext<Input, Systems>>,
  ) -> Self::Output
  where
    Self: Sized;
}

impl<Input: Clone + 'static, H> GetDeps<Input> for HCons<H, HNil> {
  type Output = H;

  fn get_deps<Indices, Systems: HList + Sculptor<Self, Indices> + Clone + 'static>(
    ctx: &Rc<ProcessContext<Input, Systems>>,
  ) -> Self::Output
  where
    Self: Sized,
  {
    ctx.get_deps::<Self, Indices>().pop().0
  }
}

impl<Input: Clone + 'static, H, T: HList + HConsMarker> GetDeps<Input> for HCons<H, T>
where
  Self: IntoTuple2,
{
  type Output = (
    <Self as IntoTuple2>::HeadType,
    <Self as IntoTuple2>::TailOutput,
  );

  fn get_deps<Indices, Systems: HList + Sculptor<Self, Indices> + Clone + 'static>(
    ctx: &Rc<ProcessContext<Input, Systems>>,
  ) -> Self::Output
  where
    Self: Sized,
  {
    ctx.get_deps().into_tuple2()
  }
}

#[macro_export]
macro_rules! SystemDeps {
    ($($type:ty),+) => {
        frunk::HList!($(std::rc::Rc<$type>),+)
    };
}

#[macro_export]
macro_rules! system_deps {
  ($x:ident) => {
    $x
  };
  ($x:ident, $($rest:ident),+) => {
    ($x, system_deps!($($rest),+))
  };
}
