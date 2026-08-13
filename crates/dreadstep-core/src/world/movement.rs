//! Movement, chase direction, and trap entry as a consequence of a successful step.

use crate::{
  Actor, ActorId, ActorKind, BlockReason, CommandError, Damage, Direction, Event, Position, Tile,
  WorldState,
};

impl WorldState {
  pub(super) fn move_actor(
    &mut self,
    actor_id: ActorId,
    direction: Direction,
  ) -> Result<Vec<Event>, CommandError> {
    let from = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    let to = from.translated(direction);
    if !self.map.is_walkable(to) {
      Ok(vec![Event::MovementBlocked {
        actor: actor_id,
        from,
        to,
        reason: BlockReason::Terrain,
      }])
    } else if let Some(blocker) = self.actor_at(to) {
      Ok(vec![Event::MovementBlocked {
        actor: actor_id,
        from,
        to,
        reason: BlockReason::Actor(blocker),
      }])
    } else {
      self
        .actors
        .get_mut(&actor_id)
        .ok_or(CommandError::UnknownActor(actor_id))?
        .position = to;
      let mut events = vec![Event::Moved {
        actor: actor_id,
        from,
        to,
      }];
      if self.map.tile_at(to) == Some(Tile::Trap) {
        self
          .map
          .set_tile(to, Tile::Floor)
          .ok_or(CommandError::UnknownActor(actor_id))?;
        let damage = Damage::TRAP;
        let remaining_hit_points = {
          let actor = self
            .actors
            .get_mut(&actor_id)
            .ok_or(CommandError::UnknownActor(actor_id))?;
          let remaining_hit_points = actor.hit_points.reduced_by(damage);
          actor.hit_points = remaining_hit_points;
          if !remaining_hit_points.is_alive() {
            actor.heard_noise = None;
            actor.status = None;
          }
          remaining_hit_points
        };
        events.push(Event::TrapTriggered {
          actor: actor_id,
          position: to,
          damage,
          remaining_hit_points,
        });
        if !remaining_hit_points.is_alive() {
          events.push(Event::Died { actor: actor_id });
        }
      }
      if self.map.tile_at(to) == Some(Tile::ChillTrap) {
        self.map.set_tile(to, Tile::Floor);
        let status = self
          .actors
          .get_mut(&actor_id)
          .ok_or(CommandError::UnknownActor(actor_id))?
          .apply_chilled();
        events.push(Event::StatusApplied {
          actor: actor_id,
          status: status.kind(),
          remaining_actions: status.remaining_actions(),
        });
      }
      Ok(events)
    }
  }
  pub(super) fn chase_direction(
    &self,
    actor: ActorId,
    target: ActorId,
  ) -> Result<Direction, CommandError> {
    let chaser = self
      .actors
      .get(&actor)
      .ok_or(CommandError::UnknownActor(actor))?;
    if chaser.kind() != ActorKind::Enemy {
      return Err(CommandError::ChaseRequiresEnemy(actor));
    }
    if actor == target {
      return Err(CommandError::CannotChaseSelf(actor));
    }
    let target_actor = self
      .actors
      .get(&target)
      .ok_or(CommandError::UnknownTarget(target))?;
    if !target_actor.is_alive() {
      return Err(CommandError::TargetDead(target));
    }
    Ok(Self::direction_toward(
      chaser.position(),
      target_actor.position(),
    ))
  }

  pub(super) fn investigate(
    &mut self,
    actor_id: ActorId,
    position: Position,
  ) -> Result<Vec<Event>, CommandError> {
    let actor = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    if actor.kind() != ActorKind::Enemy {
      return Err(CommandError::InvestigateRequiresEnemy(actor_id));
    }
    let Some(heard_noise) = actor.heard_noise() else {
      return Err(CommandError::NoNoiseToInvestigate(actor_id));
    };
    if heard_noise != position || actor.position() == position {
      return Err(CommandError::InvestigateTargetInvalid {
        actor: actor_id,
        position,
      });
    }
    let direction = Self::direction_toward(actor.position(), position);
    let events = self.move_actor(actor_id, direction)?;
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .heard_noise = None;
    Ok(events)
  }

  pub(super) fn direction_toward(from: Position, to: Position) -> Direction {
    if from.x() < to.x() {
      Direction::East
    } else if from.x() > to.x() {
      Direction::West
    } else if from.y() < to.y() {
      Direction::South
    } else {
      Direction::North
    }
  }
}
