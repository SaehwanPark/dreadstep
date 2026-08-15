//! Adapter-owned glyph and color policy for the terminal map.

use dreadstep_core::{Actor, ActorKind, EnemyBehavior, Tile};

/// Named colors used by the TTY renderer. Goldens strip these to plain characters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CellColor {
  /// Default terminal foreground.
  Default,
  /// Walls and unseen-adjacent stone.
  Gray,
  /// Player `@`.
  WhiteBold,
  /// Ordinary living enemies.
  Red,
  /// Frostcaster glyph.
  Cyan,
  /// Doors and items.
  Yellow,
  /// Floor traps.
  Magenta,
  /// Healthy hit-point bar.
  Green,
  /// Wounded hit-point bar.
  DarkYellow,
  /// Critical hit-point bar and rejected-action messages.
  DarkRed,
}

/// One visible map or HUD cell.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Cell {
  glyph: char,
  color: CellColor,
  bold: bool,
}

impl Cell {
  /// Creates a colored cell.
  #[must_use]
  pub const fn new(glyph: char, color: CellColor) -> Self {
    Self {
      glyph,
      color,
      bold: matches!(color, CellColor::WhiteBold),
    }
  }

  /// Creates a default-colored cell.
  #[must_use]
  pub const fn plain(glyph: char) -> Self {
    Self::new(glyph, CellColor::Default)
  }

  /// Returns the character drawn in this cell.
  #[must_use]
  pub const fn glyph(self) -> char {
    self.glyph
  }

  /// Returns the adapter color for this cell.
  #[must_use]
  pub const fn color(self) -> CellColor {
    self.color
  }

  /// Returns whether the TTY renderer should request a bold attribute.
  #[must_use]
  pub const fn bold(self) -> bool {
    self.bold
  }
}

/// Returns the glyph and color for one terrain tile.
#[must_use]
pub const fn tile_cell(tile: Tile) -> Cell {
  match tile {
    Tile::Floor => Cell::new('.', CellColor::Gray),
    Tile::Cover => Cell::new(':', CellColor::Green),
    Tile::Wall => Cell::new('#', CellColor::Gray),
    Tile::Door => Cell::new('+', CellColor::Yellow),
    Tile::OpenDoor => Cell::new('\'', CellColor::Yellow),
    Tile::Breakable => Cell::new('%', CellColor::DarkYellow),
    Tile::Trap => Cell::new('^', CellColor::Magenta),
    Tile::ChillTrap => Cell::new('*', CellColor::Cyan),
  }
}

/// Returns the glyph and color for one actor occupying a cell.
#[must_use]
pub fn actor_cell(actor: &Actor) -> Cell {
  if !actor.is_alive() {
    return Cell::new('%', CellColor::Gray);
  }
  match actor.kind() {
    ActorKind::Player => Cell::new('@', CellColor::WhiteBold),
    ActorKind::Enemy => match actor.enemy_behavior() {
      EnemyBehavior::Pursuer => Cell::new('p', CellColor::Red),
      EnemyBehavior::Kiter => Cell::new('k', CellColor::Red),
      EnemyBehavior::Brute => Cell::new('B', CellColor::Red),
      EnemyBehavior::Frostcaster => Cell::new('F', CellColor::Cyan),
      EnemyBehavior::Blocker => Cell::new('b', CellColor::Red),
      EnemyBehavior::Scavenger => Cell::new('s', CellColor::Red),
    },
  }
}

/// Returns the ground-item glyph used when a visible floor cell has no actor.
#[must_use]
pub const fn ground_item_cell() -> Cell {
  Cell::new(')', CellColor::Yellow)
}

/// Returns the unseen FOV glyph.
#[must_use]
pub const fn unseen_cell() -> Cell {
  Cell::plain(' ')
}

/// Returns a lowercase behavior name for HUD text.
#[must_use]
pub const fn behavior_name(behavior: EnemyBehavior) -> &'static str {
  match behavior {
    EnemyBehavior::Pursuer => "Pursuer",
    EnemyBehavior::Kiter => "Kiter",
    EnemyBehavior::Brute => "Brute",
    EnemyBehavior::Frostcaster => "Frostcaster",
    EnemyBehavior::Blocker => "Blocker",
    EnemyBehavior::Scavenger => "Scavenger",
  }
}

#[cfg(test)]
mod tests {
  use super::{actor_cell, tile_cell};
  use dreadstep_core::{Actor, ActorId, ActorKind, EnemyBehavior, Position, Tile};

  #[test]
  fn terrain_glyphs_match_nethack_inspired_table() {
    assert_eq!(tile_cell(Tile::Floor).glyph(), '.');
    assert_eq!(tile_cell(Tile::Wall).glyph(), '#');
    assert_eq!(tile_cell(Tile::Cover).glyph(), ':');
    assert_eq!(tile_cell(Tile::Door).glyph(), '+');
    assert_eq!(tile_cell(Tile::OpenDoor).glyph(), '\'');
    assert_eq!(tile_cell(Tile::Breakable).glyph(), '%');
    assert_eq!(tile_cell(Tile::Trap).glyph(), '^');
    assert_eq!(tile_cell(Tile::ChillTrap).glyph(), '*');
  }

  #[test]
  fn actor_glyphs_distinguish_behaviors_and_corpses() {
    let player = Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1));
    assert_eq!(actor_cell(&player).glyph(), '@');
    let frost = Actor::with_enemy_behavior(
      ActorId::new(3),
      Position::new(2, 1),
      EnemyBehavior::Frostcaster,
    );
    assert_eq!(actor_cell(&frost).glyph(), 'F');
    let brute =
      Actor::with_enemy_behavior(ActorId::new(4), Position::new(3, 1), EnemyBehavior::Brute);
    assert_eq!(actor_cell(&brute).glyph(), 'B');
    let scavenger = Actor::with_enemy_behavior(
      ActorId::new(5),
      Position::new(4, 1),
      EnemyBehavior::Scavenger,
    );
    assert_eq!(actor_cell(&scavenger).glyph(), 's');
  }
}
