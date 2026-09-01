//! Rectangular terrain, coordinates, and walkability.
//!
//! Out-of-bounds and blocking tiles are terrain facts. Living occupancy is decided by the world
//! so events can distinguish a wall from another actor.

use std::fmt;

/// A tile coordinate in the simulation's row-major grid.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Position {
  x: i32,
  y: i32,
}

impl Position {
  /// Creates a coordinate from its horizontal and vertical components.
  #[must_use]
  pub const fn new(x: i32, y: i32) -> Self {
    Self { x, y }
  }

  /// Returns the horizontal component.
  #[must_use]
  pub const fn x(self) -> i32 {
    self.x
  }

  /// Returns the vertical component.
  #[must_use]
  pub const fn y(self) -> i32 {
    self.y
  }

  /// Returns the adjacent coordinate in the supplied cardinal direction.
  #[must_use]
  pub const fn translated(self, direction: Direction) -> Self {
    let (dx, dy) = direction.delta();
    Self {
      x: self.x.saturating_add(dx),
      y: self.y.saturating_add(dy),
    }
  }
}

/// The four movement directions supported by the first rules-kernel slice.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
  /// One row toward decreasing vertical coordinates.
  North,
  /// One row toward increasing vertical coordinates.
  South,
  /// One column toward decreasing horizontal coordinates.
  West,
  /// One column toward increasing horizontal coordinates.
  East,
}

impl Direction {
  const fn delta(self) -> (i32, i32) {
    match self {
      Self::North => (0, -1),
      Self::South => (0, 1),
      Self::West => (-1, 0),
      Self::East => (1, 0),
    }
  }
}

/// The terrain occupying one grid cell.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Tile {
  /// A cell that actors may enter when it is not occupied.
  Floor,
  /// A walkable, transparent exit marker for the current floor.
  Stairs,
  /// A walkable cell that blocks ranged line of sight.
  Cover,
  /// A cell that blocks movement.
  Wall,
  /// A closed door that blocks movement until an adjacent actor opens it.
  Door,
  /// An opened door that remains walkable and transparent until an adjacent actor closes it.
  OpenDoor,
  /// A blocking terrain cell that an adjacent actor may break into floor.
  Breakable,
  /// A walkable floor trap that triggers once when an actor enters it.
  Trap,
  /// A walkable one-shot trap that refreshes the chilled status when entered.
  ChillTrap,
}

impl Tile {
  /// Returns whether this tile permits an actor to enter it.
  #[must_use]
  pub const fn is_walkable(self) -> bool {
    matches!(
      self,
      Self::Floor | Self::Stairs | Self::Cover | Self::OpenDoor | Self::Trap | Self::ChillTrap
    )
  }

  /// Returns whether this tile blocks a ranged line of sight.
  #[must_use]
  pub const fn blocks_ranged_line_of_sight(self) -> bool {
    matches!(
      self,
      Self::Cover | Self::Wall | Self::Door | Self::Breakable
    )
  }
}

/// Errors produced while constructing a rectangular grid map.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MapError {
  /// The width is zero, so no columns exist.
  ZeroWidth,
  /// The height is zero, so no rows exist.
  ZeroHeight,
  /// The dimensions cannot be represented as an in-memory tile buffer.
  TooLarge {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// A dimension would contain coordinates outside the signed position domain.
  CoordinateRange {
    /// The requested width.
    width: u32,
    /// The requested height.
    height: u32,
  },
  /// The tile buffer does not match the dimensions.
  TileCountMismatch {
    /// The number of tiles implied by the dimensions.
    expected: usize,
    /// The number of tiles supplied by the caller.
    actual: usize,
  },
}

impl fmt::Display for MapError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::ZeroWidth => formatter.write_str("map width must be greater than zero"),
      Self::ZeroHeight => formatter.write_str("map height must be greater than zero"),
      Self::TooLarge { width, height } => {
        write!(formatter, "map dimensions {width}x{height} are too large")
      }
      Self::CoordinateRange { width, height } => write!(
        formatter,
        "map dimensions {width}x{height} exceed the signed position range"
      ),
      Self::TileCountMismatch { expected, actual } => {
        write!(
          formatter,
          "map needs {expected} tiles but received {actual}"
        )
      }
    }
  }
}

impl std::error::Error for MapError {}

/// A finite, row-major rectangular map of terrain tiles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GridMap {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
}

impl GridMap {
  /// Creates a map filled with one terrain tile.
  ///
  /// # Errors
  ///
  /// Returns [`MapError::ZeroWidth`], [`MapError::ZeroHeight`], [`MapError::CoordinateRange`],
  /// or [`MapError::TooLarge`] when the requested dimensions cannot describe a valid buffer.
  pub fn filled(width: u32, height: u32, tile: Tile) -> Result<Self, MapError> {
    let tile_count = tile_count(width, height)?;
    Ok(Self {
      width,
      height,
      tiles: vec![tile; tile_count],
    })
  }

  /// Creates a map from row-major tiles, validating the dimensions and buffer length.
  ///
  /// # Errors
  ///
  /// Returns a [`MapError`] when a dimension is zero, the dimensions are too large, or the
  /// supplied tile count does not match the dimensions.
  pub fn from_tiles(width: u32, height: u32, tiles: Vec<Tile>) -> Result<Self, MapError> {
    let expected = tile_count(width, height)?;
    if tiles.len() != expected {
      return Err(MapError::TileCountMismatch {
        expected,
        actual: tiles.len(),
      });
    }
    Ok(Self {
      width,
      height,
      tiles,
    })
  }

  /// Returns the number of columns in the map.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the number of rows in the map.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns the validated terrain tiles in row-major order.
  ///
  /// The returned slice is immutable, so presentation and content adapters can inspect map
  /// terrain without bypassing the map's construction invariants or mutating core state.
  #[must_use]
  pub fn tiles(&self) -> &[Tile] {
    &self.tiles
  }

  /// Returns whether a position is inside the map bounds.
  #[must_use]
  pub fn in_bounds(&self, position: Position) -> bool {
    position.x >= 0
      && position.y >= 0
      && u32::try_from(position.x).is_ok_and(|x| x < self.width)
      && u32::try_from(position.y).is_ok_and(|y| y < self.height)
  }

  /// Returns the terrain at a position, or `None` outside the map.
  #[must_use]
  pub fn tile_at(&self, position: Position) -> Option<Tile> {
    self
      .in_bounds(position)
      .then(|| self.tiles[self.index(position)])
  }

  /// Returns whether an actor may enter a position based on terrain alone.
  #[must_use]
  pub fn is_walkable(&self, position: Position) -> bool {
    self.tile_at(position).is_some_and(Tile::is_walkable)
  }

  /// Replaces one in-bounds tile and returns the previous terrain.
  ///
  /// The bounded mutation is used by semantic world transitions such as opening a door. `None`
  /// means that the requested position is outside this validated map.
  pub fn set_tile(&mut self, position: Position, tile: Tile) -> Option<Tile> {
    if !self.in_bounds(position) {
      return None;
    }
    let index = self.index(position);
    Some(std::mem::replace(&mut self.tiles[index], tile))
  }

  fn index(&self, position: Position) -> usize {
    let x = usize::try_from(position.x).expect("in-bounds positions have non-negative x");
    let y = usize::try_from(position.y).expect("in-bounds positions have non-negative y");
    let width = usize::try_from(self.width).expect("map width must fit usize");
    y * width + x
  }
}

fn tile_count(width: u32, height: u32) -> Result<usize, MapError> {
  if width == 0 {
    return Err(MapError::ZeroWidth);
  }
  if height == 0 {
    return Err(MapError::ZeroHeight);
  }
  if width > i32::MAX as u32 || height > i32::MAX as u32 {
    return Err(MapError::CoordinateRange { width, height });
  }
  usize::try_from(u64::from(width) * u64::from(height))
    .map_err(|_| MapError::TooLarge { width, height })
}
