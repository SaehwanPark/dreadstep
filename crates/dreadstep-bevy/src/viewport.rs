//! Tile viewport request and clamping helpers.

use bevy::ecs::resource::Resource;
use dreadstep_core::Position;

/// A deterministic tile viewport requested by a presentation client.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationViewport {
  pub(crate) width: u32,
  pub(crate) height: u32,
  pub(crate) origin: Option<Position>,
  pub(crate) effective_width: u32,
  pub(crate) effective_height: u32,
}

impl PresentationViewport {
  /// Creates a non-empty viewport request.
  #[must_use]
  pub const fn new(width: u32, height: u32) -> Option<Self> {
    if width == 0 || height == 0 {
      return None;
    }
    Some(Self {
      width,
      height,
      origin: None,
      effective_width: 0,
      effective_height: 0,
    })
  }

  /// Returns the requested viewport width in tiles.
  #[must_use]
  pub const fn width(self) -> u32 {
    self.width
  }

  /// Returns the requested viewport height in tiles.
  #[must_use]
  pub const fn height(self) -> u32 {
    self.height
  }

  /// Returns the clamped row-major map origin, or `None` without an authoritative center.
  #[must_use]
  pub const fn origin(self) -> Option<Position> {
    self.origin
  }

  /// Returns the effective in-map width after clamping to the current map.
  #[must_use]
  pub const fn effective_width(self) -> u32 {
    self.effective_width
  }

  /// Returns the effective in-map height after clamping to the current map.
  #[must_use]
  pub const fn effective_height(self) -> u32 {
    self.effective_height
  }
}
