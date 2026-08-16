//! NetHack-style terminal presentation adapter for Dreadstep.
//!
//! This crate owns glyphs, colors, keybindings, FOV, frame layout, TTY/stdout I/O, and JSONL
//! frame journals. Simulation stays in `dreadstep-core`.

#![forbid(unsafe_code)]

mod app;
mod args;
mod frame;
mod glyphs;
mod input;
mod item_labels;
mod journal;
mod kinds;
mod messages;
mod play;
mod session;
mod smoke;
mod visibility;

pub use app::run;
pub use args::{Options, ParseError, ParseResult, USAGE, parse_options};
pub use frame::{TextFrame, render_frame};
pub use glyphs::{Cell, CellColor};
pub use input::{Intent, Key, Overlay, UiState, intent_for_key};
pub use journal::{Journal, JournalError};
pub use kinds::{SHOWCASE_COMMAND_KINDS, SHOWCASE_EVENT_KINDS, command_name, event_name};
pub use messages::format_event;
pub use session::{PLAYER, Scenario, Session, SessionOutput};
pub use visibility::{FOV_RADIUS, player_can_see, visible_positions};
