//! Model Context Protocol adapter boundary for Dreadstep.
//!
//! The in-memory [`Session`] remains the semantic adapter boundary. The [`server`] module adds
//! only a narrow local stdio process wrapper around its observation, start, and typed player-action
//! operations; it must not become a generic shell, filesystem escape hatch, or hidden source of game
//! truth.

#![forbid(unsafe_code)]

pub mod server;

pub use server::{ActParams, DreadstepMcpServer, InspectParams, StartRunParams};

use std::{error::Error, fmt};

use schemars::JsonSchema;
use serde::Serialize;

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  ReplayTrace, Tile, WorldState,
};
use dreadstep_protocol::{
  ActorId as ProtocolActorId, ActorKind as ProtocolActorKind, ActorSnapshot, CommandError,
  CommandRequest, Event, HitPoints as ProtocolHitPoints,
  ItemDefinitionId as ProtocolItemDefinitionId, ItemId as ProtocolItemId,
  Position as ProtocolPosition, ReplayEvidence, Scenario, ScenarioError, StateDigest,
  Tile as ProtocolTile, WorldError, WorldSnapshot,
};

/// Errors returned by the in-memory MCP player session.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionError {
  /// A fixed or tester-provided scenario could not be constructed.
  Scenario(ScenarioError),
  /// Core rejected the requested command.
  CommandRejected(CommandError),
  /// Core rejected a tester world mutation.
  WorldRejected(WorldError),
}

impl fmt::Display for SessionError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Scenario(error) => write!(formatter, "scenario error: {error}"),
      Self::CommandRejected(error) => write!(formatter, "command rejected: {error}"),
      Self::WorldRejected(error) => write!(formatter, "world mutation rejected: {error}"),
    }
  }
}

impl Error for SessionError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Scenario(error) => Some(error),
      Self::CommandRejected(error) => Some(error),
      Self::WorldRejected(error) => Some(error),
    }
  }
}

/// Evidence returned after one accepted player action.
#[derive(Clone, Debug, Eq, PartialEq, JsonSchema, Serialize)]
pub struct SessionOutput {
  seed: u64,
  events: Vec<Event>,
  snapshot: WorldSnapshot,
}

impl SessionOutput {
  /// Returns the session seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns protocol event evidence in execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the post-action world snapshot.
  #[must_use]
  pub const fn snapshot(&self) -> &WorldSnapshot {
    &self.snapshot
  }
}

/// An in-memory tester savepoint containing the complete deterministic session state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionSnapshot {
  seed: u64,
  world: WorldState,
  trace: ReplayTrace,
}

impl SessionSnapshot {
  /// Returns the seed captured with this savepoint.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }
}

/// A deterministic in-memory player session around one fixed developer scenario.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
  seed: u64,
  world: WorldState,
  trace: ReplayTrace,
}

impl Session {
  /// Starts a session with an explicit seed and fixed developer scenario.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::Scenario`] if the fixed map or actor setup is invalid.
  pub fn start_run(seed: u64) -> Result<Self, SessionError> {
    Ok(Self {
      seed,
      world: fixed_scenario()?,
      trace: ReplayTrace::new(seed),
    })
  }

  /// Returns the explicit session seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns a read-only protocol snapshot of the current world.
  #[must_use]
  pub fn observe(&self) -> WorldSnapshot {
    WorldSnapshot::from_world(&self.world)
  }

  /// Returns the complete protocol world snapshot for the named tester operation.
  #[must_use]
  pub fn inspect_world(&self) -> WorldSnapshot {
    self.observe()
  }

  /// Returns one protocol actor snapshot, or no value for an unknown identity.
  #[must_use]
  pub fn inspect(&self, actor: ProtocolActorId) -> Option<ActorSnapshot> {
    self
      .observe()
      .actors()
      .iter()
      .find(|snapshot| snapshot.id() == actor)
      .cloned()
  }

  /// Replaces the current world with one validated tester scenario.
  ///
  /// The explicit session seed is preserved while accepted player history and replay evidence
  /// reset to an empty trace. Invalid scenarios are rejected before either field is replaced.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::Scenario`] when core rejects the map or initial actor set.
  pub fn create_scenario(&mut self, scenario: &Scenario) -> Result<(), SessionError> {
    let tiles = scenario
      .tiles()
      .iter()
      .copied()
      .map(|tile| match tile {
        ProtocolTile::Floor => Tile::Floor,
        ProtocolTile::Wall => Tile::Wall,
      })
      .collect();
    let map = GridMap::from_tiles(scenario.width(), scenario.height(), tiles)
      .map_err(|error| SessionError::Scenario(error.into()))?;
    let actors = scenario
      .actors()
      .iter()
      .copied()
      .map(|actor| {
        let kind = match actor.kind() {
          ProtocolActorKind::Player => ActorKind::Player,
          ProtocolActorKind::Enemy => ActorKind::Enemy,
        };
        Actor::with_hit_points(
          ActorId::new(actor.id().value()),
          kind,
          Position::new(actor.position().x(), actor.position().y()),
          HitPoints::new(actor.hit_points().value()),
        )
      })
      .collect();
    let world =
      WorldState::new(map, actors).map_err(|error| SessionError::Scenario(error.into()))?;
    self.world = world;
    self.trace = ReplayTrace::new(self.seed);
    Ok(())
  }

  /// Gives one opaque item instance to an existing actor through the tester boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core rejects the target actor or duplicate
  /// item identity. The mutation does not record a player command or alter replay evidence.
  pub fn give_item(
    &mut self,
    actor: ProtocolActorId,
    item: ProtocolItemId,
    definition: ProtocolItemDefinitionId,
  ) -> Result<(), SessionError> {
    self
      .world
      .give_item(
        ActorId::new(actor.value()),
        Item::new(
          ItemId::new(item.value()),
          ItemDefinitionId::new(definition.value()),
        ),
      )
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Transfers one opaque item between existing actors through the tester boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core rejects either actor or when the source
  /// does not own the item. The mutation does not record a player command or alter replay evidence.
  pub fn transfer_item(
    &mut self,
    source_actor: ProtocolActorId,
    target_actor: ProtocolActorId,
    item: ProtocolItemId,
  ) -> Result<(), SessionError> {
    self
      .world
      .transfer_item(
        ActorId::new(source_actor.value()),
        ActorId::new(target_actor.value()),
        ItemId::new(item.value()),
      )
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Drops one opaque item at its actor's current position through the tester boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core rejects the actor or item ownership. The
  /// mutation does not record a player command or alter replay evidence.
  pub fn drop_item(
    &mut self,
    actor: ProtocolActorId,
    item: ProtocolItemId,
  ) -> Result<(), SessionError> {
    self
      .world
      .drop_item(ActorId::new(actor.value()), ItemId::new(item.value()))
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Picks one opaque item from an actor's current position through the tester boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core rejects the actor or ground ownership. The
  /// mutation does not record a player command or alter replay evidence.
  pub fn pickup_item(
    &mut self,
    actor: ProtocolActorId,
    item: ProtocolItemId,
  ) -> Result<(), SessionError> {
    self
      .world
      .pickup_item(ActorId::new(actor.value()), ItemId::new(item.value()))
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Teleports one existing actor through the tester boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core rejects the actor or destination. The
  /// mutation does not record a player command or alter replay evidence.
  pub fn teleport(
    &mut self,
    actor: ProtocolActorId,
    position: ProtocolPosition,
  ) -> Result<(), SessionError> {
    self
      .world
      .teleport(
        ActorId::new(actor.value()),
        Position::new(position.x(), position.y()),
      )
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Spawns one validated living actor through the tester operation boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core rejects the requested identity, hit
  /// points, position, terrain, or living occupancy. Rejected spawns leave the session unchanged.
  pub fn spawn(
    &mut self,
    actor: ProtocolActorId,
    kind: ProtocolActorKind,
    position: ProtocolPosition,
    hit_points: ProtocolHitPoints,
  ) -> Result<(), SessionError> {
    let actor_kind = match kind {
      ProtocolActorKind::Player => ActorKind::Player,
      ProtocolActorKind::Enemy => ActorKind::Enemy,
    };
    self
      .world
      .spawn(Actor::with_hit_points(
        ActorId::new(actor.value()),
        actor_kind,
        Position::new(position.x(), position.y()),
        HitPoints::new(hit_points.value()),
      ))
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Sets one existing actor's hit points through the tester operation boundary.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::WorldRejected`] when core cannot find the requested actor or
  /// rejects a revival due to living occupancy. The mutation does not record a player command or
  /// alter replay evidence.
  pub fn set_hp(
    &mut self,
    actor: ProtocolActorId,
    hit_points: ProtocolHitPoints,
  ) -> Result<(), SessionError> {
    self
      .world
      .set_hit_points(
        ActorId::new(actor.value()),
        HitPoints::new(hit_points.value()),
      )
      .map_err(|error| SessionError::WorldRejected(error.into()))
  }

  /// Returns protocol requests currently accepted by the core scheduler and rules.
  #[must_use]
  pub fn legal_actions(&self) -> Vec<CommandRequest> {
    self
      .world
      .legal_commands()
      .into_iter()
      .map(CommandRequest::from)
      .collect()
  }

  /// Returns accepted requests in execution order.
  #[must_use]
  pub fn history(&self) -> Vec<CommandRequest> {
    self
      .trace
      .commands()
      .iter()
      .copied()
      .map(CommandRequest::from)
      .collect()
  }

  /// Returns accepted requests in execution order using the named player operation.
  #[must_use]
  pub fn get_history(&self) -> Vec<CommandRequest> {
    self.history()
  }

  /// Captures an in-memory tester savepoint without changing the session.
  #[must_use]
  pub fn snapshot(&self) -> SessionSnapshot {
    SessionSnapshot {
      seed: self.seed,
      world: self.world.clone(),
      trace: self.trace.clone(),
    }
  }

  /// Restores the session to a previously captured in-memory tester savepoint.
  pub fn restore(&mut self, snapshot: SessionSnapshot) {
    let SessionSnapshot { seed, world, trace } = snapshot;
    self.seed = seed;
    self.world = world;
    self.trace = trace;
  }

  /// Returns the deterministic core replay digest for accepted actions.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    StateDigest::new(self.trace.digest().value())
  }

  /// Returns the explicit seed, accepted requests, and deterministic trace digest.
  #[must_use]
  pub fn get_replay(&self) -> ReplayEvidence {
    ReplayEvidence::new(self.seed, self.history(), self.replay_digest())
  }

  /// Applies one protocol request through the core and returns protocol evidence.
  ///
  /// # Errors
  ///
  /// Returns [`SessionError::CommandRejected`] when core rejects the request. Rejected commands
  /// produce no output and leave the session unchanged.
  pub fn act(&mut self, request: CommandRequest) -> Result<SessionOutput, SessionError> {
    let command = Command::from(request);
    let result = self
      .world
      .execute(command)
      .map_err(|error| SessionError::CommandRejected(error.into()))?;
    self.trace.record(command);
    Ok(SessionOutput {
      seed: self.seed,
      events: result.events().iter().copied().map(Event::from).collect(),
      snapshot: self.observe(),
    })
  }
}

fn fixed_scenario() -> Result<WorldState, SessionError> {
  let map =
    GridMap::filled(3, 1, Tile::Floor).map_err(|error| SessionError::Scenario(error.into()))?;
  WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .map_err(|error| SessionError::Scenario(error.into()))
}
