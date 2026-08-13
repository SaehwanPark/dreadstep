//! Scheduled terrain verbs: open, kick, and break one adjacent cell.

use std::collections::VecDeque;

use crate::{Actor, ActorId, CommandError, Direction, Event, GridMap, Position, Tile, WorldState};

const KICK_NOISE_RADIUS: u8 = 3;

impl WorldState {
  pub(super) fn interact(
    &mut self,
    actor_id: ActorId,
    position: Position,
  ) -> Result<Event, CommandError> {
    let actor_position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    let adjacent = actor_position
      .x()
      .abs_diff(position.x())
      .checked_add(actor_position.y().abs_diff(position.y()))
      == Some(1);
    if !adjacent || self.map.tile_at(position) != Some(Tile::Door) {
      return Err(CommandError::InteractTargetInvalid {
        actor: actor_id,
        position,
      });
    }
    self
      .map
      .set_tile(position, Tile::Floor)
      .ok_or(CommandError::InteractTargetInvalid {
        actor: actor_id,
        position,
      })?;
    Ok(Event::DoorOpened {
      actor: actor_id,
      position,
    })
  }

  pub(super) fn break_terrain(
    &mut self,
    actor_id: ActorId,
    position: Position,
  ) -> Result<Event, CommandError> {
    let actor_position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    let adjacent = actor_position
      .x()
      .abs_diff(position.x())
      .checked_add(actor_position.y().abs_diff(position.y()))
      == Some(1);
    if !adjacent || self.map.tile_at(position) != Some(Tile::Breakable) {
      return Err(CommandError::BreakTargetInvalid {
        actor: actor_id,
        position,
      });
    }
    self
      .map
      .set_tile(position, Tile::Floor)
      .ok_or(CommandError::BreakTargetInvalid {
        actor: actor_id,
        position,
      })?;
    Ok(Event::BreakableBroken {
      actor: actor_id,
      position,
    })
  }

  pub(super) fn kick_door(
    &mut self,
    actor_id: ActorId,
    position: Position,
  ) -> Result<Vec<Event>, CommandError> {
    let actor_position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    let adjacent = actor_position
      .x()
      .abs_diff(position.x())
      .checked_add(actor_position.y().abs_diff(position.y()))
      == Some(1);
    if !adjacent || self.map.tile_at(position) != Some(Tile::Door) {
      return Err(CommandError::KickTargetInvalid {
        actor: actor_id,
        position,
      });
    }
    self
      .map
      .set_tile(position, Tile::Floor)
      .ok_or(CommandError::KickTargetInvalid {
        actor: actor_id,
        position,
      })?;
    let audible_positions = audible_positions(&self.map, position);
    for enemy in self.actors.values_mut().filter(|actor| {
      actor.is_alive()
        && actor.kind() == crate::ActorKind::Enemy
        && audible_positions.contains(&actor.position())
    }) {
      enemy.heard_noise = Some(position);
    }
    Ok(vec![
      Event::DoorOpened {
        actor: actor_id,
        position,
      },
      Event::NoiseCreated {
        actor: actor_id,
        position,
        radius: KICK_NOISE_RADIUS,
      },
    ])
  }
}

/// Returns the walkable cells reached by a kick's sound in stable cardinal BFS order.
///
/// Terrain, not actor occupancy, determines whether sound crosses a cell. Keeping this bounded
/// to the fixed kick radius avoids introducing a persistent sound field or a second source model.
fn audible_positions(map: &GridMap, source: Position) -> Vec<Position> {
  if !map.is_walkable(source) {
    return Vec::new();
  }
  let mut reached = vec![source];
  let mut pending = VecDeque::from([(source, 0_u8)]);
  while let Some((position, distance)) = pending.pop_front() {
    if distance == KICK_NOISE_RADIUS {
      continue;
    }
    for direction in [
      Direction::North,
      Direction::South,
      Direction::West,
      Direction::East,
    ] {
      let next = position.translated(direction);
      if !map.is_walkable(next) || reached.contains(&next) {
        continue;
      }
      reached.push(next);
      pending.push_back((next, distance + 1));
    }
  }
  reached
}
