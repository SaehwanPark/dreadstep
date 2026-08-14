//! Second-person NetHack-style lines derived only from core events.

use dreadstep_core::{ActorId, ActorKind, BlockReason, Event, StatusKind};

use crate::glyphs::behavior_name;
use crate::session::{PLAYER, Session};

/// Formats one core event as a terminal message line.
#[must_use]
#[expect(
  clippy::too_many_lines,
  reason = "each Event variant keeps a single NetHack-style voice line"
)]
pub fn format_event(session: &Session, event: Event) -> String {
  match event {
    Event::Moved { actor, to, .. } => {
      if actor == PLAYER {
        format!("You move to ({}, {}).", to.x(), to.y())
      } else {
        format!(
          "{} moves to ({}, {}).",
          actor_name(session, actor),
          to.x(),
          to.y()
        )
      }
    }
    Event::MovementBlocked { actor, reason, .. } => {
      let subject = if actor == PLAYER {
        "You".to_string()
      } else {
        actor_name(session, actor)
      };
      match reason {
        BlockReason::Terrain => format!("{subject} cannot go there."),
        BlockReason::Actor(_) => format!("{subject} bump into a creature."),
      }
    }
    Event::Waited { actor, .. } => {
      if actor == PLAYER {
        "You wait.".to_string()
      } else {
        format!("{} waits.", actor_name(session, actor))
      }
    }
    Event::DoorOpened { actor, .. } => {
      if actor == PLAYER {
        "You open the door.".to_string()
      } else {
        format!("{} opens a door.", actor_name(session, actor))
      }
    }
    Event::DoorClosed { actor, .. } => {
      if actor == PLAYER {
        "You close the door.".to_string()
      } else {
        format!("{} closes a door.", actor_name(session, actor))
      }
    }
    Event::NoiseCreated { actor, .. } => {
      if actor == PLAYER {
        "You kick the door open. The noise echoes.".to_string()
      } else {
        format!(
          "{} kicks a door. The noise echoes.",
          actor_name(session, actor)
        )
      }
    }
    Event::BreakableBroken { actor, .. } => {
      if actor == PLAYER {
        "You smash the obstacle.".to_string()
      } else {
        format!("{} smashes an obstacle.", actor_name(session, actor))
      }
    }
    Event::TrapTriggered {
      actor,
      damage,
      remaining_hit_points,
      ..
    } => {
      if actor == PLAYER {
        format!(
          "A trap springs! You take {} damage ({}/{} HP).",
          damage.value(),
          remaining_hit_points.value(),
          session
            .actor(actor)
            .map_or(remaining_hit_points.value(), |record| {
              record.max_hit_points().value()
            })
        )
      } else {
        format!(
          "A trap springs under {} ({} HP remain).",
          actor_name(session, actor),
          remaining_hit_points.value()
        )
      }
    }
    Event::StatusApplied {
      actor,
      status,
      remaining_actions,
    } => {
      let status_name = status_name(status);
      if actor == PLAYER {
        format!("You are {status_name} ({remaining_actions} actions).")
      } else {
        format!(
          "{} is {status_name} ({remaining_actions} actions).",
          actor_name(session, actor)
        )
      }
    }
    Event::StatusExpired { actor, status } => {
      let status_name = status_name(status);
      if actor == PLAYER {
        format!("You are no longer {status_name}.")
      } else {
        format!("{} is no longer {status_name}.", actor_name(session, actor))
      }
    }
    Event::Attacked {
      attacker,
      target,
      damage,
      remaining_hit_points,
    } => {
      let verb = if attacker == PLAYER { "hit" } else { "hits" };
      format!(
        "{} {verb} {} for {} damage ({} HP).",
        actor_name(session, attacker),
        actor_name(session, target).to_lowercase(),
        damage.value(),
        remaining_hit_points.value()
      )
    }
    Event::ChillCast { caster, target } => {
      format!(
        "{} casts chill at {}.",
        actor_name(session, caster),
        actor_name(session, target).to_lowercase()
      )
    }
    Event::ItemThrown { actor, target, .. } => {
      if actor == PLAYER {
        format!(
          "You throw a frost flask at {}.",
          actor_name(session, target).to_lowercase()
        )
      } else {
        format!(
          "{} throws a frost flask at {}.",
          actor_name(session, actor),
          actor_name(session, target).to_lowercase()
        )
      }
    }
    Event::Died { actor } => {
      if actor == PLAYER {
        "You die...".to_string()
      } else {
        format!("{} dies.", actor_name(session, actor))
      }
    }
    Event::ItemEquipped { actor, item } => {
      if actor == PLAYER {
        format!("You wield item {}.", item.value())
      } else {
        format!(
          "{} wields item {}.",
          actor_name(session, actor),
          item.value()
        )
      }
    }
    Event::ItemUnequipped { actor, item } => {
      if actor == PLAYER {
        format!("You unwield item {}.", item.value())
      } else {
        format!(
          "{} unwields item {}.",
          actor_name(session, actor),
          item.value()
        )
      }
    }
    Event::ItemConsumed { actor, item, .. } => {
      if actor == PLAYER {
        format!("You use item {}.", item.value())
      } else {
        format!("{} uses item {}.", actor_name(session, actor), item.value())
      }
    }
    Event::ItemPickedUp { actor, item } => {
      if actor == PLAYER {
        format!("You pick up item {}.", item.value())
      } else {
        format!(
          "{} picks up item {}.",
          actor_name(session, actor),
          item.value()
        )
      }
    }
    Event::ItemDropped { actor, item } => {
      if actor == PLAYER {
        format!("You drop item {}.", item.value())
      } else {
        format!(
          "{} drops item {}.",
          actor_name(session, actor),
          item.value()
        )
      }
    }
    Event::Reloaded { actor, ammunition } => {
      if actor == PLAYER {
        format!("You reload ({ammunition} shots).")
      } else {
        format!(
          "{} reloads ({ammunition} shots).",
          actor_name(session, actor)
        )
      }
    }
  }
}

fn actor_name(session: &Session, id: ActorId) -> String {
  if id == PLAYER {
    return "You".to_string();
  }
  let Some(actor) = session.actor(id) else {
    return format!("creature {}", id.value());
  };
  match actor.kind() {
    ActorKind::Player => "You".to_string(),
    ActorKind::Enemy => format!(
      "the {}",
      behavior_name(actor.enemy_behavior()).to_lowercase()
    ),
  }
}

const fn status_name(status: StatusKind) -> &'static str {
  match status {
    StatusKind::Chilled => "chilled",
  }
}

#[cfg(test)]
mod tests {
  use super::format_event;
  use crate::session::{PLAYER, Session};
  use dreadstep_core::{Command, Event, Position};

  #[test]
  fn player_door_open_uses_second_person() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let output = session
      .execute(Command::Interact {
        actor: PLAYER,
        position: Position::new(2, 1),
      })
      .expect("door east of the player is legal");
    let line = format_event(&session, output.events()[0]);
    assert_eq!(line, "You open the door.");
    assert!(matches!(output.events()[0], Event::DoorOpened { .. }));
  }

  #[test]
  fn blocked_north_uses_terrain_voice() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let output = session
      .execute(Command::Move {
        actor: PLAYER,
        direction: dreadstep_core::Direction::North,
      })
      .expect("blocked movement is an accepted command");
    let line = format_event(&session, output.events()[0]);
    assert_eq!(line, "You cannot go there.");
  }
}
