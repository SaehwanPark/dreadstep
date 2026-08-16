//! Versioned external representations for Dreadstep domain concepts.
//!
//! Protocol types translate between stable wire or replay formats and the semantic types
//! owned by `dreadstep-core`. Transport-specific behavior belongs in its adapter crate,
//! not here.

#![forbid(unsafe_code)]

mod command;
mod command_error;
mod enemy_behavior;
mod error;
mod event;
mod ids;
mod item;
mod replay;
mod scenario;
mod snapshot;
mod status;

/// Version of the in-memory agent observation projection.
pub const PROTOCOL_VERSION: u16 = 32;

pub use command::{CommandRequest, Direction};
pub use enemy_behavior::EnemyBehavior;
pub use error::{CommandError, MapError, ScenarioError, WorldError};
pub use event::{BlockReason, Damage, Event};
pub use ids::{ActorId, ActorKind, HitPoints, ItemDefinitionId, ItemId, MeleeReach, Position};
pub use item::{
  AmmunitionResult, EquipmentEffect, GroundItemSnapshot, HealingResult, ItemSnapshot,
  ThrowableEffect,
};
pub use replay::{ActionTime, ReplayEvidence, StateDigest};
pub use scenario::{Scenario, ScenarioActor, Tile};
pub use snapshot::{ActorSnapshot, LifeState, RunOutcome, WorldSnapshot};
pub use status::{StatusKind, StatusSnapshot};
