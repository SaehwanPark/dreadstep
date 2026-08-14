//! README screenshot goldens must match the live TUI renderer.

use dreadstep_core::{Command, Position};
use dreadstep_tui::{PLAYER, Session, UiState, format_event, render_frame};

#[test]
fn starter_screenshot_matches_renderer() {
  let session = Session::start_item_run(7).expect("item showcase");
  let mut ui = UiState::new();
  ui.select_default_item(&session);
  let rendered = format!("{}\n", render_frame(&session, &ui).plain());
  let committed = include_str!("../../../screenshots/tui-starter.txt");
  assert_eq!(rendered, committed);
}

#[test]
fn status_screenshot_matches_renderer_after_opening_the_door() {
  let mut session = Session::start_item_run(7).expect("item showcase");
  let mut ui = UiState::new();
  ui.select_default_item(&session);
  let output = session
    .execute(Command::Interact {
      actor: PLAYER,
      position: Position::new(2, 1),
    })
    .expect("open starter door");
  ui.push_message(format_event(&session, output.events()[0]));
  let rendered = format!("{}\n", render_frame(&session, &ui).plain());
  let committed = include_str!("../../../screenshots/tui-status.txt");
  assert_eq!(rendered, committed);
}

#[test]
fn readme_embeds_screenshot_files() {
  let readme = include_str!("../../../README.md");
  let starter = include_str!("../../../screenshots/tui-starter.txt").trim_end();
  let status = include_str!("../../../screenshots/tui-status.txt").trim_end();
  assert!(
    readme.contains(starter),
    "README.md must embed screenshots/tui-starter.txt"
  );
  assert!(
    readme.contains(status),
    "README.md must embed screenshots/tui-status.txt"
  );
}
