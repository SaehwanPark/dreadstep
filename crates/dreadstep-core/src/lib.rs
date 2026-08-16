//! Deterministic game rules and domain state for Dreadstep.
//!
//! This crate owns semantic commands, events, and domain errors. It stays independent of
//! presentation, transport, storage, wall-clock time, and operating-system services so
//! the same rules can serve tests, headless tools, agents, and human-facing clients.

#![forbid(unsafe_code)]

mod actor;
mod command;
mod enemy_behavior;
mod error;
mod event;
mod hit_points;
mod ids;
mod item;
mod map;
mod replay;
mod status;
mod world;

pub use actor::{ActionCost, ActionTime, Actor, ActorKind, Damage, MeleeReach, RunOutcome};
pub use command::Command;
pub use enemy_behavior::EnemyBehavior;
pub use error::{ActionResult, CommandError, WorldError};
pub use event::{BlockReason, Event};
pub use hit_points::HitPoints;
pub use ids::{ActorId, ItemDefinitionId, ItemId};
pub use item::{
  AmmunitionAmount, AmmunitionResult, EquipmentEffect, EquipmentSlot, GroundItemStack,
  HealingAmount, HealingResult, Item, ItemEffect, ItemRarity, ThrowableEffect,
};
pub use map::{Direction, GridMap, MapError, Position, Tile};
pub use replay::{ReplayTrace, StateDigest};
pub use status::{Status, StatusKind};
pub use world::WorldState;
