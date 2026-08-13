//! Core-backed presentation state and the Bevy runtime resource.

use bevy::ecs::resource::Resource;
use dreadstep_content::{
  ContentError, chill_trap_floor, procedural_floor, starter_floor, starter_item_showcase_floor,
};
use dreadstep_core::{
  ActorId, Command, CommandError, EnemyBehavior, GridMap, ItemId, Position, ReplayTrace,
  StateDigest, Tile, WorldState,
};

use crate::{PresentationOutput, PresentationSnapshot};

/// A deterministic presentation adapter around one core world and replay trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationState {
  pub(crate) seed: u64,
  pub(crate) world: WorldState,
  pub(crate) trace: ReplayTrace,
}

impl PresentationState {
  /// Starts a presentation state from the shared authored starter floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core validation.
  pub fn start_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, starter_floor()?))
  }

  /// Starts a presentation state from the shared authored starter-item scenario.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored item floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, starter_item_showcase_floor()?))
  }

  /// Starts the authored chilled-status showcase floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] if the authored floor fails validation.
  pub fn start_chill_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, chill_trap_floor()?))
  }

  /// Starts a presentation state from a deterministic seeded procedural floor.
  ///
  /// The content boundary owns generation and core still validates the resulting world. This
  /// constructor is opt-in; the stable authored starter floor remains the default startup.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when generated content fails core validation.
  pub fn start_procedural_run(seed: u64, depth: u32) -> Result<Self, ContentError> {
    Ok(Self::new(seed, procedural_floor(seed, depth)?))
  }

  /// Creates a presentation state around an already validated core world.
  #[must_use]
  pub fn new(seed: u64, world: WorldState) -> Self {
    Self {
      seed,
      world,
      trace: ReplayTrace::new(seed),
    }
  }

  /// Returns the explicit run seed preserved by this presentation boundary.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns a stable read-only projection of the current core world.
  #[must_use]
  pub fn snapshot(&self) -> PresentationSnapshot {
    PresentationSnapshot::from_world(&self.world)
  }

  /// Returns the deterministic digest of accepted presentation commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.trace.digest()
  }

  /// Returns accepted commands in their deterministic execution order.
  ///
  /// Rejected commands are never recorded. This read-only view is suitable for diagnostic
  /// export; it is not a playback or persistence contract.
  #[must_use]
  pub fn replay_commands(&self) -> &[Command] {
    self.trace.commands()
  }

  /// Returns core's deterministic legal command projection without mutating the state.
  #[must_use]
  pub fn legal_commands(&self) -> Vec<Command> {
    self.world.legal_commands()
  }

  /// Executes one canonical command through core and projects its semantic outcome.
  ///
  /// Rejected commands are not recorded. Core validates scheduling and gameplay, so this adapter
  /// does not duplicate those rules or mutate presentation state on an error.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is not accepted.
  pub fn execute(&mut self, command: Command) -> Result<PresentationOutput, CommandError> {
    let result = self.world.execute(command)?;
    self.trace.record(command);
    Ok(PresentationOutput {
      events: result.events().to_vec(),
      snapshot: self.snapshot(),
    })
  }

  /// Returns the immutable core map for adapters that need map-specific inspection.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    self.world.map()
  }
}

/// The Bevy-owned handle for one deterministic presentation run.
///
/// The wrapped [`PresentationState`] remains the only source of simulation truth. Bevy systems
/// may read snapshots or submit explicit core commands through this resource, while ECS scene
/// components remain disposable projections.
#[derive(Debug, Eq, PartialEq, Resource)]
pub struct PresentationRuntime {
  pub(crate) state: PresentationState,
  pub(crate) output: Option<PresentationOutput>,
}

impl PresentationRuntime {
  /// Starts a runtime from the shared authored starter floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core validation.
  pub fn start_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_run(seed)?,
      output: None,
    })
  }

  /// Starts a runtime from the shared authored starter-item scenario.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored item floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_item_run(seed)?,
      output: None,
    })
  }

  /// Starts a runtime from a deterministic seeded procedural floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when generated content fails core validation.
  pub fn start_procedural_run(seed: u64, depth: u32) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_procedural_run(seed, depth)?,
      output: None,
    })
  }

  /// Wraps an already validated presentation state as an app resource.
  #[must_use]
  pub fn new(state: PresentationState) -> Self {
    Self {
      state,
      output: None,
    }
  }

  /// Returns the explicit seed preserved by the runtime.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.state.seed()
  }

  /// Returns a read-only snapshot of the authoritative core world.
  #[must_use]
  pub fn snapshot(&self) -> PresentationSnapshot {
    self.state.snapshot()
  }

  /// Returns the deterministic digest of accepted runtime commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.state.replay_digest()
  }

  /// Returns accepted commands in their deterministic execution order.
  ///
  /// Rejected commands are never recorded. This read-only view is suitable for diagnostic
  /// export; it is not a playback or persistence contract.
  #[must_use]
  pub fn replay_commands(&self) -> &[Command] {
    self.state.replay_commands()
  }

  /// Returns core's deterministic legal command projection without mutating the state.
  #[must_use]
  pub fn legal_commands(&self) -> Vec<Command> {
    self.state.legal_commands()
  }

  /// Places one authored item on the ground for the display-free desktop smoke fixture.
  ///
  /// This setup-only mutation is intentionally not exposed as a player command and does not
  /// enter replay evidence; the smoke path then exercises the scheduled [`Command::Pickup`]
  /// transition through the same runtime used by the visible client.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_pickup(
    &mut self,
    actor: ActorId,
    item: ItemId,
  ) -> Result<(), dreadstep_core::WorldError> {
    self.state.world.drop_item(actor, item)
  }

  /// Repositions one authored actor for the display-free smoke fixture.
  ///
  /// This setup-only mutation does not enter replay evidence; it makes the reach-weapon smoke
  /// assertion independent of enemy chase timing while preserving the normal attack command.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_teleport(
    &mut self,
    actor: ActorId,
    position: Position,
  ) -> Result<(), dreadstep_core::WorldError> {
    self.state.world.teleport(actor, position)
  }

  /// Authors one Kiter in the display-free smoke fixture without entering replay evidence.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_kiter(
    &mut self,
    actor: ActorId,
  ) -> Result<(), dreadstep_core::WorldError> {
    if self
      .state
      .world
      .set_enemy_behavior(actor, EnemyBehavior::Kiter)
      .is_none()
    {
      return Err(dreadstep_core::WorldError::UnknownActor(actor));
    }
    Ok(())
  }

  /// Places one closed door for the display-free desktop smoke fixture.
  ///
  /// This setup-only mutation does not enter replay evidence; the smoke path then exercises the
  /// normal scheduled [`Command::Interact`] transition through the same runtime as the visible
  /// client.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_door(
    &mut self,
    position: Position,
  ) -> Result<(), dreadstep_core::WorldError> {
    if self.state.world.set_tile(position, Tile::Door).is_none() {
      return Err(dreadstep_core::WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position,
      });
    }
    Ok(())
  }

  /// Restores one authored showcase cell to ordinary floor for an independent smoke fixture.
  ///
  /// The visible starter-item showcase intentionally begins with a reachable closed door, while
  /// the smoke runner later reuses that cell for unrelated actor-placement and terrain checks.
  /// This setup-only normalization keeps those fixtures independent without entering replay
  /// evidence or changing the core player path.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_floor(
    &mut self,
    position: Position,
  ) -> Result<(), dreadstep_core::WorldError> {
    if self.state.world.set_tile(position, Tile::Floor).is_none() {
      return Err(dreadstep_core::WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position,
      });
    }
    Ok(())
  }

  /// Places one breakable terrain cell for the display-free desktop smoke fixture.
  ///
  /// This setup-only mutation does not enter replay evidence; the smoke path then exercises the
  /// normal scheduled [`Command::Break`] transition through the same runtime as the visible client.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_breakable(
    &mut self,
    position: Position,
  ) -> Result<(), dreadstep_core::WorldError> {
    if self
      .state
      .world
      .set_tile(position, Tile::Breakable)
      .is_none()
    {
      return Err(dreadstep_core::WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position,
      });
    }
    Ok(())
  }

  /// Places one floor trap for the display-free desktop smoke fixture.
  ///
  /// This setup-only mutation does not enter replay evidence; the smoke path then exercises the
  /// normal scheduled enemy chase transition through the same runtime as the visible client.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_trap(
    &mut self,
    position: Position,
  ) -> Result<(), dreadstep_core::WorldError> {
    if self.state.world.set_tile(position, Tile::Trap).is_none() {
      return Err(dreadstep_core::WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position,
      });
    }
    Ok(())
  }

  /// Places a one-shot chill trap for the display-free smoke fixture.
  #[cfg(feature = "desktop")]
  pub(crate) fn prepare_smoke_chill_trap(
    &mut self,
    position: Position,
  ) -> Result<(), dreadstep_core::WorldError> {
    if self
      .state
      .world
      .set_tile(position, Tile::ChillTrap)
      .is_none()
    {
      return Err(dreadstep_core::WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position,
      });
    }
    Ok(())
  }

  /// Returns the latest accepted command output without consuming it.
  #[must_use]
  pub const fn output(&self) -> Option<&PresentationOutput> {
    self.output.as_ref()
  }

  /// Takes the latest accepted command output, if one is pending.
  pub fn take_output(&mut self) -> Option<PresentationOutput> {
    self.output.take()
  }

  /// Delegates one command to the wrapped presentation state and core simulation.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is rejected. Rejected commands do not
  /// mutate the core world or replay trace, and clear any stale output so consumers never observe
  /// an earlier command as feedback for a rejected one.
  pub fn execute(&mut self, command: Command) -> Result<PresentationOutput, CommandError> {
    self.output = None;
    let output = self.state.execute(command)?;
    self.output = Some(output.clone());
    Ok(output)
  }
}
