//! Scheduled terrain verbs: open, kick, and break one adjacent cell.

use crate::{Actor, ActorId, CommandError, Event, Position, Tile, WorldState};

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
    for enemy in self.actors.values_mut().filter(|actor| {
      actor.is_alive()
        && actor.kind() == crate::ActorKind::Enemy
        && actor
          .position()
          .x()
          .abs_diff(position.x())
          .saturating_add(actor.position().y().abs_diff(position.y()))
          <= 3
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
        radius: 3,
      },
    ])
  }
}
