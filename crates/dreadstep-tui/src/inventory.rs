//! Inventory-specific frame projections and selected-item action guidance.

use dreadstep_core::{Actor, Command, EquipmentSlot, Item};

use crate::glyphs::{Cell, CellColor, push_styled};
use crate::input::UiState;
use crate::item_labels::item_kind_label_and_color;
use crate::session::{PLAYER, Session};

pub(crate) fn inventory_overlay_lines(session: &Session, ui: &UiState) -> Vec<Vec<Cell>> {
  let mut lines = Vec::new();
  let mut header = Vec::new();
  push_styled(&mut header, "Inventory", CellColor::WhiteBold);
  push_styled(
    &mut header,
    " (Tab cycles, compare, e equip, q use, x drop, i close):",
    CellColor::Gray,
  );
  lines.push(header);

  let Some(player) = session.actor(PLAYER) else {
    let mut row = Vec::new();
    push_styled(&mut row, "No player.", CellColor::Default);
    lines.push(row);
    return lines;
  };
  if player.inventory().is_empty() {
    let mut row = Vec::new();
    push_styled(&mut row, "Your pack is empty.", CellColor::Default);
    lines.push(row);
  } else {
    for item in player.inventory() {
      let mut row = Vec::new();
      let is_selected = Some(item.id()) == ui.selected_item();
      let is_equipped = player.is_item_equipped(item.id());
      if is_selected {
        push_styled(&mut row, "* ", CellColor::Yellow);
      } else {
        push_styled(&mut row, "  ", CellColor::Default);
      }
      let (label_text, item_color) = item_kind_label_and_color(item);
      let id = item.id().value();
      let full_label = format!("{id}) {label_text}");
      let display_color = if is_selected {
        CellColor::WhiteBold
      } else {
        item_color
      };
      push_styled(&mut row, &full_label, display_color);
      if is_equipped {
        push_styled(&mut row, equipment_state_label(item), CellColor::Green);
      }
      lines.push(row);
    }
    lines.push(comparison_line(player, ui));
    lines.push(inventory_action_line(session, ui, player));
  }
  lines
}

fn comparison_line(player: &Actor, ui: &UiState) -> Vec<Cell> {
  let mut comparison = Vec::new();
  push_styled(&mut comparison, "Compare: ", CellColor::Gray);
  let selected = ui
    .selected_item()
    .and_then(|selected| player.inventory().iter().find(|item| item.id() == selected));
  let equipped = player
    .equipped_item()
    .and_then(|equipped| player.inventory().iter().find(|item| item.id() == equipped));
  match (selected, equipped) {
    (Some(selected), Some(equipped)) if selected.id() == equipped.id() => {
      push_styled(
        &mut comparison,
        "selected item is wielded",
        CellColor::Green,
      );
    }
    (Some(selected), Some(equipped)) => {
      let (selected_label, selected_color) = item_kind_label_and_color(selected);
      let (equipped_label, equipped_color) = item_kind_label_and_color(equipped);
      push_styled(&mut comparison, &selected_label, selected_color);
      push_styled(&mut comparison, " vs ", CellColor::Gray);
      push_styled(&mut comparison, &equipped_label, equipped_color);
    }
    (Some(selected), None) => {
      let (selected_label, selected_color) = item_kind_label_and_color(selected);
      push_styled(&mut comparison, &selected_label, selected_color);
      push_styled(&mut comparison, " vs nothing", CellColor::Gray);
    }
    (None, _) => push_styled(&mut comparison, "no selection", CellColor::Default),
  }
  comparison
}

fn inventory_action_line(session: &Session, ui: &UiState, player: &Actor) -> Vec<Cell> {
  let mut line = Vec::new();
  push_styled(&mut line, "Actions: ", CellColor::Gray);
  let Some(selected) = ui
    .selected_item()
    .and_then(|selected| player.inventory().iter().find(|item| item.id() == selected))
  else {
    push_styled(&mut line, "select an item", CellColor::Default);
    return line;
  };

  let legal = session.legal_commands();
  let mut labels = Vec::new();
  if player.is_item_equipped(selected.id())
    && legal
      .iter()
      .any(|command| matches!(command, Command::Unequip { actor: PLAYER }))
  {
    labels.push("T unequip");
  }
  if selected.equipment_effect().is_some()
    && legal.iter().any(|command| {
      matches!(
        command,
        Command::Equip {
          actor: PLAYER,
          item,
        } if *item == selected.id()
      )
    })
  {
    labels.push("e equip");
  }
  if legal.iter().any(|command| {
    matches!(
      command,
      Command::UseItem {
        actor: PLAYER,
        item,
      } if *item == selected.id()
    )
  }) {
    labels.push("q use");
  }
  if legal.iter().any(|command| {
    matches!(
      command,
      Command::Throw {
        actor: PLAYER,
        item,
        ..
      } if *item == selected.id()
    )
  }) {
    labels.push("t throw");
  }
  if legal.iter().any(|command| {
    matches!(
      command,
      Command::Drop {
        actor: PLAYER,
        item,
      } if *item == selected.id()
    )
  }) {
    labels.push("x drop");
  }
  if labels.is_empty() {
    push_styled(&mut line, "none available", CellColor::Default);
  } else {
    push_styled(&mut line, &labels.join(", "), CellColor::Default);
  }
  line
}

pub(crate) fn equipment_state_label(item: &Item) -> &'static str {
  match item.equipment_slot() {
    Some(EquipmentSlot::Weapon) => " (wielded)",
    Some(EquipmentSlot::Armor) => " (worn)",
    None => " (equipped)",
  }
}
