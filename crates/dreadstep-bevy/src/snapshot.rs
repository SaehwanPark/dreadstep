//! Immutable presentation snapshots and accepted-command output.

use dreadstep_core::{
  ActionTime, Actor, ActorId, Event, GroundItemStack, StateDigest, Tile, WorldState,
};

use crate::RunOutcome;

/// A deterministic read-only projection consumed by future map, actor, ground-item, and inventory
/// renderers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationSnapshot {
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) tiles: Vec<Tile>,
  pub(crate) actors: Vec<Actor>,
  pub(crate) ground_items: Vec<GroundItemStack>,
  pub(crate) outcome: RunOutcome,
  pub(crate) current_time: ActionTime,
  pub(crate) next_actor: Option<ActorId>,
  pub(crate) digest: StateDigest,
}

impl PresentationSnapshot {
  pub(crate) fn from_world(world: &WorldState) -> Self {
    Self {
      width: world.map().width(),
      height: world.map().height(),
      tiles: world.map().tiles().to_vec(),
      actors: world.actors().cloned().collect(),
      ground_items: world.ground_items().to_vec(),
      outcome: world.outcome(),
      current_time: world.current_time(),
      next_actor: world.next_actor(),
      digest: world.digest(),
    }
  }

  /// Returns the map width in tiles.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the map height in tiles.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns immutable row-major terrain for the projected map.
  #[must_use]
  pub fn tiles(&self) -> &[Tile] {
    &self.tiles
  }

  /// Returns immutable actor records in stable [`ActorId`] order.
  #[must_use]
  pub fn actors(&self) -> &[Actor] {
    &self.actors
  }

  /// Returns immutable ground-item stacks in core-provided row-major and insertion order.
  #[must_use]
  pub fn ground_items(&self) -> &[GroundItemStack] {
    &self.ground_items
  }

  /// Returns the canonical run outcome projected by core.
  #[must_use]
  pub const fn outcome(&self) -> RunOutcome {
    self.outcome
  }

  /// Returns the current core action time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns the actor currently selected by core scheduling.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the stable core state digest for this projection.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }
}

/// Evidence returned after one accepted presentation command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationOutput {
  pub(crate) events: Vec<Event>,
  pub(crate) snapshot: PresentationSnapshot,
}

impl PresentationOutput {
  /// Returns semantic core events in deterministic execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the post-command presentation projection.
  #[must_use]
  pub const fn snapshot(&self) -> &PresentationSnapshot {
    &self.snapshot
  }
}
