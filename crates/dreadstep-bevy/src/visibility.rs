//! Presentation-only field-of-view projection.

use bevy::ecs::resource::Resource;
use dreadstep_core::{ActorId, Position};

/// Presentation-only field-of-view projection for one actor.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct PresentationVisibility {
  pub(crate) actor: ActorId,
  pub(crate) radius: u32,
  pub(crate) active: bool,
  pub(crate) positions: Vec<Position>,
}

impl PresentationVisibility {
  /// Creates an inactive field-of-view request for one actor and cardinal step radius.
  #[must_use]
  pub const fn new(actor: ActorId, radius: u32) -> Self {
    Self {
      actor,
      radius,
      active: false,
      positions: Vec::new(),
    }
  }

  /// Returns the actor whose position anchors the projection.
  #[must_use]
  pub const fn actor(&self) -> ActorId {
    self.actor
  }

  /// Returns the maximum number of cardinal floor steps revealed from the anchor.
  #[must_use]
  pub const fn radius(&self) -> u32 {
    self.radius
  }

  /// Returns whether a valid runtime/input pair has populated the projection.
  #[must_use]
  pub const fn is_active(&self) -> bool {
    self.active
  }

  /// Returns visible positions in stable row-major map order.
  #[must_use]
  pub fn visible_positions(&self) -> &[Position] {
    &self.positions
  }

  /// Returns whether a map position is visible, or treats an inactive optional projection as a
  /// no-op so headless clients retain the historical fully visible behavior.
  #[must_use]
  pub fn is_visible(&self, position: Position) -> bool {
    !self.active || self.positions.contains(&position)
  }
}
