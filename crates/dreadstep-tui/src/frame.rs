//! Pure NetHack-style frame layout. No terminal I/O lives here.

use dreadstep_core::{
  ActorKind, EquipmentEffect, Item, ItemEffect, RunOutcome, StatusKind, ThrowableEffect,
};

use crate::glyphs::{
  Cell, CellColor, actor_cell, behavior_name, ground_item_cell, tile_cell, unseen_cell,
};
use crate::input::{Overlay, UiState};
use crate::kinds::{command_name, outcome_name};
use crate::session::{PLAYER, Session};
use crate::visibility::{FOV_RADIUS, visible_positions};

/// Maximum message lines retained in the NetHack-style message window.
pub const MESSAGE_WINDOW_LINES: usize = 8;

/// Width of the hit-point bar.
pub const HEALTH_BAR_WIDTH: usize = 10;

/// One rendered terminal frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextFrame {
  lines: Vec<Vec<Cell>>,
}

impl TextFrame {
  /// Returns rows of colored cells.
  #[must_use]
  pub fn lines(&self) -> &[Vec<Cell>] {
    &self.lines
  }

  /// Returns the frame as plain text with no ANSI colors, trailing spaces stripped per line.
  #[must_use]
  pub fn plain(&self) -> String {
    self
      .lines
      .iter()
      .map(|line| {
        line
          .iter()
          .map(|cell| cell.glyph())
          .collect::<String>()
          .trim_end()
          .to_string()
      })
      .collect::<Vec<_>>()
      .join("\n")
  }
}

/// Renders the current session and UI into a deterministic text frame.
#[must_use]
pub fn render_frame(session: &Session, ui: &UiState) -> TextFrame {
  let mut lines = Vec::new();
  lines.push(plain_line(&header(session)));
  for index in 0..MESSAGE_WINDOW_LINES {
    let message = ui.messages().get(index).map_or("", String::as_str);
    lines.push(message_line(message));
  }
  match ui.overlay() {
    Overlay::None => lines.extend(map_lines(session)),
    Overlay::Help => lines.extend(overlay_lines(HELP_TEXT)),
    Overlay::Inventory => lines.extend(overlay_lines(&inventory_overlay(session, ui))),
  }
  lines.extend(status_lines(session, ui));
  TextFrame { lines }
}

fn header(session: &Session) -> String {
  let scenario = match session.scenario() {
    crate::session::Scenario::ItemShowcase => "item_showcase".to_string(),
    crate::session::Scenario::Procedural { depth } => format!("procedural_floor depth {depth}"),
  };
  format!("Dreadstep  seed:{}  {scenario}", session.seed())
}

fn map_lines(session: &Session) -> Vec<Vec<Cell>> {
  let map = session.map();
  let width = usize::try_from(map.width()).unwrap_or_default();
  let height = usize::try_from(map.height()).unwrap_or_default();
  let origin = session.actor(PLAYER).map(dreadstep_core::Actor::position);
  let visible = origin
    .map(|position| visible_positions(map, position, FOV_RADIUS))
    .unwrap_or_default();
  let mut rows = Vec::with_capacity(height);
  for y in 0..height {
    let mut row = Vec::with_capacity(width);
    for x in 0..width {
      let position = dreadstep_core::Position::new(
        i32::try_from(x).unwrap_or_default(),
        i32::try_from(y).unwrap_or_default(),
      );
      if !visible.contains(&position) {
        row.push(unseen_cell());
        continue;
      }
      row.push(cell_at(session, position));
    }
    rows.push(row);
  }
  rows
}

fn cell_at(session: &Session, position: dreadstep_core::Position) -> Cell {
  // Actor iteration is id-ordered, so occupancy must prefer living actors over corpses
  // rather than the first record on the tile.
  if let Some(living) = session
    .actors()
    .find(|actor| actor.is_alive() && actor.position() == position)
  {
    return actor_cell(living);
  }
  if let Some(corpse) = session
    .actors()
    .find(|actor| !actor.is_alive() && actor.position() == position)
  {
    return actor_cell(corpse);
  }
  if session
    .ground_items()
    .iter()
    .any(|stack| stack.position() == position && !stack.items().is_empty())
  {
    return ground_item_cell();
  }
  session
    .map()
    .tile_at(position)
    .map_or_else(unseen_cell, tile_cell)
}

fn status_lines(session: &Session, ui: &UiState) -> Vec<Vec<Cell>> {
  let player = session.actor(PLAYER);
  let hp = player.map_or(0, |actor| actor.hit_points().value());
  let max_hp = player.map_or(0, |actor| actor.max_hit_points().value());
  let position = player.map_or_else(
    || "-,-".to_string(),
    |actor| format!("{},{}", actor.position().x(), actor.position().y()),
  );
  let next = match session.next_actor() {
    Some(id) if id == PLAYER => "you".to_string(),
    Some(id) => format!("{}", id.value()),
    None => "-".to_string(),
  };
  let ammo = player.map_or(0, dreadstep_core::Actor::ranged_ammo);
  let status = player.and_then(dreadstep_core::Actor::status).map_or_else(
    || "none".to_string(),
    |status| match status.kind() {
      StatusKind::Chilled => format!("Chilled {}", status.remaining_actions()),
    },
  );
  let bar = health_bar(hp, max_hp);
  let mut lines = vec![
    health_line(
      &format!(
        "HP:{hp}/{max_hp} {bar}  Pos:({position})  T:{}  Next:{next}",
        session.current_time().value()
      ),
      hp,
      max_hp,
    ),
    plain_line(&format!(
      "Ammo:{ammo}/{}  Status:{status}  Outcome:{}",
      dreadstep_core::Actor::RANGED_AMMO_CAPACITY,
      outcome_name(session.outcome())
    )),
    plain_line(&inventory_status(session, ui)),
    plain_line(&intent_line(session)),
    plain_line(controls_line(session)),
  ];
  match session.outcome() {
    RunOutcome::Victory => lines.push(plain_line(victory_hint(session))),
    RunOutcome::Defeat => lines.push(plain_line("You die. Press R to restart, Esc to quit.")),
    RunOutcome::InProgress => {}
  }
  lines
}

fn health_bar(hp: u16, max_hp: u16) -> String {
  if max_hp == 0 {
    return format!("[{}]", " ".repeat(HEALTH_BAR_WIDTH));
  }
  let filled = usize::from(hp).saturating_mul(HEALTH_BAR_WIDTH) / usize::from(max_hp);
  let filled = filled.min(HEALTH_BAR_WIDTH);
  format!(
    "[{}{}]",
    "#".repeat(filled),
    "-".repeat(HEALTH_BAR_WIDTH.saturating_sub(filled))
  )
}

fn health_line(text: &str, hp: u16, max_hp: u16) -> Vec<Cell> {
  let color = if max_hp == 0 {
    CellColor::Default
  } else if hp * 2 > max_hp {
    CellColor::Green
  } else if hp * 4 > max_hp {
    CellColor::DarkYellow
  } else {
    CellColor::DarkRed
  };
  text.chars().map(|glyph| Cell::new(glyph, color)).collect()
}

fn message_line(text: &str) -> Vec<Cell> {
  let color = if text.contains("cannot") || text.contains("Rejected") || text.contains("die") {
    CellColor::DarkRed
  } else if text.contains("chilled") || text.contains("Chilled") {
    CellColor::Cyan
  } else {
    CellColor::Default
  };
  if text.is_empty() {
    return vec![Cell::plain(' ')];
  }
  text.chars().map(|glyph| Cell::new(glyph, color)).collect()
}

fn inventory_status(session: &Session, ui: &UiState) -> String {
  let Some(player) = session.actor(PLAYER) else {
    return "Inv:".to_string();
  };
  if player.inventory().is_empty() {
    return "Inv: (empty)".to_string();
  }
  let items = player
    .inventory()
    .iter()
    .map(|item| {
      let mut label = item_label(item);
      if Some(item.id()) == player.equipped_item() {
        label.push_str(" (wielded)");
      }
      if Some(item.id()) == ui.selected_item() {
        label.push('*');
      }
      label
    })
    .collect::<Vec<_>>()
    .join("  ");
  format!("Inv: {items}")
}

fn item_label(item: &Item) -> String {
  let id = item.id().value();
  if matches!(item.throwable_effect(), Some(ThrowableEffect::Chill)) {
    return format!("{id}) flask");
  }
  match item.effect() {
    ItemEffect::Heal { amount } => format!("{id}) heal+{}", amount.value()),
    ItemEffect::RestoreAmmunition { amount } => format!("{id}) ammo+{}", amount.value()),
    ItemEffect::None => match item.equipment_effect() {
      Some(EquipmentEffect::MinimumMeleeReach { reach }) => {
        format!("{id}) reach{}", reach.value())
      }
      None => format!("{id}) item"),
    },
  }
}

fn intent_line(session: &Session) -> String {
  let Some(next) = session.next_actor() else {
    return "Intent: -".to_string();
  };
  if next == PLAYER {
    return "Intent: (your turn)".to_string();
  }
  let Some(actor) = session.actor(next) else {
    return "Intent: -".to_string();
  };
  if actor.kind() != ActorKind::Enemy {
    return "Intent: -".to_string();
  }
  let behavior = behavior_name(actor.enemy_behavior());
  match session.preferred_enemy_command(next, PLAYER) {
    Some(command) => format!("Intent: {behavior} {}", command_name(command)),
    None => format!("Intent: {behavior}"),
  }
}

fn controls_line(session: &Session) -> &'static str {
  match (session.outcome(), session.scenario()) {
    (RunOutcome::Victory, crate::session::Scenario::Procedural { .. }) => {
      "R restart  N next depth (procedural)  Esc quit  ? help"
    }
    (RunOutcome::Victory, crate::session::Scenario::ItemShowcase) | (RunOutcome::Defeat, _) => {
      "R restart  Esc quit  ? help"
    }
    (RunOutcome::InProgress, _) => {
      "hjkl/WASD move  . wait  o open  c close  , pickup  i inv  ? help  Esc quit"
    }
  }
}

fn victory_hint(session: &Session) -> &'static str {
  match session.scenario() {
    crate::session::Scenario::Procedural { .. } => {
      "Showcase complete. Press N for the next depth, R to restart."
    }
    crate::session::Scenario::ItemShowcase => "Showcase complete. Press R to restart, Esc to quit.",
  }
}

fn inventory_overlay(session: &Session, ui: &UiState) -> String {
  let mut lines = vec!["Inventory (Tab cycles, e equip, q use, x drop, i close):".to_string()];
  let Some(player) = session.actor(PLAYER) else {
    lines.push("No player.".to_string());
    return lines.join("\n");
  };
  if player.inventory().is_empty() {
    lines.push("Your pack is empty.".to_string());
  } else {
    for item in player.inventory() {
      let marker = if Some(item.id()) == ui.selected_item() {
        "*"
      } else {
        " "
      };
      let wielded = if Some(item.id()) == player.equipped_item() {
        " (wielded)"
      } else {
        ""
      };
      lines.push(format!("{marker} {}{wielded}", item_label(item)));
    }
  }
  lines.join("\n")
}

const HELP_TEXT: &str = "\
Commands (NetHack-inspired; Dreadstep is cardinal-only)
 hjkl or arrows or WASD   move (bump to attack or open)
 yubn                     refused (no diagonals)
 . Space Enter            wait
 o I                      open door
 c                        close door
 Ctrl-d K                 kick door
 S B                      smash breakable
 F A                      melee
 f G                      ranged
 t                        throw selected frost flask
 , p                      pick up
 x D                      drop selected
 e E                      wield/equip selected
 T Q                      unwield
 q U                      quaff/use selected
 r                        reload
 i                        inventory
 Tab / Shift-Tab          cycle items
 R                        restart
 N                        next procedural depth after victory
 ?                        this help
 Esc                      close overlay or quit";

fn overlay_lines(text: &str) -> Vec<Vec<Cell>> {
  text.lines().map(plain_line).collect()
}

fn plain_line(text: &str) -> Vec<Cell> {
  if text.is_empty() {
    return vec![Cell::plain(' ')];
  }
  text.chars().map(Cell::plain).collect()
}

#[cfg(test)]
mod tests {
  use super::render_frame;
  use crate::input::UiState;
  use crate::session::{PLAYER, Session};
  use dreadstep_core::{Command, Position};

  #[test]
  fn starter_frame_shows_player_and_adjacent_door() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    let plain = render_frame(&session, &ui).plain();
    assert!(plain.contains('@'), "player glyph missing:\n{plain}");
    assert!(plain.contains('+'), "door glyph missing:\n{plain}");
    assert!(plain.contains("seed:7"), "header missing:\n{plain}");
  }

  #[test]
  fn opening_the_door_updates_messages_and_glyph() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    let output = session
      .execute(Command::Interact {
        actor: PLAYER,
        position: Position::new(2, 1),
      })
      .expect("open");
    ui.push_message(crate::messages::format_event(&session, output.events()[0]));
    let plain = render_frame(&session, &ui).plain();
    assert!(plain.contains("You open the door."));
    assert!(plain.contains('\''), "open-door glyph missing:\n{plain}");
  }

  #[test]
  fn living_actor_glyph_wins_over_a_lower_id_corpse() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let corpse = dreadstep_core::ActorId::new(2);
    let living = dreadstep_core::ActorId::new(3);
    let position = session.actor(corpse).expect("enemy 2").position();
    session
      .set_hit_points_for_test(corpse, dreadstep_core::HitPoints::new(0))
      .expect("kill lower-id actor");
    session
      .prepare_smoke_teleport(living, position)
      .expect("living actor may occupy a corpse tile");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    let glyph = super::cell_at(&session, position);
    assert_eq!(
      glyph.glyph(),
      'F',
      "living frostcaster must draw over corpse at {position:?}"
    );
  }
}
