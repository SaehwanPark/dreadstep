//! Pure NetHack-style frame layout. No terminal I/O lives here.

use dreadstep_core::{ActorKind, RunOutcome, StatusKind};

use crate::glyphs::{
  Cell, CellColor, actor_cell, behavior_name, ground_item_cell, push_styled, tile_cell, unseen_cell,
};
use crate::input::{Overlay, UiState};
use crate::inventory::{equipment_state_label, inventory_overlay_lines};
use crate::item_labels::item_kind_label_and_color;
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
  lines.push(header_line(session));
  for index in 0..MESSAGE_WINDOW_LINES {
    let message = ui.messages().get(index).map_or("", String::as_str);
    lines.push(message_line(message));
  }

  // Separation between message window and dungeon rendering / overlay.
  lines.push(plain_line(""));

  match ui.overlay() {
    Overlay::None => lines.extend(map_lines(session)),
    Overlay::Help => lines.extend(help_overlay_lines()),
    Overlay::Inventory => lines.extend(inventory_overlay_lines(session, ui)),
  }

  // Separation between dungeon rendering / overlay and status section.
  lines.push(plain_line(""));

  lines.extend(status_lines(session, ui));

  // Separation between status section and intent/controls section.
  lines.push(plain_line(""));

  lines.extend(intent_and_controls_lines(session));

  TextFrame { lines }
}

fn header_line(session: &Session) -> Vec<Cell> {
  let mut cells = Vec::new();
  push_styled(&mut cells, "Dreadstep", CellColor::WhiteBold);
  push_styled(&mut cells, "  seed:", CellColor::Gray);
  push_styled(
    &mut cells,
    &format!("{}", session.seed()),
    CellColor::Yellow,
  );
  push_styled(&mut cells, "  ", CellColor::Default);
  match session.scenario() {
    crate::session::Scenario::ItemShowcase => {
      push_styled(&mut cells, "item_showcase", CellColor::Cyan);
    }
    crate::session::Scenario::Procedural { depth } => {
      push_styled(
        &mut cells,
        &format!("procedural_floor depth {depth}"),
        CellColor::Cyan,
      );
    }
  }
  cells
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
  vec![
    health_line(session),
    ammo_status_outcome_line(session),
    inventory_status_line(session, ui),
  ]
}

fn health_line(session: &Session) -> Vec<Cell> {
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
  let bar = health_bar(hp, max_hp);
  let hp_color = if max_hp == 0 {
    CellColor::Default
  } else if hp * 2 > max_hp {
    CellColor::Green
  } else if hp * 4 > max_hp {
    CellColor::DarkYellow
  } else {
    CellColor::DarkRed
  };

  let mut cells = Vec::new();
  push_styled(&mut cells, &format!("HP:{hp}/{max_hp} {bar}"), hp_color);
  push_styled(&mut cells, "  Pos:(", CellColor::Gray);
  push_styled(&mut cells, &position, CellColor::Default);
  push_styled(&mut cells, ")  T:", CellColor::Gray);
  push_styled(
    &mut cells,
    &format!("{}", session.current_time().value()),
    CellColor::Default,
  );
  push_styled(&mut cells, "  Next:", CellColor::Gray);
  if next == "you" {
    push_styled(&mut cells, &next, CellColor::WhiteBold);
  } else {
    push_styled(&mut cells, &next, CellColor::Red);
  }
  cells
}
fn ammo_status_outcome_line(session: &Session) -> Vec<Cell> {
  let player = session.actor(PLAYER);
  let ammo = player.map_or(0, dreadstep_core::Actor::ranged_ammo);
  let status = player.and_then(dreadstep_core::Actor::status);
  let outcome = session.outcome();

  let mut cells = Vec::new();
  push_styled(&mut cells, "Ammo:", CellColor::Gray);
  let ammo_str = format!("{ammo}/{}", dreadstep_core::Actor::RANGED_AMMO_CAPACITY);
  let ammo_color = if ammo == 0 {
    CellColor::DarkRed
  } else {
    CellColor::Yellow
  };
  push_styled(&mut cells, &ammo_str, ammo_color);

  push_styled(&mut cells, "  Status:", CellColor::Gray);
  match status {
    None => push_styled(&mut cells, "none", CellColor::Default),
    Some(status_val) => match status_val.kind() {
      StatusKind::Chilled => {
        push_styled(
          &mut cells,
          &format!("Chilled {}", status_val.remaining_actions()),
          CellColor::Cyan,
        );
      }
    },
  }

  push_styled(&mut cells, "  Outcome:", CellColor::Gray);
  let outcome_str = outcome_name(outcome);
  let outcome_color = match outcome {
    RunOutcome::InProgress => CellColor::Default,
    RunOutcome::Victory => CellColor::Green,
    RunOutcome::Defeat => CellColor::DarkRed,
  };
  push_styled(&mut cells, outcome_str, outcome_color);

  cells
}

fn inventory_status_line(session: &Session, ui: &UiState) -> Vec<Cell> {
  let mut cells = Vec::new();
  push_styled(&mut cells, "Inv:", CellColor::Gray);
  let Some(player) = session.actor(PLAYER) else {
    return cells;
  };
  if player.inventory().is_empty() {
    push_styled(&mut cells, " (empty)", CellColor::Default);
    return cells;
  }
  for (index, item) in player.inventory().iter().enumerate() {
    if index == 0 {
      push_styled(&mut cells, " ", CellColor::Default);
    } else {
      push_styled(&mut cells, "  ", CellColor::Default);
    }
    let id = item.id().value();
    let is_equipped = player.is_item_equipped(item.id());
    let is_selected = Some(item.id()) == ui.selected_item();

    let (label_text, item_color) = item_kind_label_and_color(item);
    let full_label = format!("{id}) {label_text}");

    let display_color = if is_selected {
      CellColor::WhiteBold
    } else {
      item_color
    };
    push_styled(&mut cells, &full_label, display_color);

    if is_equipped {
      push_styled(&mut cells, equipment_state_label(item), CellColor::Green);
    }
    if is_selected {
      push_styled(&mut cells, "*", CellColor::Yellow);
    }
  }
  cells
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
fn message_line(text: &str) -> Vec<Cell> {
  let color = if text.contains("cannot")
    || text.contains("Rejected")
    || text.contains("Unavailable")
    || text.contains("die")
    || text.contains("dies")
  {
    CellColor::DarkRed
  } else if text.contains("trap")
    || text.contains("Trap")
    || text.contains("springs")
    || text.contains("hit")
    || text.contains("hits")
    || text.contains("damage")
  {
    CellColor::DarkYellow
  } else if text.contains("chilled")
    || text.contains("Chilled")
    || text.contains("chill")
    || text.contains("Chill")
    || text.contains("frost")
    || text.contains("Frost")
  {
    CellColor::Cyan
  } else if text.contains("noise")
    || text.contains("echoes")
    || text.contains("kick")
    || text.contains("smash")
    || text.contains("obstacle")
  {
    CellColor::Yellow
  } else if text.contains("open")
    || text.contains("close")
    || text.contains("pick up")
    || text.contains("drop")
    || text.contains("wield")
    || text.contains("unwield")
    || text.contains("use")
    || text.contains("reload")
  {
    CellColor::Green
  } else {
    CellColor::Default
  };
  if text.is_empty() {
    return vec![Cell::plain(' ')];
  }
  text.chars().map(|glyph| Cell::new(glyph, color)).collect()
}

fn intent_and_controls_lines(session: &Session) -> Vec<Vec<Cell>> {
  let mut lines = vec![intent_line(session), controls_line(session)];
  match session.outcome() {
    RunOutcome::Victory => lines.push(victory_hint_line(session)),
    RunOutcome::Defeat => lines.push(defeat_line()),
    RunOutcome::InProgress => {}
  }
  lines
}

fn intent_line(session: &Session) -> Vec<Cell> {
  let mut cells = Vec::new();
  push_styled(&mut cells, "Intent:", CellColor::Gray);
  let Some(next) = session.next_actor() else {
    push_styled(&mut cells, " -", CellColor::Default);
    return cells;
  };
  if next == PLAYER {
    push_styled(&mut cells, " (your turn)", CellColor::Green);
    return cells;
  }
  let Some(actor) = session.actor(next) else {
    push_styled(&mut cells, " -", CellColor::Default);
    return cells;
  };
  if actor.kind() != ActorKind::Enemy {
    push_styled(&mut cells, " -", CellColor::Default);
    return cells;
  }
  push_styled(&mut cells, " ", CellColor::Default);
  let behavior = actor.enemy_behavior();
  let b_name = behavior_name(behavior);
  let b_color = match behavior {
    dreadstep_core::EnemyBehavior::Frostcaster => CellColor::Cyan,
    dreadstep_core::EnemyBehavior::Pursuer
    | dreadstep_core::EnemyBehavior::Brute
    | dreadstep_core::EnemyBehavior::Zombie => CellColor::Red,
    dreadstep_core::EnemyBehavior::Kiter | dreadstep_core::EnemyBehavior::Scavenger => {
      CellColor::DarkYellow
    }
    dreadstep_core::EnemyBehavior::Blocker => CellColor::Magenta,
  };
  push_styled(&mut cells, b_name, b_color);
  if let Some(command) = session.preferred_enemy_command(next, PLAYER) {
    push_styled(&mut cells, " ", CellColor::Default);
    push_styled(&mut cells, command_name(command), CellColor::Yellow);
  }
  cells
}

fn controls_line(session: &Session) -> Vec<Cell> {
  let text = match (session.outcome(), session.scenario()) {
    (RunOutcome::Victory, crate::session::Scenario::Procedural { .. }) => {
      "R restart  N next depth (procedural)  Esc quit  ? help"
    }
    (RunOutcome::Victory, crate::session::Scenario::ItemShowcase) | (RunOutcome::Defeat, _) => {
      "R restart  Esc quit  ? help"
    }
    (RunOutcome::InProgress, _) => {
      "hjkl/WASD move  . wait  o open  c close  , pickup  i inv  ? help  Esc quit"
    }
  };
  let mut cells = Vec::new();
  let pairs = text.split("  ");
  for (i, pair) in pairs.enumerate() {
    if i > 0 {
      push_styled(&mut cells, "  ", CellColor::Default);
    }
    if let Some((key, desc)) = pair.split_once(' ') {
      push_styled(&mut cells, key, CellColor::Yellow);
      push_styled(&mut cells, " ", CellColor::Default);
      push_styled(&mut cells, desc, CellColor::Gray);
    } else {
      push_styled(&mut cells, pair, CellColor::Default);
    }
  }
  cells
}

fn victory_hint_line(session: &Session) -> Vec<Cell> {
  let text = match session.scenario() {
    crate::session::Scenario::Procedural { .. } => {
      "Showcase complete. Press N for the next depth, R to restart."
    }
    crate::session::Scenario::ItemShowcase => "Showcase complete. Press R to restart, Esc to quit.",
  };
  let mut cells = Vec::new();
  push_styled(&mut cells, text, CellColor::Green);
  cells
}

fn defeat_line() -> Vec<Cell> {
  let mut cells = Vec::new();
  push_styled(
    &mut cells,
    "You die. Press R to restart, Esc to quit.",
    CellColor::DarkRed,
  );
  cells
}

fn help_overlay_lines() -> Vec<Vec<Cell>> {
  HELP_TEXT
    .lines()
    .enumerate()
    .map(|(index, line)| {
      let mut cells = Vec::new();
      if index == 0 {
        push_styled(&mut cells, line, CellColor::WhiteBold);
      } else if let Some((key_part, desc_part)) = line.split_once("   ") {
        push_styled(&mut cells, key_part, CellColor::Yellow);
        push_styled(&mut cells, "   ", CellColor::Default);
        push_styled(&mut cells, desc_part, CellColor::Default);
      } else {
        push_styled(&mut cells, line, CellColor::Default);
      }
      cells
    })
    .collect()
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

fn plain_line(text: &str) -> Vec<Cell> {
  if text.is_empty() {
    return vec![Cell::plain(' ')];
  }
  text.chars().map(Cell::plain).collect()
}

#[cfg(test)]
mod tests {
  use super::{MESSAGE_WINDOW_LINES, render_frame};
  use crate::glyphs::CellColor;
  use crate::input::{Overlay, UiState};
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
  fn frame_has_empty_lines_separating_sections() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    let frame = render_frame(&session, &ui);
    let plain_lines: Vec<String> = frame
      .lines()
      .iter()
      .map(|line| {
        line
          .iter()
          .map(|c| c.glyph())
          .collect::<String>()
          .trim_end()
          .to_string()
      })
      .collect();

    // Line 0: Header
    // Lines 1..=8: Message window (8 lines)
    // Line 9 (index 1 + 8 = 9): Empty line separating message window and map
    assert_eq!(plain_lines[1 + MESSAGE_WINDOW_LINES], "");

    // Find the status section starting with "HP:"
    let hp_index = plain_lines
      .iter()
      .position(|line| line.starts_with("HP:"))
      .expect("HP line must exist");

    // There must be an empty line right before the status section (between map and status)
    assert!(hp_index > 0);
    assert_eq!(
      plain_lines[hp_index - 1],
      "",
      "empty line expected before status section"
    );

    // Find the intent line starting with "Intent:"
    let intent_index = plain_lines
      .iter()
      .position(|line| line.starts_with("Intent:"))
      .expect("Intent line must exist");

    // There must be an empty line right before the intent section (between status and intent)
    assert!(intent_index > 0);
    assert_eq!(
      plain_lines[intent_index - 1],
      "",
      "empty line expected before intent section"
    );
  }

  #[test]
  fn frame_lines_have_styled_colors() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    let frame = render_frame(&session, &ui);

    // Header has WhiteBold title
    assert_eq!(frame.lines()[0][0].color(), CellColor::WhiteBold);

    // HP line starts with green when full health
    let hp_line = frame
      .lines()
      .iter()
      .find(|line| {
        let text: String = line.iter().map(|c| c.glyph()).collect();
        text.starts_with("HP:")
      })
      .expect("HP line");
    assert_eq!(hp_line[0].color(), CellColor::Green);
  }

  #[test]
  fn overlays_preserve_section_spacing() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    ui.toggle_inventory();
    assert_eq!(ui.overlay(), Overlay::Inventory);
    let frame = render_frame(&session, &ui);
    let plain_lines: Vec<String> = frame
      .lines()
      .iter()
      .map(|line| {
        line
          .iter()
          .map(|c| c.glyph())
          .collect::<String>()
          .trim_end()
          .to_string()
      })
      .collect();

    let hp_index = plain_lines
      .iter()
      .position(|line| line.starts_with("HP:"))
      .expect("HP line");
    assert_eq!(plain_lines[hp_index - 1], "");
  }

  #[test]
  fn inventory_overlay_compares_selected_item_with_equipped_item() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    ui.toggle_inventory();

    let plain = render_frame(&session, &ui).plain();
    assert!(
      plain.contains("Compare: reach2 vs nothing"),
      "comparison missing:\n{plain}"
    );
  }

  #[test]
  fn inventory_overlay_lists_legal_actions_for_selected_equipment() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    ui.toggle_inventory();

    let plain = render_frame(&session, &ui).plain();
    assert!(
      plain.contains("Actions: e equip, x drop"),
      "selected equipment actions missing:\n{plain}"
    );
  }

  #[test]
  fn inventory_overlay_lists_use_and_drop_for_selected_consumable() {
    let session = Session::start_item_run(7).expect("item showcase");
    let mut ui = UiState::new();
    ui.select_default_item(&session);
    ui.select_inventory(&session, true);
    ui.toggle_inventory();

    let plain = render_frame(&session, &ui).plain();
    assert!(
      plain.contains("Actions: q use, x drop"),
      "selected consumable actions missing:\n{plain}"
    );
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
