//! Typed HUD status and scheduled-enemy intent projections.

use bevy::ecs::resource::Resource;
use dreadstep_core::{
  ActionTime, ActorId, ActorKind, Command, EnemyBehavior, HitPoints, Position, Status,
};

/// A typed status projection for a future HUD.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationHud {
  pub(crate) actor: ActorId,
  pub(crate) kind: Option<ActorKind>,
  pub(crate) position: Option<Position>,
  pub(crate) hit_points: Option<HitPoints>,
  pub(crate) ready_at: Option<ActionTime>,
  pub(crate) status: Option<Status>,
}

/// A read-only projection of the currently scheduled enemy's next legal core command.
///
/// This is an intent signal for presentation only. It does not reserve the command, alter core
/// scheduling, or predict a future turn after the current scheduler state changes.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct PresentationEnemyIntent {
  pub(crate) actor: Option<ActorId>,
  pub(crate) behavior: Option<EnemyBehavior>,
  pub(crate) command: Option<Command>,
}

impl PresentationEnemyIntent {
  /// Creates an empty intent projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      actor: None,
      behavior: None,
      command: None,
    }
  }

  /// Returns the scheduled enemy identity, when core has one.
  #[must_use]
  pub const fn actor(&self) -> Option<ActorId> {
    self.actor
  }

  /// Returns the authored behavior of the scheduled enemy, when core has one.
  #[must_use]
  pub const fn behavior(&self) -> Option<EnemyBehavior> {
    self.behavior
  }

  /// Returns the exact legal command selected for presentation from core's current projection.
  #[must_use]
  pub const fn command(&self) -> Option<Command> {
    self.command
  }
}

impl Default for PresentationEnemyIntent {
  fn default() -> Self {
    Self::new()
  }
}

impl PresentationHud {
  /// Creates an empty HUD projection for one controlled actor identity.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      kind: None,
      position: None,
      hit_points: None,
      ready_at: None,
      status: None,
    }
  }

  /// Returns the actor whose status is being projected.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the controlled actor kind, or `None` for an unknown actor.
  #[must_use]
  pub const fn kind(self) -> Option<ActorKind> {
    self.kind
  }

  /// Returns the controlled actor position, or `None` for an unknown actor.
  #[must_use]
  pub const fn position(self) -> Option<Position> {
    self.position
  }

  /// Returns the controlled actor hit points, or `None` for an unknown actor.
  #[must_use]
  pub const fn hit_points(self) -> Option<HitPoints> {
    self.hit_points
  }

  /// Returns the controlled actor's next-ready time, or `None` for an unknown actor.
  #[must_use]
  pub const fn ready_at(self) -> Option<ActionTime> {
    self.ready_at
  }

  /// Returns the controlled actor's active status, if any.
  #[must_use]
  pub const fn status(self) -> Option<Status> {
    self.status
  }
}
