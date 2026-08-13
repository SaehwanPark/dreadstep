//! Melee and ranged attack resolution.
//!
//! Legal discovery and execution share the same reach and line-of-sight predicates so advertised
//! commands cannot fail for a different geometric reason.

use crate::{Actor, ActorId, CommandError, Damage, Event, MeleeReach, Position, Tile, WorldState};

impl WorldState {
  pub(super) fn attack(
    &mut self,
    attacker: ActorId,
    target: ActorId,
  ) -> Result<Vec<Event>, CommandError> {
    let reach = self
      .actors
      .get(&attacker)
      .map(Actor::melee_reach)
      .ok_or(CommandError::UnknownActor(attacker))?;
    self.attack_with_distance(
      attacker,
      target,
      |first, second| Self::is_melee_distance(first, second, reach),
      Damage::MELEE,
      false,
    )
  }

  pub(super) fn ranged_attack(
    &mut self,
    attacker: ActorId,
    target: ActorId,
  ) -> Result<Vec<Event>, CommandError> {
    self.attack_with_distance(
      attacker,
      target,
      Self::is_ranged_distance,
      Damage::RANGED,
      true,
    )
  }

  pub(super) fn attack_with_distance(
    &mut self,
    attacker: ActorId,
    target: ActorId,
    in_range: impl FnOnce(Position, Position) -> bool,
    damage: Damage,
    ranged: bool,
  ) -> Result<Vec<Event>, CommandError> {
    if attacker == target {
      return Err(CommandError::CannotAttackSelf(attacker));
    }
    let attacker_position = self
      .actors
      .get(&attacker)
      .map(Actor::position)
      .ok_or(CommandError::UnknownActor(attacker))?;
    let target_actor = self
      .actors
      .get(&target)
      .ok_or(CommandError::UnknownTarget(target))?;
    if !target_actor.is_alive() {
      return Err(CommandError::TargetDead(target));
    }
    let target_position = target_actor.position();
    if !in_range(attacker_position, target_position) {
      return Err(if ranged {
        CommandError::RangedAttackOutOfRange { attacker, target }
      } else {
        CommandError::AttackOutOfRange { attacker, target }
      });
    }
    if ranged && !self.has_ranged_line_of_sight(attacker_position, target_position) {
      return Err(CommandError::RangedAttackNoLineOfSight { attacker, target });
    }
    let remaining_hit_points = target_actor.hit_points().reduced_by(damage);
    self
      .actors
      .get_mut(&target)
      .ok_or(CommandError::UnknownTarget(target))?
      .hit_points = remaining_hit_points;
    let mut events = vec![Event::Attacked {
      attacker,
      target,
      damage,
      remaining_hit_points,
    }];
    if !remaining_hit_points.is_alive() {
      events.push(Event::Died { actor: target });
    }
    Ok(events)
  }

  pub(super) fn is_ranged_distance(first: Position, second: Position) -> bool {
    let distance = first
      .x()
      .abs_diff(second.x())
      .saturating_add(first.y().abs_diff(second.y()));
    (2..=3).contains(&distance)
  }

  pub(super) fn is_melee_distance(first: Position, second: Position, reach: MeleeReach) -> bool {
    let distance = first
      .x()
      .abs_diff(second.x())
      .saturating_add(first.y().abs_diff(second.y()));
    distance <= u32::from(reach.value())
  }

  pub(super) fn has_ranged_line_of_sight(&self, first: Position, second: Position) -> bool {
    if first.x() == second.x() {
      let step = if first.y() < second.y() { 1 } else { -1 };
      let mut y = first.y() + step;
      while y != second.y() {
        if self
          .map
          .tile_at(Position::new(first.x(), y))
          .is_none_or(Tile::blocks_ranged_line_of_sight)
        {
          return false;
        }
        y += step;
      }
      true
    } else if first.y() == second.y() {
      let step = if first.x() < second.x() { 1 } else { -1 };
      let mut x = first.x() + step;
      while x != second.x() {
        if self
          .map
          .tile_at(Position::new(x, first.y()))
          .is_none_or(Tile::blocks_ranged_line_of_sight)
        {
          return false;
        }
        x += step;
      }
      true
    } else {
      false
    }
  }
}
