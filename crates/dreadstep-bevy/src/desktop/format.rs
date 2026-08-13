//! Journal payloads, HUD text, and event/command formatting.

use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::system::{Query, Res};
use bevy::prelude::Text;
use dreadstep_core::{
  Actor, ActorId, ActorKind, BlockReason, Command, Direction, Event, Item, ItemId, Position,
  RunOutcome, Tile,
};
use serde_json::{Value, json};

use crate::{
  PresentationEnemyIntent, PresentationRuntime, PresentationSnapshot, PresentationVisibility,
  SceneRenderPlaceholder,
};

use super::journal::journal_path;
use super::session::DesktopSession;
use super::session::DesktopStatus;
use super::{HEALTH_BAR_WIDTH, PLAYER, SHOWCASE_MAX_HIT_POINTS};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HudLineKind {
  Stats,
  Inventory,
  Messages,
  Controls,
  Journal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(crate) struct HudLine(pub(crate) HudLineKind);

pub(crate) fn state_payload(runtime: &PresentationRuntime, extra: Value) -> Value {
  let snapshot = runtime.snapshot();
  json!({
    "state": snapshot_value(&snapshot),
    "state_digest": snapshot.digest().value(),
    "replay_digest": runtime.replay_digest().value(),
    "extra": extra,
  })
}

pub(crate) fn snapshot_value(snapshot: &crate::PresentationSnapshot) -> Value {
  let actors = snapshot
    .actors()
    .iter()
    .map(actor_value)
    .collect::<Vec<_>>();
  let ground_items = snapshot
    .ground_items()
    .iter()
    .map(|stack| {
      json!({
        "position": position_value(stack.position()),
        "items": stack.items().iter().copied().map(item_value).collect::<Vec<_>>(),
      })
    })
    .collect::<Vec<_>>();
  json!({
    "map": {
      "width": snapshot.width(),
      "height": snapshot.height(),
      "tiles": snapshot.tiles().iter().copied().map(tile_name).collect::<Vec<_>>(),
    },
    "outcome": outcome_name(snapshot.outcome()),
    "actors": actors,
    "ground_items": ground_items,
    "scheduler": {
      "current_time": snapshot.current_time().value(),
      "next_actor": snapshot.next_actor().map(ActorId::value),
    },
  })
}

pub(crate) fn actor_value(actor: &Actor) -> Value {
  json!({
    "id": actor.id().value(),
    "kind": actor_kind_name(actor.kind()),
    "position": position_value(actor.position()),
    "hit_points": actor.hit_points().value(),
    "melee_reach": actor.melee_reach().value(),
    "ranged_ammo": actor.ranged_ammo(),
    "alive": actor.is_alive(),
    "ready_at": actor.ready_at().value(),
    "equipped": actor.equipped_item().map(ItemId::value),
    "inventory": actor.inventory().iter().copied().map(item_value).collect::<Vec<_>>(),
  })
}

pub(crate) fn item_value(item: Item) -> Value {
  json!({
    "id": item.id().value(),
    "definition": item.definition().value(),
    "equipment_effect": item.equipment_effect().map(|effect| match effect {
      dreadstep_core::EquipmentEffect::MinimumMeleeReach { reach } => {
        json!({ "minimum_melee_reach": reach.value() })
      }
    }),
  })
}

pub(crate) fn position_value(position: Position) -> Value {
  json!({ "x": position.x(), "y": position.y() })
}

pub(crate) fn tile_name(tile: Tile) -> &'static str {
  match tile {
    Tile::Floor => "floor",
    Tile::Cover => "cover",
    Tile::Wall => "wall",
    Tile::Door => "door",
    Tile::Breakable => "breakable",
    Tile::Trap => "trap",
  }
}

pub(crate) fn actor_kind_name(kind: ActorKind) -> &'static str {
  match kind {
    ActorKind::Player => "player",
    ActorKind::Enemy => "enemy",
  }
}

pub(crate) fn outcome_name(outcome: RunOutcome) -> &'static str {
  match outcome {
    RunOutcome::InProgress => "in_progress",
    RunOutcome::Defeat => "defeat",
    RunOutcome::Victory => "victory",
  }
}

pub(crate) fn placeholder_name(placeholder: SceneRenderPlaceholder) -> &'static str {
  match placeholder {
    SceneRenderPlaceholder::Terrain => "terrain",
    SceneRenderPlaceholder::Player => "player",
    SceneRenderPlaceholder::Enemy => "enemy",
    SceneRenderPlaceholder::DeadActor => "dead",
    SceneRenderPlaceholder::GroundItem => "ground_item",
    SceneRenderPlaceholder::InventoryItem => "inventory_item",
  }
}

pub(crate) fn command_name(command: Command) -> &'static str {
  match command {
    Command::Move { .. } => "move",
    Command::Wait { .. } => "wait",
    Command::Interact { .. } => "interact",
    Command::Break { .. } => "break",
    Command::Kick { .. } => "kick",
    Command::Attack { .. } => "attack",
    Command::RangedAttack { .. } => "ranged_attack",
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

pub(crate) fn command_value(command: Command) -> Value {
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
    Command::Attack { actor, target } => {
      json!({ "kind": "attack", "actor": actor.value(), "target": target.value() })
    }
    Command::RangedAttack { actor, target } => {
      json!({ "kind": "ranged_attack", "actor": actor.value(), "target": target.value() })
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

pub(crate) fn direction_name(direction: Direction) -> &'static str {
  match direction {
    Direction::North => "north",
    Direction::South => "south",
    Direction::West => "west",
    Direction::East => "east",
  }
}

pub(crate) fn event_value(event: Event) -> Value {
  match event {
    Event::Moved { actor, from, to } => {
      json!({ "kind": "moved", "actor": actor.value(), "from": position_value(from), "to": position_value(to) })
    }
    Event::MovementBlocked {
      actor,
      from,
      to,
      reason,
    } => json!({
      "kind": "movement_blocked",
      "actor": actor.value(),
      "from": position_value(from),
      "to": position_value(to),
      "reason": block_reason_value(reason),
    }),
    Event::Waited { actor, at } => {
      json!({ "kind": "waited", "actor": actor.value(), "at": at.value() })
    }
    Event::DoorOpened { actor, position } => json!({
      "kind": "door_opened",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Event::BreakableBroken { actor, position } => json!({
      "kind": "breakable_broken",
      "actor": actor.value(),
      "position": position_value(position),
    }),
    Event::NoiseCreated {
      actor,
      position,
      radius,
    } => json!({
      "kind": "noise_created",
      "actor": actor.value(),
      "position": position_value(position),
      "radius": radius,
    }),
    Event::TrapTriggered {
      actor,
      position,
      damage,
      remaining_hit_points,
    } => json!({
      "kind": "trap_triggered",
      "actor": actor.value(),
      "position": position_value(position),
      "damage": damage.value(),
      "remaining_hit_points": remaining_hit_points.value(),
    }),
    Event::Attacked {
      attacker,
      target,
      damage,
      remaining_hit_points,
    } => json!({
      "kind": "attacked",
      "attacker": attacker.value(),
      "target": target.value(),
      "damage": damage.value(),
      "remaining_hit_points": remaining_hit_points.value(),
    }),
    Event::Died { actor } => json!({ "kind": "died", "actor": actor.value() }),
    Event::ItemEquipped { actor, item } => {
      json!({ "kind": "item_equipped", "actor": actor.value(), "item": item.value() })
    }
    Event::ItemUnequipped { actor, item } => {
      json!({ "kind": "item_unequipped", "actor": actor.value(), "item": item.value() })
    }
    Event::ItemConsumed {
      actor,
      item,
      healing,
      ammunition,
    } => {
      let healing = healing.map_or(Value::Null, |result| {
        json!({
          "amount": result.amount(),
          "remaining_hit_points": result.remaining_hit_points().value(),
        })
      });
      let ammunition = ammunition.map_or(Value::Null, |result| {
        json!({
          "amount": result.amount(),
          "remaining_ammunition": result.remaining_ammunition(),
        })
      });
      json!({ "kind": "item_consumed", "actor": actor.value(), "item": item.value(), "healing": healing, "ammunition": ammunition })
    }
    Event::ItemPickedUp { actor, item } => {
      json!({ "kind": "item_picked_up", "actor": actor.value(), "item": item.value() })
    }
    Event::ItemDropped { actor, item } => {
      json!({ "kind": "item_dropped", "actor": actor.value(), "item": item.value() })
    }
    Event::Reloaded { actor, ammunition } => {
      json!({ "kind": "reloaded", "actor": actor.value(), "ammunition": ammunition })
    }
  }
}

pub(crate) fn block_reason_value(reason: BlockReason) -> Value {
  match reason {
    BlockReason::Terrain => json!({ "kind": "terrain" }),
    BlockReason::Actor(actor) => json!({ "kind": "actor", "actor": actor.value() }),
  }
}

pub(crate) fn event_message(event: Event) -> String {
  match event {
    Event::Moved { actor, to, .. } => {
      format!("Actor {} moved to ({}, {}).", actor.value(), to.x(), to.y())
    }
    Event::MovementBlocked { actor, reason, .. } => {
      format!("Actor {} blocked by {:?}.", actor.value(), reason)
    }
    Event::Waited { actor, at } => format!("Actor {} waited at t{}.", actor.value(), at.value()),
    Event::DoorOpened { actor, position } => format!(
      "Actor {} opened the door at ({}, {}).",
      actor.value(),
      position.x(),
      position.y()
    ),
    Event::BreakableBroken { actor, position } => format!(
      "Actor {} broke terrain at ({}, {}).",
      actor.value(),
      position.x(),
      position.y()
    ),
    Event::NoiseCreated {
      actor,
      position,
      radius,
    } => format!(
      "Actor {} created noise at ({}, {}) with radius {}.",
      actor.value(),
      position.x(),
      position.y(),
      radius
    ),
    Event::TrapTriggered {
      actor,
      position,
      damage,
      remaining_hit_points,
    } => format!(
      "Actor {} triggered a trap at ({}, {}) for {} damage ({} HP left).",
      actor.value(),
      position.x(),
      position.y(),
      damage.value(),
      remaining_hit_points.value()
    ),
    Event::Attacked {
      attacker,
      target,
      remaining_hit_points,
      ..
    } => format!(
      "Actor {} hit {} ({} HP left).",
      attacker.value(),
      target.value(),
      remaining_hit_points.value()
    ),
    Event::Died { actor } => format!("Actor {} died.", actor.value()),
    Event::ItemEquipped { actor, item } => {
      format!("Actor {} equipped item {}.", actor.value(), item.value())
    }
    Event::ItemUnequipped { actor, item } => {
      format!("Actor {} unequipped item {}.", actor.value(), item.value())
    }
    Event::ItemConsumed {
      actor,
      item,
      healing,
      ammunition,
    } => {
      if let Some(ammunition) = ammunition {
        format!(
          "Actor {} consumed item {} and restored {} ammunition ({} shots).",
          actor.value(),
          item.value(),
          ammunition.amount(),
          ammunition.remaining_ammunition()
        )
      } else if let Some(healing) = healing {
        format!(
          "Actor {} consumed item {} and restored {} HP ({} HP).",
          actor.value(),
          item.value(),
          healing.amount(),
          healing.remaining_hit_points().value()
        )
      } else {
        format!("Actor {} consumed item {}.", actor.value(), item.value())
      }
    }
    Event::ItemPickedUp { actor, item } => {
      format!("Actor {} picked up item {}.", actor.value(), item.value())
    }
    Event::ItemDropped { actor, item } => {
      format!("Actor {} dropped item {}.", actor.value(), item.value())
    }
    Event::Reloaded { actor, ammunition } => {
      format!("Actor {} reloaded to {} shots.", actor.value(), ammunition)
    }
  }
}

pub(crate) fn health_bar_text(hit_points: i32) -> String {
  let clamped = usize::try_from(hit_points.clamp(0, SHOWCASE_MAX_HIT_POINTS)).unwrap_or_default();
  let maximum = usize::try_from(SHOWCASE_MAX_HIT_POINTS).unwrap_or_default();
  let filled = ((clamped * HEALTH_BAR_WIDTH) + (maximum / 2)) / maximum;
  format!(
    "[{}{}]",
    "#".repeat(filled),
    "-".repeat(HEALTH_BAR_WIDTH - filled)
  )
}

pub(crate) fn visibility_summary_values(active: bool, radius: u32, visible_tiles: usize) -> String {
  if active {
    format!("FOV {visible_tiles} tiles (radius {radius})")
  } else {
    "FOV full map".to_string()
  }
}

pub(crate) fn visibility_summary(visibility: Option<&PresentationVisibility>) -> String {
  visibility.map_or_else(
    || visibility_summary_values(false, 0, 0),
    |visibility| {
      visibility_summary_values(
        visibility.is_active(),
        visibility.radius(),
        visibility.visible_positions().len(),
      )
    },
  )
}

pub(crate) fn enemy_intent_summary(intent: Option<&PresentationEnemyIntent>) -> String {
  let Some(intent) = intent else {
    return "Intent unavailable".to_string();
  };
  match (intent.actor(), intent.command()) {
    (Some(actor), Some(Command::Chase { target, .. })) => {
      format!(
        "Intent: enemy {} chases actor {}",
        actor.value(),
        target.value()
      )
    }
    (Some(actor), Some(Command::Investigate { position, .. })) => format!(
      "Intent: enemy {} investigates noise at ({}, {})",
      actor.value(),
      position.x(),
      position.y()
    ),
    (Some(actor), Some(command)) => format!("Intent: enemy {} {:?}", actor.value(), command),
    _ => "Intent: none".to_string(),
  }
}

pub(crate) fn scenario_label(procedural: bool, depth: u32) -> String {
  if procedural {
    format!("Procedural floor · depth {depth}")
  } else {
    "Starter item floor".to_string()
  }
}

pub(crate) fn terminal_hud_message(status: &DesktopStatus, procedural: bool, depth: u32) -> String {
  match status {
    DesktopStatus::Victory if procedural => match depth.checked_add(1) {
      Some(next_depth) => {
        format!("Floor cleared — press N for depth {next_depth}, or Shift+R to restart")
      }
      None => "Floor cleared — next depth unavailable; press Shift+R to restart".to_string(),
    },
    DesktopStatus::Victory => "Showcase complete — press Shift+R to restart".to_string(),
    DesktopStatus::Defeat => "Showcase failed — press Shift+R to restart".to_string(),
    _ => String::new(),
  }
}

pub(crate) fn controls_text(procedural: bool) -> &'static str {
  if procedural {
    "Arrows/WASD move  Space/Enter wait\nF attack  G ranged  Tab select  E equip  P pickup  X drop\nQ unequip  U consume  R reload  Shift+R restart  N next procedural floor after victory\nEsc/close quit"
  } else {
    "Arrows/WASD move  Space/Enter wait\nF attack  G ranged  Tab select  E equip  P pickup  X drop\nQ unequip  U consume  R reload  Shift+R restart\nEsc/close quit"
  }
}

pub(crate) fn format_hud_stats(
  player: Option<&Actor>,
  snapshot: &PresentationSnapshot,
  status: &DesktopStatus,
  scenario: &str,
  terminal: &str,
  visibility: Option<&PresentationVisibility>,
  intent: Option<&PresentationEnemyIntent>,
) -> String {
  let terminal_line = if terminal.is_empty() {
    String::new()
  } else {
    format!("{terminal}\n")
  };
  let enemies_remaining = snapshot
    .actors()
    .iter()
    .filter(|actor| actor.kind() == ActorKind::Enemy && actor.is_alive())
    .count();
  let Some(player) = player else {
    return format!(
      "{}\nPlayer unavailable\nTurn t={} next={}\nEnemies remaining: {}\n{}\n{}\n{}Status: {:?}",
      scenario,
      snapshot.current_time().value(),
      snapshot
        .next_actor()
        .map_or_else(|| "-".to_string(), |id| id.value().to_string()),
      enemies_remaining,
      visibility_summary(visibility),
      enemy_intent_summary(intent),
      terminal_line,
      status
    );
  };
  let hit_points = i32::from(player.hit_points().value());
  format!(
    "{}\nHP {} {}/{}  pos ({},{})\nTurn t={} next={}  enemies {}\n{}\n{}\n{}Status: {:?}",
    scenario,
    health_bar_text(hit_points),
    hit_points.clamp(0, SHOWCASE_MAX_HIT_POINTS),
    SHOWCASE_MAX_HIT_POINTS,
    player.position().x(),
    player.position().y(),
    snapshot.current_time().value(),
    snapshot
      .next_actor()
      .map_or_else(|| "-".to_string(), |id| id.value().to_string()),
    enemies_remaining,
    visibility_summary(visibility),
    enemy_intent_summary(intent),
    terminal_line,
    status
  )
}

pub(crate) fn desktop_update_hud(
  runtime: Option<Res<PresentationRuntime>>,
  session: Option<Res<DesktopSession>>,
  visibility: Option<Res<PresentationVisibility>>,
  intent: Option<Res<PresentationEnemyIntent>>,
  mut lines: Query<(&mut Text, &HudLine), With<HudLine>>,
) {
  let Some(runtime) = runtime else { return };
  let Some(session) = session else { return };
  let snapshot = runtime.snapshot();
  let player = snapshot.actors().iter().find(|actor| actor.id() == PLAYER);
  let stats = format_hud_stats(
    player,
    &snapshot,
    &session.status,
    &scenario_label(session.procedural, session.depth),
    &terminal_hud_message(&session.status, session.procedural, session.depth),
    visibility.as_deref(),
    intent.as_deref(),
  );
  let inventory = player.map_or_else(
    || "Inventory unavailable".to_string(),
    |player| {
      let items = player
        .inventory()
        .iter()
        .map(|item| {
          let selected = session.selected_item == Some(item.id());
          let equipped = player.equipped_item() == Some(item.id());
          format!(
            "{}item {} (def {}){}{}",
            if selected { "> " } else { "  " },
            item.id().value(),
            item.definition().value(),
            item
              .equipment_effect()
              .map_or_else(String::new, |effect| match effect {
                dreadstep_core::EquipmentEffect::MinimumMeleeReach { reach } => {
                  format!(" [reach {}]", reach.value())
                }
              },),
            if equipped { " [equipped]" } else { "" }
          )
        })
        .collect::<Vec<_>>();
      if items.is_empty() {
        "(empty)".to_string()
      } else {
        items.join("\n")
      }
    },
  );
  let messages = if session.messages.is_empty() {
    "(no events yet)".to_string()
  } else {
    session
      .messages
      .iter()
      .cloned()
      .collect::<Vec<_>>()
      .join("\n")
  };
  let controls = controls_text(session.procedural);
  let journal = format!(
    "{}\nseed {}",
    journal_path(&session.journal).display(),
    session.seed
  );
  for (mut text, line) in &mut lines {
    let value = match line.0 {
      HudLineKind::Stats => &stats,
      HudLineKind::Inventory => &inventory,
      HudLineKind::Messages => &messages,
      HudLineKind::Controls => controls,
      HudLineKind::Journal => &journal,
    };
    *text = Text::new(value);
  }
}

/// The exhaustive formatter is public for integration tests and future coverage checks.
#[must_use]
pub fn event_kind(event: Event) -> &'static str {
  crate::showcase_event_name(event)
}
