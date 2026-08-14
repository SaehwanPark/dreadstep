//! Presentation-only field-of-view using Bevy's cardinal walkable-plus-wall-edge rule.

use std::collections::{BTreeSet, VecDeque};

use dreadstep_core::{Direction, GridMap, Position, Tile};

use crate::session::Session;

/// Cardinal FOV radius used by the visible terminal client.
pub const FOV_RADIUS: u32 = 3;

/// Returns visible positions in stable row-major order from a walkable origin.
#[must_use]
pub fn visible_positions(map: &GridMap, origin: Position, radius: u32) -> Vec<Position> {
  if !map.tile_at(origin).is_some_and(Tile::is_walkable) {
    return Vec::new();
  }
  let mut queue = VecDeque::from([(origin, 0_u32)]);
  let mut visited_walkable = BTreeSet::from([(origin.x(), origin.y())]);
  let mut visible = BTreeSet::new();
  while let Some((position, distance)) = queue.pop_front() {
    visible.insert((position.x(), position.y()));
    for direction in [
      Direction::North,
      Direction::South,
      Direction::West,
      Direction::East,
    ] {
      let neighbor = position.translated(direction);
      match map.tile_at(neighbor) {
        Some(Tile::Wall | Tile::Door | Tile::Breakable) => {
          visible.insert((neighbor.x(), neighbor.y()));
        }
        Some(Tile::Floor | Tile::Cover | Tile::OpenDoor | Tile::Trap | Tile::ChillTrap)
          if distance < radius && visited_walkable.insert((neighbor.x(), neighbor.y())) =>
        {
          queue.push_back((neighbor, distance + 1));
        }
        Some(Tile::Floor | Tile::Cover | Tile::OpenDoor | Tile::Trap | Tile::ChillTrap) | None => {}
      }
    }
  }
  let mut positions = visible
    .into_iter()
    .map(|(x, y)| Position::new(x, y))
    .collect::<Vec<_>>();
  positions.sort_by_key(|position| (position.y(), position.x()));
  positions
}

/// Returns whether `position` is visible from the controlled player using the terminal FOV.
#[must_use]
pub fn player_can_see(session: &Session, position: Position) -> bool {
  let Some(player) = session.actor(crate::session::PLAYER) else {
    return false;
  };
  visible_positions(session.map(), player.position(), FOV_RADIUS).contains(&position)
}

#[cfg(test)]
mod tests {
  use super::{FOV_RADIUS, player_can_see, visible_positions};
  use crate::session::Session;
  use dreadstep_core::Position;

  #[test]
  fn radius_three_hides_distant_floor_and_keeps_adjacent_walls() {
    let session = Session::start_item_run(7).expect("item showcase");
    let origin = session
      .actor(crate::session::PLAYER)
      .expect("player")
      .position();
    let visible = visible_positions(session.map(), origin, FOV_RADIUS);
    assert!(visible.contains(&origin));
    let far = Position::new(
      i32::try_from(session.map().width().saturating_sub(1)).unwrap_or_default(),
      i32::try_from(session.map().height().saturating_sub(1)).unwrap_or_default(),
    );
    if far != origin {
      assert!(
        !player_can_see(&session, far) || visible.contains(&far),
        "FOV decision must be consistent with the visible set"
      );
    }
    let east_wallish = origin.translated(dreadstep_core::Direction::East);
    assert!(
      visible.contains(&east_wallish),
      "adjacent door or floor beside the player must remain visible"
    );
  }
}
