//! Disposable camera-anchor projection.

use bevy::ecs::resource::Resource;
use dreadstep_core::{ActorId, Position};

/// A disposable camera-anchor projection for future viewport systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationCamera {
  pub(crate) actor: ActorId,
  pub(crate) center: Option<Position>,
}

impl PresentationCamera {
  /// Creates a camera anchor for one controlled actor before a runtime projection exists.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      center: None,
    }
  }

  /// Returns the actor whose position supplies this camera anchor.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the latest authoritative center, or `None` for an unknown actor.
  #[must_use]
  pub const fn center(self) -> Option<Position> {
    self.center
  }
}
