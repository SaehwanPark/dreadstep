//! Core-backed terminal session. Glyphs and I/O stay outside this type.

use dreadstep_content::{ContentError, procedural_floor, starter_item_showcase_floor};
use dreadstep_core::{
  Actor, ActorId, Command, CommandError, EnemyBehavior, Event, FloorAdvanceError, FloorRecord,
  GridMap, ItemId, Position, ReplayTrace, RunOutcome, RunState, StateDigest, Tile, WorldError,
  WorldState,
};
use std::fmt;

/// The controlled player identity used by the terminal showcase.
pub const PLAYER: ActorId = ActorId::new(1);

/// Authored melee target used by display-free smoke.
pub const ATTACK_TARGET: ActorId = ActorId::new(2);

/// Authored ranged/frostcaster identity used by display-free smoke.
pub const RANGED_TARGET: ActorId = ActorId::new(3);

/// Authored reach-weapon item.
pub const EQUIP_ITEM: ItemId = ItemId::new(103);

/// Authored Frost Flask item.
pub const FROST_FLASK: ItemId = ItemId::new(104);

/// Authored healing consumable.
pub const CONSUME_ITEM: ItemId = ItemId::new(101);

/// Authored ammunition consumable used as the pickup/drop smoke item.
pub const PICKUP_ITEM: ItemId = ItemId::new(102);

/// Scenario identity preserved in journals and status lines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scenario {
  /// Authored starter-item showcase floor.
  ItemShowcase,
  /// Seeded procedural corridor floor.
  Procedural {
    /// Authored depth passed to content generation.
    depth: u32,
  },
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SessionState {
  ItemShowcase(WorldState),
  Procedural(RunState),
}

/// Errors raised while generating or validating the next procedural floor.
#[derive(Debug, Eq, PartialEq)]
pub enum SessionAdvanceError {
  /// The session is not a procedural run.
  NotProcedural,
  /// The procedural content generator rejected the requested floor.
  Content(ContentError),
  /// Core rejected the requested floor transition.
  Core(FloorAdvanceError),
}

impl fmt::Display for SessionAdvanceError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NotProcedural => write!(formatter, "session is not procedural"),
      Self::Content(error) => write!(formatter, "procedural floor content failed: {error}"),
      Self::Core(error) => write!(formatter, "core floor transition failed: {error}"),
    }
  }
}

impl std::error::Error for SessionAdvanceError {}

/// Accepted command output projected for the terminal adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOutput {
  events: Vec<Event>,
}

impl SessionOutput {
  /// Returns semantic events in execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }
}

/// A deterministic presentation adapter around core world/run state and a replay trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
  seed: u64,
  state: SessionState,
  trace: ReplayTrace,
}

impl Session {
  /// Starts the authored item showcase used by the default terminal client.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(
      seed,
      SessionState::ItemShowcase(starter_item_showcase_floor()?),
    ))
  }

  /// Starts an opt-in seeded procedural floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when generated content fails core validation.
  pub fn start_procedural_run(seed: u64, depth: u32) -> Result<Self, ContentError> {
    Ok(Self::new(
      seed,
      SessionState::Procedural(RunState::new(seed, depth, procedural_floor(seed, depth)?)),
    ))
  }

  fn new(seed: u64, state: SessionState) -> Self {
    Self {
      seed,
      state,
      trace: ReplayTrace::new(seed),
    }
  }

  /// Returns the explicit run seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns the presentation-owned scenario identity.
  #[must_use]
  pub const fn scenario(&self) -> Scenario {
    match &self.state {
      SessionState::ItemShowcase(_) => Scenario::ItemShowcase,
      SessionState::Procedural(run) => Scenario::Procedural { depth: run.depth() },
    }
  }

  /// Returns the current procedural depth, or zero for the authored item showcase.
  #[must_use]
  pub fn depth(&self) -> u32 {
    match &self.state {
      SessionState::ItemShowcase(_) => 0,
      SessionState::Procedural(run) => run.depth(),
    }
  }

  /// Returns compact core-owned history for procedural floors.
  #[must_use = "inspect the procedural floor history"]
  pub fn floor_history(&self) -> &[FloorRecord] {
    match &self.state {
      SessionState::ItemShowcase(_) => &[],
      SessionState::Procedural(run) => run.history(),
    }
  }

  /// Returns the immutable core map.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    self.world().map()
  }

  /// Returns a living or dead actor by identity.
  #[must_use]
  pub fn actor(&self, id: ActorId) -> Option<&Actor> {
    self.world().actor(id)
  }

  /// Returns actor records in stable identity order.
  pub fn actors(&self) -> impl Iterator<Item = &Actor> + '_ {
    self.world().actors()
  }

  /// Returns ground-item stacks in core order.
  #[must_use]
  pub fn ground_items(&self) -> &[dreadstep_core::GroundItemStack] {
    self.world().ground_items()
  }

  /// Returns the canonical run outcome.
  #[must_use]
  pub fn outcome(&self) -> RunOutcome {
    self.world().outcome()
  }

  /// Returns the scheduled actor, if any.
  #[must_use]
  pub fn next_actor(&self) -> Option<ActorId> {
    self.world().next_actor()
  }

  /// Returns the world's current action time.
  #[must_use]
  pub fn current_time(&self) -> dreadstep_core::ActionTime {
    self.world().current_time()
  }

  /// Returns the deterministic world digest.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    self.world().digest()
  }

  /// Returns the digest of accepted commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.trace.digest()
  }

  /// Returns accepted commands in execution order.
  #[must_use]
  pub fn replay_commands(&self) -> &[Command] {
    self.trace.commands()
  }

  /// Returns core's legal command projection.
  #[must_use]
  pub fn legal_commands(&self) -> Vec<Command> {
    self.world().legal_commands()
  }

  /// Returns core's preferred enemy command for a scheduled actor.
  #[must_use]
  pub fn preferred_enemy_command(&self, actor: ActorId, target: ActorId) -> Option<Command> {
    self.world().preferred_enemy_command(actor, target)
  }

  /// Executes one canonical command through core.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is rejected. Rejected commands do not
  /// enter the replay trace.
  pub fn execute(&mut self, command: Command) -> Result<SessionOutput, CommandError> {
    let result = match &mut self.state {
      SessionState::ItemShowcase(world) => world.execute(command),
      SessionState::Procedural(run) => run.execute(command),
    }?;
    self.trace.record(command);
    Ok(SessionOutput {
      events: result.events().to_vec(),
    })
  }

  /// Places one tile for a display-free smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::TeleportOutOfBounds`] when the cell is outside the map.
  pub fn prepare_smoke_tile(&mut self, position: Position, tile: Tile) -> Result<(), WorldError> {
    let SessionState::ItemShowcase(world) = &mut self.state else {
      return Err(WorldError::TeleportOutOfBounds {
        actor: PLAYER,
        position,
      });
    };
    world
      .set_tile(position, tile)
      .map(|_| ())
      .ok_or(WorldError::TeleportOutOfBounds {
        actor: PLAYER,
        position,
      })
  }

  /// Authors one enemy behavior for a smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the identity is missing or not an enemy.
  pub fn prepare_smoke_behavior(
    &mut self,
    actor: ActorId,
    behavior: EnemyBehavior,
  ) -> Result<(), WorldError> {
    let SessionState::ItemShowcase(world) = &mut self.state else {
      return Err(WorldError::UnknownActor(actor));
    };
    world
      .set_enemy_behavior(actor, behavior)
      .map(|_| ())
      .ok_or(WorldError::UnknownActor(actor))
  }

  /// Teleports one actor for a smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns a core [`WorldError`] when the destination is invalid.
  pub fn prepare_smoke_teleport(
    &mut self,
    actor: ActorId,
    position: Position,
  ) -> Result<(), WorldError> {
    let SessionState::ItemShowcase(world) = &mut self.state else {
      return Err(WorldError::UnknownActor(actor));
    };
    world.teleport(actor, position)
  }

  /// Drops one owned item for the pickup smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns a core [`WorldError`] when the actor or item cannot drop.
  pub fn prepare_smoke_pickup(&mut self, actor: ActorId, item: ItemId) -> Result<(), WorldError> {
    let SessionState::ItemShowcase(world) = &mut self.state else {
      return Err(WorldError::UnknownActor(actor));
    };
    world.drop_item(actor, item)
  }

  /// Generates and core-validates the next procedural floor while retaining run history.
  ///
  /// # Errors
  ///
  /// Returns [`SessionAdvanceError::NotProcedural`] for the authored showcase,
  /// [`SessionAdvanceError::Content`] when generation fails, or [`SessionAdvanceError::Core`]
  /// when the run-level transition rejects the generated world.
  pub fn advance_procedural_floor(
    &mut self,
  ) -> Result<dreadstep_core::FloorTransition, SessionAdvanceError> {
    let SessionState::Procedural(run) = &mut self.state else {
      return Err(SessionAdvanceError::NotProcedural);
    };
    let next_depth = run.depth().checked_add(1).ok_or(SessionAdvanceError::Core(
      FloorAdvanceError::DepthOverflow { depth: run.depth() },
    ))?;
    let next_world =
      procedural_floor(self.seed, next_depth).map_err(SessionAdvanceError::Content)?;
    let transition = run
      .advance(next_depth, next_world)
      .map_err(SessionAdvanceError::Core)?;
    self.trace = ReplayTrace::new(self.seed);
    Ok(transition)
  }

  const fn world(&self) -> &WorldState {
    match &self.state {
      SessionState::ItemShowcase(world) => world,
      SessionState::Procedural(run) => run.world(),
    }
  }

  #[cfg(test)]
  pub(crate) fn set_hit_points_for_test(
    &mut self,
    actor: ActorId,
    hit_points: dreadstep_core::HitPoints,
  ) -> Result<(), WorldError> {
    let SessionState::ItemShowcase(world) = &mut self.state else {
      return Err(WorldError::UnknownActor(actor));
    };
    world.set_hit_points(actor, hit_points)
  }

  #[cfg(test)]
  pub(crate) fn from_procedural_world_for_test(seed: u64, depth: u32, world: WorldState) -> Self {
    Self::new(
      seed,
      SessionState::Procedural(RunState::new(seed, depth, world)),
    )
  }
}

#[cfg(test)]
mod tests {
  use super::{PLAYER, Scenario, Session, SessionAdvanceError, SessionState};
  use dreadstep_content::procedural_floor;
  use dreadstep_core::{Command, HitPoints, RunOutcome, RunState};

  #[test]
  fn item_run_records_accepted_commands_in_replay() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let wait = Command::Wait { actor: PLAYER };
    session.execute(wait).expect("wait is legal at start");
    assert_eq!(session.replay_commands(), &[wait]);
    assert_ne!(session.replay_digest().value(), 0);
  }

  #[test]
  fn rejected_commands_do_not_enter_replay() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let illegal = Command::Unequip { actor: PLAYER };
    assert!(session.execute(illegal).is_err());
    assert!(session.replay_commands().is_empty());
  }

  #[test]
  fn procedural_session_projects_core_depth_and_initial_history() {
    let session = Session::start_procedural_run(7, 1).expect("procedural floor");

    assert_eq!(session.scenario(), Scenario::Procedural { depth: 1 });
    assert_eq!(session.depth(), 1);
    assert_eq!(session.floor_history().len(), 1);
    assert_eq!(session.floor_history()[0].depth(), 1);
  }

  #[test]
  fn procedural_session_delegates_unwon_advance_to_core() {
    let mut session = Session::start_procedural_run(7, 1).expect("procedural floor");
    let before = session.clone();

    assert!(matches!(
      session.advance_procedural_floor(),
      Err(SessionAdvanceError::Core(
        dreadstep_core::FloorAdvanceError::NotVictorious { .. }
      ))
    ));
    assert_eq!(session, before);
  }

  #[test]
  fn procedural_session_keeps_core_history_and_resets_replay_after_advance() {
    let mut world = procedural_floor(7, 1).expect("procedural floor");
    let enemy_ids = world
      .actors()
      .filter(|actor| actor.kind() == dreadstep_core::ActorKind::Enemy)
      .map(dreadstep_core::Actor::id)
      .collect::<Vec<_>>();
    for enemy_id in enemy_ids {
      world
        .set_hit_points(enemy_id, HitPoints::new(0))
        .expect("tester mutation should defeat generated enemy");
    }
    let mut session = Session::new(7, SessionState::Procedural(RunState::new(7, 1, world)));
    assert_eq!(session.outcome(), RunOutcome::Victory);

    session
      .execute(Command::Wait { actor: PLAYER })
      .expect("the session still delegates commands to core after victory");
    let completed_digest = session.digest();

    session
      .advance_procedural_floor()
      .expect("victory should advance the generated floor");
    assert_eq!(session.depth(), 2);
    assert_eq!(session.floor_history().len(), 2);
    assert!(session.replay_commands().is_empty());
    assert_eq!(session.floor_history()[0].outcome(), RunOutcome::Victory);
    assert_eq!(session.floor_history()[0].state_digest(), completed_digest);
    assert_eq!(session.floor_history()[1].depth(), 2);
  }
}
