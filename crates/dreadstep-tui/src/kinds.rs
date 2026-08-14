//! Stable command and event kind names used by smoke coverage and journals.

use dreadstep_core::{Command, Direction, Event, Position, RunOutcome};
use serde_json::{Value, json};

/// Every current command kind that TUI smoke must demonstrate.
pub const SHOWCASE_COMMAND_KINDS: [&str; 19] = [
  "move",
  "wait",
  "interact",
  "kick",
  "close",
  "break",
  "attack",
  "ranged_attack",
  "cast_chill",
  "throw",
  "retreat",
  "chase",
  "investigate",
  "equip",
  "unequip",
  "use_item",
  "pickup",
  "drop",
  "reload",
];

/// Every current event kind that TUI smoke must observe.
pub const SHOWCASE_EVENT_KINDS: [&str; 20] = [
  "moved",
  "movement_blocked",
  "waited",
  "door_opened",
  "door_closed",
  "noise_created",
  "breakable_broken",
  "trap_triggered",
  "status_applied",
  "status_expired",
  "attacked",
  "chill_cast",
  "item_thrown",
  "died",
  "item_equipped",
  "item_unequipped",
  "item_consumed",
  "item_picked_up",
  "item_dropped",
  "reloaded",
];

/// Returns the smoke/journal kind name for one core command.
#[must_use]
pub const fn command_name(command: Command) -> &'static str {
  match command {
    Command::Move { .. } => "move",
    Command::Wait { .. } => "wait",
    Command::Interact { .. } => "interact",
    Command::Break { .. } => "break",
    Command::Kick { .. } => "kick",
    Command::Close { .. } => "close",
    Command::Attack { .. } => "attack",
    Command::RangedAttack { .. } => "ranged_attack",
    Command::CastChill { .. } => "cast_chill",
    Command::Throw { .. } => "throw",
    Command::Retreat { .. } => "retreat",
    Command::Chase { .. } => "chase",
    Command::Investigate { .. } => "investigate",
    Command::Equip { .. } => "equip",
    Command::Unequip { .. } => "unequip",
    Command::UseItem { .. } => "use_item",
    Command::Pickup { .. } => "pickup",
    Command::Drop { .. } => "drop",
    Command::Reload { .. } => "reload",
  }
}

/// Returns the smoke/journal kind name for one core event.
#[must_use]
pub const fn event_name(event: Event) -> &'static str {
  match event {
    Event::Moved { .. } => "moved",
    Event::MovementBlocked { .. } => "movement_blocked",
    Event::Waited { .. } => "waited",
    Event::DoorOpened { .. } => "door_opened",
    Event::DoorClosed { .. } => "door_closed",
    Event::NoiseCreated { .. } => "noise_created",
    Event::BreakableBroken { .. } => "breakable_broken",
    Event::TrapTriggered { .. } => "trap_triggered",
    Event::StatusApplied { .. } => "status_applied",
    Event::StatusExpired { .. } => "status_expired",
    Event::Attacked { .. } => "attacked",
    Event::ChillCast { .. } => "chill_cast",
    Event::ItemThrown { .. } => "item_thrown",
    Event::Died { .. } => "died",
    Event::ItemEquipped { .. } => "item_equipped",
    Event::ItemUnequipped { .. } => "item_unequipped",
    Event::ItemConsumed { .. } => "item_consumed",
    Event::ItemPickedUp { .. } => "item_picked_up",
    Event::ItemDropped { .. } => "item_dropped",
    Event::Reloaded { .. } => "reloaded",
  }
}

/// Returns a JSON object describing one core command.
#[must_use]
pub fn command_value(command: Command) -> Value {
  match command {
    Command::Move { actor, direction } => {
      json!({ "kind": "move", "actor": actor.value(), "direction": direction_name(direction) })
    }
    Command::Wait { actor } => json!({ "kind": "wait", "actor": actor.value() }),
    Command::Interact { actor, position } => json!({
      "kind": "interact",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Break { actor, position } => json!({
      "kind": "break",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Kick { actor, position } => json!({
      "kind": "kick",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Close { actor, position } => json!({
      "kind": "close",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Attack { actor, target } => {
      json!({ "kind": "attack", "actor": actor.value(), "target": target.value() })
    }
    Command::RangedAttack { actor, target } => {
      json!({ "kind": "ranged_attack", "actor": actor.value(), "target": target.value() })
    }
    Command::CastChill { actor, target } => {
      json!({ "kind": "cast_chill", "actor": actor.value(), "target": target.value() })
    }
    Command::Throw {
      actor,
      item,
      target,
    } => {
      json!({ "kind": "throw", "actor": actor.value(), "item": item.value(), "target": target.value() })
    }
    Command::Retreat { actor, target } => {
      json!({ "kind": "retreat", "actor": actor.value(), "target": target.value() })
    }
    Command::Chase { actor, target } => {
      json!({ "kind": "chase", "actor": actor.value(), "target": target.value() })
    }
    Command::Investigate { actor, position } => json!({
      "kind": "investigate",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Command::Equip { actor, item } => {
      json!({ "kind": "equip", "actor": actor.value(), "item": item.value() })
    }
    Command::Unequip { actor } => json!({ "kind": "unequip", "actor": actor.value() }),
    Command::UseItem { actor, item } => {
      json!({ "kind": "use_item", "actor": actor.value(), "item": item.value() })
    }
    Command::Pickup { actor, item } => {
      json!({ "kind": "pickup", "actor": actor.value(), "item": item.value() })
    }
    Command::Drop { actor, item } => {
      json!({ "kind": "drop", "actor": actor.value(), "item": item.value() })
    }
    Command::Reload { actor } => json!({ "kind": "reload", "actor": actor.value() }),
  }
}

/// Returns the journal name for a canonical run outcome.
#[must_use]
pub const fn outcome_name(outcome: RunOutcome) -> &'static str {
  match outcome {
    RunOutcome::InProgress => "in_progress",
    RunOutcome::Defeat => "defeat",
    RunOutcome::Victory => "victory",
  }
}

const fn direction_name(direction: Direction) -> &'static str {
  match direction {
    Direction::North => "north",
    Direction::South => "south",
    Direction::West => "west",
    Direction::East => "east",
  }
}

fn position_value(position: Position) -> Value {
  json!({ "x": position.x(), "y": position.y() })
}
