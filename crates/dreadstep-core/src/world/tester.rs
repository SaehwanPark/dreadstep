//! Explicit tester mutations that stay outside player replay.
//!
//! These operations are deterministic and atomic, but they do not record accepted player
//! history. Protocol and MCP only convert requests into these methods.

use crate::{Actor, ActorId, HitPoints, Position, WorldError, WorldState};

impl WorldState {
  /// Validates and inserts one living actor for an explicit tester operation.
  ///
  /// Dead actor records do not occupy tiles, so a new living actor may use a position retained by
  /// a dead record. The inserted actor becomes ready at the world's current action time, so a
  /// tester mutation cannot rewind the deterministic timeline.
  ///
  /// # Errors
  ///
  /// Returns a [`WorldError`] when the identity, hit points, position, terrain, or living
  /// occupancy is invalid. A rejected actor is not inserted.
  pub fn spawn(&mut self, actor: Actor) -> Result<(), WorldError> {
    let actor_id = actor.id();
    let position = actor.position();
    if self.actors.contains_key(&actor_id) {
      return Err(WorldError::DuplicateActorId(actor_id));
    }
    if !actor.is_alive() {
      return Err(WorldError::ActorDeadAtStart { actor: actor_id });
    }
    if !self.map.in_bounds(position) {
      return Err(WorldError::ActorOutOfBounds {
        actor: actor_id,
        position,
      });
    }
    if !self.map.is_walkable(position) {
      return Err(WorldError::ActorOnBlockedTile {
        actor: actor_id,
        position,
      });
    }
    if let Some(first) = self
      .actors
      .values()
      .find(|existing| existing.is_alive() && existing.position() == position)
    {
      return Err(WorldError::OverlappingActors {
        first: first.id(),
        second: actor_id,
        position,
      });
    }

    let mut actor = actor;
    actor.ready_at = self.current_time;
    self.actors.insert(actor_id, actor);
    Ok(())
  }

  /// Teleports one existing actor for an explicit tester operation.
  ///
  /// Teleport preserves the actor's identity, life, hit points, inventory, and ready time, and
  /// does not alter the world's current action time. Living actors occupy destinations; dead
  /// records do not, so a dead actor may be positioned on a living actor's tile until it is
  /// revived. The destination must remain a walkable map position.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when no actor has the requested identity,
  /// [`WorldError::TeleportOutOfBounds`] or [`WorldError::TeleportOnBlockedTile`] when the
  /// destination is invalid, or [`WorldError::TeleportOccupied`] when a living actor would
  /// overlap another living actor. Rejected teleports leave the world unchanged.
  pub fn teleport(&mut self, actor_id: ActorId, position: Position) -> Result<(), WorldError> {
    let Some(existing) = self.actors.get(&actor_id) else {
      return Err(WorldError::UnknownActor(actor_id));
    };
    if !self.map.in_bounds(position) {
      return Err(WorldError::TeleportOutOfBounds {
        actor: actor_id,
        position,
      });
    }
    if !self.map.is_walkable(position) {
      return Err(WorldError::TeleportOnBlockedTile {
        actor: actor_id,
        position,
      });
    }
    if existing.is_alive()
      && let Some(blocker) = self
        .actors
        .values()
        .find(|actor| actor.id() != actor_id && actor.is_alive() && actor.position() == position)
    {
      return Err(WorldError::TeleportOccupied {
        actor: actor_id,
        blocker: blocker.id(),
        position,
      });
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?
      .position = position;
    Ok(())
  }

  /// Sets one existing actor's hit points for an explicit tester operation.
  ///
  /// Setting zero leaves the dead actor record inspectable while existing scheduling and
  /// occupancy queries exclude it. Reviving a dead actor anchors its readiness at the current
  /// action time so the mutation cannot rewind the deterministic timeline. Removing a living
  /// actor may advance the current time to the next surviving actor's readiness, but never moves
  /// it backward; other actor fields remain unchanged.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when no actor has the requested identity or
  /// [`WorldError::OverlappingActors`] when reviving would overlap a living actor.
  pub fn set_hit_points(
    &mut self,
    actor_id: ActorId,
    hit_points: HitPoints,
  ) -> Result<(), WorldError> {
    let current_time = self.current_time;
    let Some(existing) = self.actors.get(&actor_id) else {
      return Err(WorldError::UnknownActor(actor_id));
    };
    let was_alive = existing.is_alive();
    if !was_alive && hit_points.is_alive() {
      let position = existing.position();
      if let Some(first) = self
        .actors
        .values()
        .find(|actor| actor.is_alive() && actor.position() == position)
      {
        return Err(WorldError::OverlappingActors {
          first: first.id(),
          second: actor_id,
          position,
        });
      }
    }
    let actor = self
      .actors
      .get_mut(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    actor.hit_points = hit_points;
    if !was_alive && hit_points.is_alive() {
      actor.ready_at = current_time;
    } else if was_alive
      && !hit_points.is_alive()
      && let Some(next_actor) = self.next_actor()
      && let Some(next_ready_at) = self.actors.get(&next_actor).map(Actor::ready_at)
    {
      self.current_time = next_ready_at;
    }
    Ok(())
  }
}
