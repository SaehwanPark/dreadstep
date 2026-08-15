//! Deterministic enemy intent selection policy.
//!
//! This read-only policy is shared by presentation intent and the desktop enemy driver.

use crate::{Actor, ActorId, ActorKind, Command, EnemyBehavior, Tile, WorldState};

impl WorldState {
  /// Selects the deterministic enemy intent from the scheduled actor's legal commands.
  ///
  /// A Kiter retreats from an adjacent target; a wounded Scavenger prioritizes retreat;
  /// a Blocker attacks or waits in place; a Brute breaks blocking breakables before pursuit;
  /// a Frostcaster casts chill at range before standard attacks.
  #[must_use]
  pub fn preferred_enemy_command(&self, actor_id: ActorId, target_id: ActorId) -> Option<Command> {
    let actor = self.actors.get(&actor_id)?;
    if actor.kind() != ActorKind::Enemy || !actor.is_alive() || self.next_actor() != Some(actor_id)
    {
      return None;
    }
    let legal = self.legal_commands();

    if let Some(command) = Self::preferred_kiter_command(actor, actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_scavenger_command(actor, actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_blocker_command(actor, actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_attack_command(actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_frostcaster_command(actor, actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_ranged_command(actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_investigate_command(actor_id, &legal) {
      return Some(command);
    }
    if let Some(command) = self.preferred_brute_command(actor, actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_chase_command(actor_id, target_id, &legal) {
      return Some(command);
    }
    if let Some(command) = Self::preferred_wait_command(actor_id, &legal) {
      return Some(command);
    }
    legal.iter().find(|cmd| cmd.actor() == actor_id).copied()
  }

  fn preferred_kiter_command(
    actor: &Actor,
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    if actor.enemy_behavior() == EnemyBehavior::Kiter {
      legal
        .iter()
        .find(|cmd| {
          matches!(
            cmd,
            Command::Retreat { actor, target } if *actor == actor_id && *target == target_id
          )
        })
        .copied()
    } else {
      None
    }
  }

  fn preferred_scavenger_command(
    actor: &Actor,
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    if actor.enemy_behavior() == EnemyBehavior::Scavenger
      && actor.hit_points().value() < actor.max_hit_points().value()
    {
      if let Some(command) = legal.iter().find(|cmd| {
        matches!(
          cmd,
          Command::Retreat { actor, target } if *actor == actor_id && *target == target_id
        )
      }) {
        return Some(*command);
      }
      if let Some(command) = legal.iter().find(|cmd| {
        matches!(
          cmd,
          Command::Attack { actor, target } if *actor == actor_id && *target == target_id
        )
      }) {
        return Some(*command);
      }
      legal
        .iter()
        .find(|cmd| matches!(cmd, Command::Wait { actor } if *actor == actor_id))
        .copied()
    } else {
      None
    }
  }

  fn preferred_blocker_command(
    actor: &Actor,
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    if actor.enemy_behavior() == EnemyBehavior::Blocker {
      if let Some(command) = legal.iter().find(|cmd| {
        matches!(
          cmd,
          Command::Attack { actor, target } if *actor == actor_id && *target == target_id
        )
      }) {
        return Some(*command);
      }
      legal
        .iter()
        .find(|cmd| matches!(cmd, Command::Wait { actor } if *actor == actor_id))
        .copied()
    } else {
      None
    }
  }

  fn preferred_attack_command(
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    legal
      .iter()
      .find(|cmd| {
        matches!(
          cmd,
          Command::Attack { actor, target } if *actor == actor_id && *target == target_id
        )
      })
      .copied()
  }

  fn preferred_frostcaster_command(
    actor: &Actor,
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    if actor.enemy_behavior() == EnemyBehavior::Frostcaster {
      legal
        .iter()
        .find(|cmd| {
          matches!(
            cmd,
            Command::CastChill { actor, target } if *actor == actor_id && *target == target_id
          )
        })
        .copied()
    } else {
      None
    }
  }

  fn preferred_ranged_command(
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    legal
      .iter()
      .find(|cmd| {
        matches!(
          cmd,
          Command::RangedAttack { actor, target } if *actor == actor_id && *target == target_id
        )
      })
      .copied()
  }

  fn preferred_investigate_command(actor_id: ActorId, legal: &[Command]) -> Option<Command> {
    legal
      .iter()
      .find(|cmd| matches!(cmd, Command::Investigate { actor, .. } if *actor == actor_id))
      .copied()
  }

  fn preferred_brute_command(
    &self,
    actor: &Actor,
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    if actor.enemy_behavior() == EnemyBehavior::Brute
      && let Ok(direction) = self.chase_direction(actor_id, target_id)
    {
      let position = actor.position().translated(direction);
      if self.map.tile_at(position) == Some(Tile::Breakable) {
        return legal
          .iter()
          .find(|cmd| {
            matches!(
              cmd,
              Command::Break { actor, position: candidate }
                if *actor == actor_id && *candidate == position
            )
          })
          .copied();
      }
    }
    None
  }

  fn preferred_chase_command(
    actor_id: ActorId,
    target_id: ActorId,
    legal: &[Command],
  ) -> Option<Command> {
    legal
      .iter()
      .find(|cmd| {
        matches!(
          cmd,
          Command::Chase { actor, target } if *actor == actor_id && *target == target_id
        )
      })
      .copied()
  }

  fn preferred_wait_command(actor_id: ActorId, legal: &[Command]) -> Option<Command> {
    legal
      .iter()
      .find(|cmd| matches!(cmd, Command::Wait { actor } if *actor == actor_id))
      .copied()
  }
}
