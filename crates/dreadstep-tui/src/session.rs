//! Core-backed terminal session. Glyphs and I/O stay outside this type.

use dreadstep_content::{ContentError, procedural_floor, starter_item_showcase_floor};
use dreadstep_core::{
  Actor, ActorId, Command, CommandError, EnemyBehavior, Event, GridMap, ItemId, Position,
  ReplayTrace, RunOutcome, StateDigest, Tile, WorldError, WorldState,
};

/// The controlled player identity used by the terminal showcase.
pub const PLAYER: ActorId = ActorId::new(1);

/// Authored melee target used by display-free smoke.
pub const ATTACK_TARGET: ActorId = ActorId::new(2);

/// Authored ranged/frostcaster identity used by display-free smoke.
pub const RANGED_TARGET: ActorId = ActorId::new(3);

/// Authored reach-weapon item.
pub const EQUIP_ITEM: ItemId = ItemId::new(103);

/// Authored Frost Flask item.
pub const FROST_FLASK: ItemId = ItemId::new(104);

/// Authored healing consumable.
pub const CONSUME_ITEM: ItemId = ItemId::new(101);

/// Authored ammunition consumable used as the pickup/drop smoke item.
pub const PICKUP_ITEM: ItemId = ItemId::new(102);

/// Scenario identity preserved in journals and status lines.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scenario {
  /// Authored starter-item showcase floor.
  ItemShowcase,
  /// Seeded procedural corridor floor.
  Procedural {
    /// Authored depth passed to content generation.
    depth: u32,
  },
}

/// Accepted command output projected for the terminal adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOutput {
  events: Vec<Event>,
}

impl SessionOutput {
  /// Returns semantic events in execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }
}

/// A deterministic presentation adapter around one core world and replay trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Session {
  seed: u64,
  scenario: Scenario,
  world: WorldState,
  trace: ReplayTrace,
}

impl Session {
  /// Starts the authored item showcase used by the default terminal client.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(
      seed,
      Scenario::ItemShowcase,
      starter_item_showcase_floor()?,
    ))
  }

  /// Starts an opt-in seeded procedural floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when generated content fails core validation.
  pub fn start_procedural_run(seed: u64, depth: u32) -> Result<Self, ContentError> {
    Ok(Self::new(
      seed,
      Scenario::Procedural { depth },
      procedural_floor(seed, depth)?,
    ))
  }

  fn new(seed: u64, scenario: Scenario, world: WorldState) -> Self {
    Self {
      seed,
      scenario,
      world,
      trace: ReplayTrace::new(seed),
    }
  }

  /// Returns the explicit run seed.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns the presentation-owned scenario identity.
  #[must_use]
  pub const fn scenario(&self) -> Scenario {
    self.scenario
  }

  /// Returns the immutable core map.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    self.world.map()
  }

  /// Returns a living or dead actor by identity.
  #[must_use]
  pub fn actor(&self, id: ActorId) -> Option<&Actor> {
    self.world.actor(id)
  }

  /// Returns actor records in stable identity order.
  pub fn actors(&self) -> impl Iterator<Item = &Actor> + '_ {
    self.world.actors()
  }

  /// Returns ground-item stacks in core order.
  #[must_use]
  pub fn ground_items(&self) -> &[dreadstep_core::GroundItemStack] {
    self.world.ground_items()
  }

  /// Returns the canonical run outcome.
  #[must_use]
  pub fn outcome(&self) -> RunOutcome {
    self.world.outcome()
  }

  /// Returns the scheduled actor, if any.
  #[must_use]
  pub fn next_actor(&self) -> Option<ActorId> {
    self.world.next_actor()
  }

  /// Returns the world's current action time.
  #[must_use]
  pub fn current_time(&self) -> dreadstep_core::ActionTime {
    self.world.current_time()
  }

  /// Returns the deterministic world digest.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    self.world.digest()
  }

  /// Returns the digest of accepted commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.trace.digest()
  }

  /// Returns accepted commands in execution order.
  #[must_use]
  pub fn replay_commands(&self) -> &[Command] {
    self.trace.commands()
  }

  /// Returns core's legal command projection.
  #[must_use]
  pub fn legal_commands(&self) -> Vec<Command> {
    self.world.legal_commands()
  }

  /// Returns core's preferred enemy command for a scheduled actor.
  #[must_use]
  pub fn preferred_enemy_command(&self, actor: ActorId, target: ActorId) -> Option<Command> {
    self.world.preferred_enemy_command(actor, target)
  }

  /// Executes one canonical command through core.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is rejected. Rejected commands do not
  /// enter the replay trace.
  pub fn execute(&mut self, command: Command) -> Result<SessionOutput, CommandError> {
    let result = self.world.execute(command)?;
    self.trace.record(command);
    Ok(SessionOutput {
      events: result.events().to_vec(),
    })
  }

  /// Places one tile for a display-free smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::TeleportOutOfBounds`] when the cell is outside the map.
  pub fn prepare_smoke_tile(&mut self, position: Position, tile: Tile) -> Result<(), WorldError> {
    self
      .world
      .set_tile(position, tile)
      .map(|_| ())
      .ok_or(WorldError::TeleportOutOfBounds {
        actor: PLAYER,
        position,
      })
  }

  /// Authors one enemy behavior for a smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the identity is missing or not an enemy.
  pub fn prepare_smoke_behavior(
    &mut self,
    actor: ActorId,
    behavior: EnemyBehavior,
  ) -> Result<(), WorldError> {
    self
      .world
      .set_enemy_behavior(actor, behavior)
      .map(|_| ())
      .ok_or(WorldError::UnknownActor(actor))
  }

  /// Teleports one actor for a smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns a core [`WorldError`] when the destination is invalid.
  pub fn prepare_smoke_teleport(
    &mut self,
    actor: ActorId,
    position: Position,
  ) -> Result<(), WorldError> {
    self.world.teleport(actor, position)
  }

  /// Drops one owned item for the pickup smoke fixture without replay evidence.
  ///
  /// # Errors
  ///
  /// Returns a core [`WorldError`] when the actor or item cannot drop.
  pub fn prepare_smoke_pickup(&mut self, actor: ActorId, item: ItemId) -> Result<(), WorldError> {
    self.world.drop_item(actor, item)
  }
}

#[cfg(test)]
mod tests {
  use super::{PLAYER, Session};
  use dreadstep_core::Command;

  #[test]
  fn item_run_records_accepted_commands_in_replay() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let wait = Command::Wait { actor: PLAYER };
    session.execute(wait).expect("wait is legal at start");
    assert_eq!(session.replay_commands(), &[wait]);
    assert_ne!(session.replay_digest().value(), 0);
  }

  #[test]
  fn rejected_commands_do_not_enter_replay() {
    let mut session = Session::start_item_run(7).expect("item showcase");
    let illegal = Command::Unequip { actor: PLAYER };
    assert!(session.execute(illegal).is_err());
    assert!(session.replay_commands().is_empty());
  }
}
