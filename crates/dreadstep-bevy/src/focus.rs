//! Disposable focus projection for the controlled actor.

use bevy::ecs::resource::Resource;
use dreadstep_core::{ActorId, Position};

/// A disposable focus projection for future camera and viewport systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationFocus {
  pub(crate) actor: ActorId,
  pub(crate) position: Option<Position>,
}

impl PresentationFocus {
  /// Creates an empty focus projection for one controlled actor.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      position: None,
    }
  }

  /// Returns the actor whose position is being projected.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the latest known core position, or `None` for an unknown actor.
  #[must_use]
  pub const fn position(self) -> Option<Position> {
    self.position
  }
}
