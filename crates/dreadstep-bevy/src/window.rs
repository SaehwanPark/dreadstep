//! Validated window and tile-size presentation requests.

use bevy::ecs::resource::Resource;
use bevy::math::Vec2;
use dreadstep_core::Position;

use crate::ScenePixelPosition;

/// A validated logical window request for a future desktop presentation client.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationWindow {
  pub(crate) logical_width: u32,
  pub(crate) logical_height: u32,
  pub(crate) pixel_scale: u32,
  pub(crate) physical_width: u32,
  pub(crate) physical_height: u32,
}

impl PresentationWindow {
  /// Creates a non-empty request with checked integer pixel dimensions.
  #[must_use]
  pub const fn new(logical_width: u32, logical_height: u32, pixel_scale: u32) -> Option<Self> {
    if logical_width == 0 || logical_height == 0 || pixel_scale == 0 {
      return None;
    }
    let Some(physical_width) = logical_width.checked_mul(pixel_scale) else {
      return None;
    };
    let Some(physical_height) = logical_height.checked_mul(pixel_scale) else {
      return None;
    };
    Some(Self {
      logical_width,
      logical_height,
      pixel_scale,
      physical_width,
      physical_height,
    })
  }

  /// Returns the logical width before pixel scaling.
  #[must_use]
  pub const fn logical_width(self) -> u32 {
    self.logical_width
  }

  /// Returns the logical height before pixel scaling.
  #[must_use]
  pub const fn logical_height(self) -> u32 {
    self.logical_height
  }

  /// Returns the integer scale from logical to physical pixels.
  #[must_use]
  pub const fn pixel_scale(self) -> u32 {
    self.pixel_scale
  }

  /// Returns the checked physical width.
  #[must_use]
  pub const fn physical_width(self) -> u32 {
    self.physical_width
  }

  /// Returns the checked physical height.
  #[must_use]
  pub const fn physical_height(self) -> u32 {
    self.physical_height
  }
}

/// A caller-selected logical tile extent for the future renderer.
///
/// The proposal keeps 24×24 and 32×32 as asset-experiment candidates, so this resource does not
/// choose a project-wide default. It only validates the dimensions supplied by a presentation
/// client and provides checked conversion from map coordinates to logical pixel coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationTileSize {
  pub(crate) width: u32,
  pub(crate) height: u32,
}

impl PresentationTileSize {
  /// Creates a non-empty logical tile extent.
  #[must_use]
  pub const fn new(width: u32, height: u32) -> Option<Self> {
    if width == 0 || height == 0 {
      return None;
    }
    Some(Self { width, height })
  }

  /// Returns the logical tile width.
  #[must_use]
  pub const fn width(self) -> u32 {
    self.width
  }

  /// Returns the logical tile height.
  #[must_use]
  pub const fn height(self) -> u32 {
    self.height
  }

  /// Converts an in-map coordinate into a checked logical-pixel origin.
  #[must_use]
  pub fn pixel_position(self, position: Position) -> Option<ScenePixelPosition> {
    let x = u32::try_from(position.x()).ok()?;
    let y = u32::try_from(position.y()).ok()?;
    Some(ScenePixelPosition {
      x: x.checked_mul(self.width)?,
      y: y.checked_mul(self.height)?,
    })
  }

  pub(crate) fn sprite_size(self) -> Vec2 {
    // Bevy's Sprite API stores custom dimensions as f32; the selected presentation tile sizes
    // are small logical pixel extents (24×24/32×32), so this adapter conversion is intentional.
    #[allow(clippy::cast_precision_loss)]
    {
      Vec2::new(self.width as f32, self.height as f32)
    }
  }
}
