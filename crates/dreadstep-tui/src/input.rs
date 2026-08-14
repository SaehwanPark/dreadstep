//! Terminal key mapping onto core commands and presentation-only overlays.

use dreadstep_core::{ActorKind, Command, Direction, ItemId, Tile};

use crate::session::{PLAYER, Session};

/// Overlay windows that replace the map region without mutating core.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Overlay {
  /// No overlay; the dungeon map is visible.
  #[default]
  None,
  /// NetHack-style help listing.
  Help,
  /// Inventory listing with the current selection.
  Inventory,
}

/// Disposable presentation state for selection, messages, and overlays.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiState {
  messages: Vec<String>,
  selected_item: Option<ItemId>,
  overlay: Overlay,
}

impl Default for UiState {
  fn default() -> Self {
    Self::new()
  }
}

impl UiState {
  /// Creates empty UI state.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      messages: Vec::new(),
      selected_item: None,
      overlay: Overlay::None,
    }
  }

  /// Seeds inventory selection from the player's current items.
  pub fn select_default_item(&mut self, session: &Session) {
    let Some(player) = session.actor(PLAYER) else {
      self.selected_item = None;
      return;
    };
    self.selected_item = player
      .inventory()
      .iter()
      .find(|item| item.id() == crate::session::EQUIP_ITEM)
      .or_else(|| player.inventory().first())
      .map(|item| item.id());
  }

  /// Returns the last eight message lines, oldest first.
  #[must_use]
  pub fn messages(&self) -> &[String] {
    &self.messages
  }

  /// Returns the currently selected inventory item.
  #[must_use]
  pub const fn selected_item(&self) -> Option<ItemId> {
    self.selected_item
  }

  /// Returns the active overlay.
  #[must_use]
  pub const fn overlay(&self) -> Overlay {
    self.overlay
  }

  /// Pushes a message, retaining at most eight lines.
  pub fn push_message(&mut self, line: impl Into<String>) {
    self.messages.push(line.into());
    let excess = self.messages.len().saturating_sub(8);
    if excess > 0 {
      self.messages.drain(..excess);
    }
  }

  /// Clears the message buffer, used on restart.
  pub fn clear_messages(&mut self) {
    self.messages.clear();
  }

  /// Cycles inventory selection.
  pub fn select_inventory(&mut self, session: &Session, reverse: bool) {
    let Some(player) = session.actor(PLAYER) else {
      self.selected_item = None;
      return;
    };
    let items = player.inventory();
    if items.is_empty() {
      self.selected_item = None;
      return;
    }
    let current = self
      .selected_item
      .and_then(|selected| items.iter().position(|item| item.id() == selected));
    let index = match (current, reverse) {
      (Some(index), false) => (index + 1) % items.len(),
      (Some(index), true) => (index + items.len() - 1) % items.len(),
      (None, _) => 0,
    };
    self.selected_item = items.get(index).map(|item| item.id());
  }

  /// Toggles the help overlay, closing inventory if needed.
  pub fn toggle_help(&mut self) {
    self.overlay = if matches!(self.overlay, Overlay::Help) {
      Overlay::None
    } else {
      Overlay::Help
    };
  }

  /// Toggles the inventory overlay, closing help if needed.
  pub fn toggle_inventory(&mut self) {
    self.overlay = if matches!(self.overlay, Overlay::Inventory) {
      Overlay::None
    } else {
      Overlay::Inventory
    };
  }

  /// Closes any overlay.
  pub fn close_overlay(&mut self) {
    self.overlay = Overlay::None;
  }
}

/// A decoded key independent of the terminal crate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
  /// A printable character without Control.
  Char(char),
  /// A Control-modified character such as Ctrl-d.
  Ctrl(char),
  /// Arrow up.
  Up,
  /// Arrow down.
  Down,
  /// Arrow left.
  Left,
  /// Arrow right.
  Right,
  /// Enter / Return.
  Enter,
  /// Tab.
  Tab,
  /// Shift-Tab.
  BackTab,
  /// Escape.
  Escape,
}

/// Presentation-only intents that are not core commands.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Intent {
  /// Submit this legal core command.
  Command(Command),
  /// `NetHack` diagonal keys are refused; Dreadstep is cardinal-only.
  DiagonalRefused,
  /// Cycle inventory forward.
  SelectNextItem,
  /// Cycle inventory backward.
  SelectPrevItem,
  /// Toggle help overlay.
  ToggleHelp,
  /// Toggle inventory overlay.
  ToggleInventory,
  /// Restart the same seed and scenario.
  Restart,
  /// Advance a procedural run after victory.
  NextFloor,
  /// Shut down the process.
  Shutdown,
  /// Close the current overlay.
  CloseOverlay,
  /// No mapping or the requested command is not currently legal.
  Unavailable(&'static str),
}

/// Maps one key onto a presentation intent using legal core commands.
#[must_use]
pub fn intent_for_key(key: Key, session: &Session, ui: &UiState) -> Intent {
  match key {
    Key::Escape => {
      if ui.overlay() == Overlay::None {
        Intent::Shutdown
      } else {
        Intent::CloseOverlay
      }
    }
    Key::Ctrl('c') => Intent::Shutdown,
    Key::Char('?') => Intent::ToggleHelp,
    Key::Char('i') => Intent::ToggleInventory,
    Key::Tab => Intent::SelectNextItem,
    Key::BackTab => Intent::SelectPrevItem,
    Key::Char('R') => Intent::Restart,
    Key::Char('N') => Intent::NextFloor,
    Key::Char('y' | 'u' | 'b' | 'n') => Intent::DiagonalRefused,
    other => match command_for_key(other, session, ui) {
      Some(command) => Intent::Command(command),
      None => Intent::Unavailable(describe_key(other)),
    },
  }
}

fn describe_key(key: Key) -> &'static str {
  match key {
    Key::Char('h' | 'a') | Key::Left => "west",
    Key::Char('j' | 's') | Key::Down => "south",
    Key::Char('k' | 'w') | Key::Up => "north",
    Key::Char('l' | 'd') | Key::Right => "east",
    Key::Char('.' | ' ') | Key::Enter => "wait",
    _ => "that command",
  }
}

fn command_for_key(key: Key, session: &Session, ui: &UiState) -> Option<Command> {
  let legal = session.legal_commands();
  let candidate = match key {
    Key::Char('h' | 'a') | Key::Left => bump_or_move(session, &legal, Direction::West),
    Key::Char('j' | 's') | Key::Down => bump_or_move(session, &legal, Direction::South),
    Key::Char('k' | 'w') | Key::Up => bump_or_move(session, &legal, Direction::North),
    Key::Char('l' | 'd') | Key::Right => bump_or_move(session, &legal, Direction::East),
    Key::Char('.' | ' ') | Key::Enter => Some(Command::Wait { actor: PLAYER }),
    Key::Char(',' | 'p' | 'P') => legal
      .iter()
      .filter_map(|command| match command {
        Command::Pickup { item, .. } => Some((*item, *command)),
        _ => None,
      })
      .min_by_key(|(item, _)| *item)
      .map(|(_, command)| command),
    // WASD claims lowercase d/s/w/a for movement; NetHack drop/smash/wield/attack use
    // these shifted or Bevy-alias keys so both vocabularies remain reachable.
    Key::Char('x' | 'X' | 'D') => ui.selected_item().and_then(|item| {
      legal.iter().copied().find(|command| {
        matches!(
          command,
          Command::Drop {
            actor: PLAYER,
            item: candidate,
          } if *candidate == item
        )
      })
    }),
    Key::Char('o' | 'I') => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Interact { actor: PLAYER, .. })),
    Key::Char('c') => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Close { actor: PLAYER, .. })),
    Key::Ctrl('d') | Key::Char('K') => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Kick { actor: PLAYER, .. })),
    Key::Char('S' | 'B') => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Break { actor: PLAYER, .. })),
    Key::Char('A' | 'F') => legal
      .iter()
      .filter_map(|command| match command {
        Command::Attack { target, .. } => Some((*target, *command)),
        _ => None,
      })
      .min_by_key(|(target, _)| *target)
      .map(|(_, command)| command),
    Key::Char('f' | 'G') => legal
      .iter()
      .filter_map(|command| match command {
        Command::RangedAttack { target, .. } => Some((*target, *command)),
        _ => None,
      })
      .min_by_key(|(target, _)| *target)
      .map(|(_, command)| command),
    Key::Char('t') => ui.selected_item().and_then(|item| {
      legal
        .iter()
        .filter_map(|command| match command {
          Command::Throw {
            item: candidate,
            target,
            ..
          } if *candidate == item => Some((*target, *command)),
          _ => None,
        })
        .min_by_key(|(target, _)| *target)
        .map(|(_, command)| command)
    }),
    Key::Char('e' | 'E') => ui.selected_item().map(|item| Command::Equip {
      actor: PLAYER,
      item,
    }),
    Key::Char('T' | 'Q') => Some(Command::Unequip { actor: PLAYER }),
    Key::Char('q' | 'U') => ui.selected_item().map(|item| Command::UseItem {
      actor: PLAYER,
      item,
    }),
    Key::Char('r') => legal
      .iter()
      .copied()
      .find(|command| matches!(command, Command::Reload { actor: PLAYER })),
    _ => None,
  }?;
  legal.into_iter().find(|command| *command == candidate)
}

fn bump_or_move(session: &Session, legal: &[Command], direction: Direction) -> Option<Command> {
  let player = session.actor(PLAYER)?;
  let destination = player.position().translated(direction);
  if let Some(enemy) = session.actors().find(|actor| {
    actor.is_alive() && actor.kind() == ActorKind::Enemy && actor.position() == destination
  }) {
    let target = enemy.id();
    if let Some(attack) = legal.iter().copied().find(|command| {
      matches!(
        command,
        Command::Attack {
          actor: PLAYER,
          target: candidate,
        } if *candidate == target
      )
    }) {
      return Some(attack);
    }
  }
  if session.map().tile_at(destination) == Some(Tile::Door)
    && let Some(interact) = legal.iter().copied().find(|command| {
      matches!(
        command,
        Command::Interact {
          actor: PLAYER,
          position,
        } if *position == destination
      )
    })
  {
    return Some(interact);
  }
  legal.iter().copied().find(|command| {
    matches!(
      command,
      Command::Move {
        actor: PLAYER,
        direction: candidate,
      } if *candidate == direction
    )
  })
}

#[cfg(test)]
mod tests {
  use super::{Intent, Key, UiState, intent_for_key};
  use crate::session::{PLAYER, Session};
  use dreadstep_core::{Command, Direction, Position};

  #[test]
  fn hjkl_selects_cardinal_moves_or_bump_open() {
    let session = Session::start_item_run(7).expect("item showcase");
    let ui = UiState::new();
    let east = intent_for_key(Key::Char('l'), &session, &ui);
    assert_eq!(
      east,
      Intent::Command(Command::Interact {
        actor: PLAYER,
        position: Position::new(2, 1),
      })
    );
    let west = intent_for_key(Key::Char('h'), &session, &ui);
    assert_eq!(
      west,
      Intent::Command(Command::Move {
        actor: PLAYER,
        direction: Direction::West,
      })
    );
  }

  #[test]
  fn diagonal_keys_are_refused_without_core_calls() {
    let session = Session::start_item_run(7).expect("item showcase");
    let ui = UiState::new();
    assert_eq!(
      intent_for_key(Key::Char('y'), &session, &ui),
      Intent::DiagonalRefused
    );
    assert_eq!(
      intent_for_key(Key::Char('n'), &session, &ui),
      Intent::DiagonalRefused
    );
  }

  #[test]
  fn period_waits() {
    let session = Session::start_item_run(7).expect("item showcase");
    let ui = UiState::new();
    assert_eq!(
      intent_for_key(Key::Char('.'), &session, &ui),
      Intent::Command(Command::Wait { actor: PLAYER })
    );
  }
}
