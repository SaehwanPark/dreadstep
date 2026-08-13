//! Keyboard intent and the controlled-actor input resource.

use bevy::ecs::resource::Resource;
use bevy::ecs::schedule::SystemSet;
use bevy::input::keyboard::KeyCode;
use dreadstep_core::{ActorId, Command, Direction};

/// A supported keyboard intent before it is addressed to one core actor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyboardIntent {
  /// Move one tile in a cardinal direction.
  Move(Direction),
  /// Spend one standard action without moving.
  Wait,
}

/// Selects which presentation boundary owns keyboard command submission.
///
/// The default headless behavior remains active when this resource is absent. A desktop client
/// inserts [`Self::External`] so it can select item/combat commands and journal every attempt
/// before the projection plugin runs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub enum PresentationKeyboardMode {
  /// Preserve the original move/wait dispatcher owned by [`crate::PresentationPlugin`].
  #[default]
  BuiltIn,
  /// Leave keyboard input for an external client driver.
  External,
}

/// Orders the presentation plugin relative to external client systems.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum PresentationSet {
  /// Synchronize authoritative runtime state into disposable ECS projections.
  Projection,
}

impl KeyboardIntent {
  /// Converts supported arrow/WASD and wait keys into presentation intent.
  #[must_use]
  pub const fn from_key(key: KeyCode) -> Option<Self> {
    match key {
      KeyCode::ArrowUp | KeyCode::KeyW => Some(Self::Move(Direction::North)),
      KeyCode::ArrowDown | KeyCode::KeyS => Some(Self::Move(Direction::South)),
      KeyCode::ArrowLeft | KeyCode::KeyA => Some(Self::Move(Direction::West)),
      KeyCode::ArrowRight | KeyCode::KeyD => Some(Self::Move(Direction::East)),
      KeyCode::Enter | KeyCode::Space => Some(Self::Wait),
      _ => None,
    }
  }

  /// Addresses this intent to an explicit actor as a canonical core command.
  #[must_use]
  pub const fn command(self, actor: ActorId) -> Command {
    match self {
      Self::Move(direction) => Command::Move { actor, direction },
      Self::Wait => Command::Wait { actor },
    }
  }
}

/// Selects the core actor addressed by keyboard intents.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationInput {
  pub(crate) actor: ActorId,
}

impl PresentationInput {
  /// Creates keyboard control for one explicit actor identity.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self { actor }
  }

  /// Returns the actor addressed by keyboard intents.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }
}
