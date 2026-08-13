//! Movement, chase direction, and trap entry as a consequence of a successful step.

use crate::{
  Actor, ActorId, ActorKind, BlockReason, CommandError, Damage, Direction, Event, Tile, WorldState,
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
    let from = chaser.position();
    let to = target_actor.position();
    if from.x() < to.x() {
      Ok(Direction::East)
    } else if from.x() > to.x() {
      Ok(Direction::West)
    } else if from.y() < to.y() {
      Ok(Direction::South)
    } else {
      Ok(Direction::North)
    }
  }
}
