//! Process boundary: argument parsing, TTY loop, stdout frames, and capture.

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal::{
  EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{cursor, execute, queue};

use crate::args::{Options, ParseResult, USAGE, parse_options};
use crate::frame::render_frame;
use crate::glyphs::CellColor;
use crate::input::{Intent, Key, Overlay, intent_for_key};
use crate::journal::Journal;
use crate::play::{Play, Status};
use crate::session::{PLAYER, Session};
use crate::smoke::run_smoke;

const ENEMY_DELAY: Duration = Duration::from_millis(150);
const FRAME_SEPARATOR: &str = "----";

/// Runs the terminal client.
///
/// # Errors
///
/// Returns an I/O or parse failure through stderr and a non-zero exit code rather than a
/// library [`Result`], matching the other adapter binaries.
#[must_use]
pub fn run<I, S>(args: I) -> ExitCode
where
  I: IntoIterator<Item = S>,
  S: AsRef<str>,
{
  match parse_options(args) {
    Ok(ParseResult::Help) => {
      print!("{USAGE}");
      ExitCode::SUCCESS
    }
    Ok(ParseResult::Options(options)) => run_options(options),
    Err(error) => {
      eprintln!("error: {error}");
      eprintln!("{USAGE}");
      ExitCode::from(2)
    }
  }
}

fn run_options(options: Options) -> ExitCode {
  let session = if options.procedural {
    Session::start_procedural_run(options.seed, options.depth)
  } else {
    Session::start_item_run(options.seed)
  };
  let session = match session {
    Ok(session) => session,
    Err(error) => {
      eprintln!("error: {error}");
      return ExitCode::from(2);
    }
  };
  let journal = match Journal::open(&options.log_dir) {
    Ok(journal) => journal,
    Err(error) => {
      eprintln!("error: {error}");
      return ExitCode::from(1);
    }
  };
  let mut play = Play::new(session, journal);
  let _ = play.record(
    "run_started",
    serde_json::json!({
      "seed": play.session.seed(),
      "scenario": match play.session.scenario() {
        crate::session::Scenario::ItemShowcase => "item_showcase",
        crate::session::Scenario::Procedural { .. } => "procedural_floor",
      },
      "journal": play.journal_path().display().to_string(),
    }),
  );
  let _ = play.record_frame("startup");
  if options.smoke {
    return run_smoke(play);
  }
  if let Some(directory) = options.capture_dir {
    return capture_screenshots(&mut play, &directory);
  }
  let print_frames = options.print_frames || !io::stdin().is_terminal();
  if print_frames {
    run_print_frames(&mut play, options.no_delay)
  } else {
    run_tty(&mut play, options.no_delay)
  }
}

fn capture_screenshots(play: &mut Play, directory: &Path) -> ExitCode {
  if let Err(error) = std::fs::create_dir_all(directory) {
    eprintln!("error: {error}");
    return ExitCode::from(1);
  }
  if write_capture(directory.join("tui-starter.txt"), &play.frame().plain()).is_err() {
    return ExitCode::from(1);
  }
  let opened = play.submit_command(
    "capture",
    dreadstep_core::Command::Interact {
      actor: PLAYER,
      position: dreadstep_core::Position::new(2, 1),
    },
  );
  if !opened {
    eprintln!("error: capture could not open the starter door");
    return ExitCode::from(1);
  }
  if write_capture(directory.join("tui-status.txt"), &play.frame().plain()).is_err() {
    return ExitCode::from(1);
  }
  play.shutdown("capture")
}

fn write_capture(path: std::path::PathBuf, contents: &str) -> io::Result<()> {
  std::fs::write(path, format!("{contents}\n"))
}

fn run_print_frames(play: &mut Play, _no_delay: bool) -> ExitCode {
  let mut stdout = io::stdout();
  if writeln!(stdout, "{}\n{}", FRAME_SEPARATOR, play.frame().plain()).is_err() {
    return play.shutdown("stdout_fault");
  }
  loop {
    if matches!(play.status, Status::Faulted(_)) {
      return play.shutdown("fault");
    }
    if play.session.next_actor() != Some(PLAYER) && matches!(play.status, Status::Running) {
      let _ = play.drive_enemies(false);
      let _ = writeln!(stdout, "{}\n{}", FRAME_SEPARATOR, play.frame().plain());
      continue;
    }
    let event = match event::read() {
      Ok(event) => event,
      Err(error) => {
        play.fault(error.to_string());
        return play.shutdown("input_fault");
      }
    };
    let Some(key) = key_from_event(&event) else {
      continue;
    };
    if handle_key(play, key) {
      let _ = writeln!(stdout, "{}\n{}", FRAME_SEPARATOR, play.frame().plain());
    }
    if matches!(play.status, Status::Shutdown(_) | Status::Faulted(_)) {
      return play.shutdown("interactive");
    }
  }
}

fn run_tty(play: &mut Play, no_delay: bool) -> ExitCode {
  if let Err(error) = enable_raw_mode() {
    eprintln!("error: {error}");
    return ExitCode::from(1);
  }
  let mut stdout = io::stdout();
  if let Err(error) = execute!(stdout, EnterAlternateScreen, cursor::Hide) {
    let _ = disable_raw_mode();
    eprintln!("error: {error}");
    return ExitCode::from(1);
  }
  let result = tty_loop(play, no_delay, &mut stdout);
  let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
  let _ = disable_raw_mode();
  result
}

fn tty_loop(play: &mut Play, no_delay: bool, stdout: &mut io::Stdout) -> ExitCode {
  if draw_tty(stdout, play).is_err() {
    return play.shutdown("draw_fault");
  }
  loop {
    if matches!(play.status, Status::Faulted(_)) {
      return play.shutdown("fault");
    }
    if play.session.next_actor() != Some(PLAYER) && matches!(play.status, Status::Running) {
      if no_delay {
        let _ = play.drive_enemies(false);
        let _ = draw_tty(stdout, play);
        continue;
      }
      match event::poll(ENEMY_DELAY) {
        Ok(false) => {
          let _ = play.drive_enemies(false);
          let _ = draw_tty(stdout, play);
          continue;
        }
        Ok(true) => {}
        Err(error) => {
          play.fault(error.to_string());
          return play.shutdown("input_fault");
        }
      }
    }
    let event = match event::read() {
      Ok(event) => event,
      Err(error) => {
        play.fault(error.to_string());
        return play.shutdown("input_fault");
      }
    };
    let Some(key) = key_from_event(&event) else {
      continue;
    };
    if handle_key(play, key) {
      let _ = draw_tty(stdout, play);
    }
    if matches!(play.status, Status::Shutdown(_) | Status::Faulted(_)) {
      return play.shutdown("interactive");
    }
  }
}

fn handle_key(play: &mut Play, key: Key) -> bool {
  match intent_for_key(key, &play.session, &play.ui) {
    Intent::Shutdown => {
      play.status = Status::Shutdown("escape".to_string());
      true
    }
    Intent::CloseOverlay => {
      play.ui.close_overlay();
      let _ = play.record_frame("overlay");
      true
    }
    Intent::ToggleHelp => {
      play.ui.toggle_help();
      let _ = play.record_frame("overlay");
      true
    }
    Intent::ToggleInventory => {
      play.ui.toggle_inventory();
      let _ = play.record_frame("overlay");
      true
    }
    Intent::SelectNextItem => {
      play.ui.select_inventory(&play.session, false);
      let _ = play.record_frame("select");
      true
    }
    Intent::SelectPrevItem => {
      play.ui.select_inventory(&play.session, true);
      let _ = play.record_frame("select");
      true
    }
    Intent::Restart => play.restart(),
    Intent::NextFloor => play.advance_floor(),
    Intent::DiagonalRefused => {
      play
        .ui
        .push_message("You cannot move diagonally.".to_string());
      let _ = play.record_frame("diagonal_refused");
      true
    }
    Intent::Unavailable(label) => {
      if play.ui.overlay() != Overlay::None {
        play.ui.close_overlay();
        return true;
      }
      if !matches!(play.status, Status::Running) {
        return false;
      }
      if play.session.next_actor() == Some(PLAYER) {
        play.ui.push_message(format!("You cannot {label} now."));
      } else {
        play
          .ui
          .push_message("Unavailable input (enemy scheduled).".to_string());
      }
      let _ = play.record_frame("unavailable");
      true
    }
    Intent::Command(command) => {
      if play.ui.overlay() != Overlay::None {
        play.ui.close_overlay();
      }
      if !matches!(play.status, Status::Running) {
        return false;
      }
      if play.session.next_actor() != Some(PLAYER) {
        play
          .ui
          .push_message("Unavailable input (enemy scheduled).".to_string());
        return true;
      }
      play.submit_command("player", command);
      true
    }
  }
}

fn key_from_event(event: &Event) -> Option<Key> {
  let Event::Key(KeyEvent {
    code,
    modifiers,
    kind,
    ..
  }) = event
  else {
    return None;
  };
  if !matches!(*kind, KeyEventKind::Press | KeyEventKind::Repeat) {
    return None;
  }
  if modifiers.contains(KeyModifiers::CONTROL) {
    return match code {
      KeyCode::Char('c' | 'C') => Some(Key::Ctrl('c')),
      KeyCode::Char('d' | 'D') => Some(Key::Ctrl('d')),
      _ => None,
    };
  }
  Some(match code {
    KeyCode::Esc => Key::Escape,
    KeyCode::Enter => Key::Enter,
    KeyCode::Up => Key::Up,
    KeyCode::Down => Key::Down,
    KeyCode::Left => Key::Left,
    KeyCode::Right => Key::Right,
    KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => Key::BackTab,
    KeyCode::Tab => Key::Tab,
    KeyCode::BackTab => Key::BackTab,
    KeyCode::Char(character) => Key::Char(*character),
    _ => return None,
  })
}

fn draw_tty(stdout: &mut io::Stdout, play: &Play) -> io::Result<()> {
  let frame = render_frame(&play.session, &play.ui);
  queue!(
    stdout,
    cursor::MoveTo(0, 0),
    crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
  )?;
  for (row, line) in frame.lines().iter().enumerate() {
    let row = u16::try_from(row).unwrap_or(u16::MAX);
    queue!(stdout, cursor::MoveTo(0, row))?;
    for cell in line {
      queue!(
        stdout,
        SetForegroundColor(color_of(cell.color())),
        SetAttribute(if cell.bold() {
          Attribute::Bold
        } else {
          Attribute::Reset
        }),
        crossterm::style::Print(cell.glyph())
      )?;
    }
    queue!(stdout, ResetColor, SetAttribute(Attribute::Reset))?;
  }
  stdout.flush()
}

const fn color_of(color: CellColor) -> Color {
  match color {
    CellColor::Default => Color::Reset,
    CellColor::Gray => Color::Grey,
    CellColor::WhiteBold => Color::White,
    CellColor::Red => Color::Red,
    CellColor::Cyan => Color::Cyan,
    CellColor::Yellow => Color::Yellow,
    CellColor::Magenta => Color::Magenta,
    CellColor::Green => Color::Green,
    CellColor::DarkYellow => Color::DarkYellow,
    CellColor::DarkRed => Color::DarkRed,
  }
}
