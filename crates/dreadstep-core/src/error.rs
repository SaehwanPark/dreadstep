//! Structured world-construction and command-rejection errors.
//!
//! Recoverable failures stay typed so adapters can project them without guessing strings.

use std::fmt;

use crate::{ActionTime, ActorId, Event, ItemId, Position};

/// Errors produced while constructing or explicitly mutating a world state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
  /// A tester mutation addresses no actor in the world.
  UnknownActor(ActorId),
  /// An item identity is already owned by an actor in the world.
  DuplicateItemId(ItemId),
  /// An actor already carries the fixed maximum number of item instances.
  InventoryFull(ActorId),
  /// An actor does not own the item requested by a tester transfer.
  ItemNotOwned {
    /// The actor whose inventory was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// An equipped item cannot be moved by a tester inventory mutation.
  ItemEquipped {
    /// The actor whose equipment references the item.
    actor: ActorId,
    /// The equipped item identity.
    item: ItemId,
  },
  /// An actor has no matching item in the ground stack at its current position.
  ItemNotOnGround {
    /// The actor whose current ground stack was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// A tester teleport destination is outside the map.
  TeleportOutOfBounds {
    /// The actor being teleported.
    actor: ActorId,
    /// The invalid destination.
    position: Position,
  },
  /// A tester teleport destination is blocking terrain.
  TeleportOnBlockedTile {
    /// The actor being teleported.
    actor: ActorId,
    /// The blocked destination.
    position: Position,
  },
  /// A living tester teleport destination is occupied by another living actor.
  TeleportOccupied {
    /// The actor being teleported.
    actor: ActorId,
    /// The living actor already at the destination.
    blocker: ActorId,
    /// The occupied destination.
    position: Position,
  },
  /// Two actors use the same stable identity.
  DuplicateActorId(ActorId),
  /// An actor starts outside the map.
  ActorOutOfBounds {
    /// The actor outside the map.
    actor: ActorId,
    /// The invalid starting position.
    position: Position,
  },
  /// An actor starts on a blocking terrain tile.
  ActorOnBlockedTile {
    /// The actor on blocked terrain.
    actor: ActorId,
    /// The invalid starting position.
    position: Position,
  },
  /// Two distinct actors start on one position.
  OverlappingActors {
    /// The actor inserted first.
    first: ActorId,
    /// The actor that overlaps the first actor.
    second: ActorId,
    /// The shared position.
    position: Position,
  },
  /// An actor starts with zero hit points.
  ActorDeadAtStart {
    /// The actor that starts dead.
    actor: ActorId,
  },
}

impl fmt::Display for WorldError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownActor(actor) => write!(formatter, "unknown actor {}", actor.value()),
      Self::DuplicateItemId(item) => {
        write!(formatter, "item id {} is duplicated", item.value())
      }
      Self::InventoryFull(actor) => {
        write!(formatter, "actor {} inventory is full", actor.value())
      }
      Self::ItemNotOwned { actor, item } => write!(
        formatter,
        "actor {} does not own item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemEquipped { actor, item } => write!(
        formatter,
        "actor {} cannot move equipped item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotOnGround { actor, item } => write!(
        formatter,
        "actor {} has no item {} on the ground at its position",
        actor.value(),
        item.value()
      ),
      Self::TeleportOutOfBounds { actor, position } => write!(
        formatter,
        "actor {} cannot teleport out of bounds to ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::TeleportOnBlockedTile { actor, position } => write!(
        formatter,
        "actor {} cannot teleport onto blocked tile at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::TeleportOccupied {
        actor,
        blocker,
        position,
      } => write!(
        formatter,
        "actor {} cannot teleport onto actor {} at ({}, {})",
        actor.value(),
        blocker.value(),
        position.x(),
        position.y()
      ),
      Self::DuplicateActorId(actor) => {
        write!(formatter, "actor id {} is duplicated", actor.value())
      }
      Self::ActorOutOfBounds { actor, position } => write!(
        formatter,
        "actor {} starts out of bounds at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::ActorOnBlockedTile { actor, position } => write!(
        formatter,
        "actor {} starts on blocked tile at ({}, {})",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::OverlappingActors {
        first,
        second,
        position,
      } => write!(
        formatter,
        "actors {} and {} overlap at ({}, {})",
        first.value(),
        second.value(),
        position.x(),
        position.y()
      ),
      Self::ActorDeadAtStart { actor } => {
        write!(
          formatter,
          "actor {} starts with zero hit points",
          actor.value()
        )
      }
    }
  }
}

impl std::error::Error for WorldError {}

/// Errors produced when a command cannot be applied to the current world state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommandError {
  /// The command addresses no actor in the world.
  UnknownActor(ActorId),
  /// The command addresses an actor other than the deterministic next actor.
  ActorNotScheduled {
    /// The actor addressed by the command.
    requested: ActorId,
    /// The actor selected by ready time and identity.
    scheduled: ActorId,
  },
  /// The actor's next ready time would overflow the integer timeline.
  ScheduleOverflow(ActorId),
  /// The command actor is dead and cannot act.
  ActorDead(ActorId),
  /// The attack target is not present in the world.
  UnknownTarget(ActorId),
  /// The attack target is already dead.
  TargetDead(ActorId),
  /// An actor cannot target itself with an attack.
  CannotAttackSelf(ActorId),
  /// A chase command must be issued by an enemy actor.
  ChaseRequiresEnemy(ActorId),
  /// A pickup command must be issued by a player actor.
  PickupRequiresPlayer(ActorId),
  /// A drop command must be issued by a player actor.
  DropRequiresPlayer(ActorId),
  /// A reload command must be issued by a player actor.
  ReloadRequiresPlayer(ActorId),
  /// An interaction must target an adjacent closed door.
  InteractTargetInvalid {
    /// The actor issuing the interaction.
    actor: ActorId,
    /// The requested interaction position.
    position: Position,
  },
  /// A kick did not target an adjacent closed door.
  KickTargetInvalid {
    /// The actor issuing the kick.
    actor: ActorId,
    /// The requested door position.
    position: Position,
  },
  /// A close did not target an adjacent open door.
  CloseTargetInvalid {
    /// The actor issuing the close command.
    actor: ActorId,
    /// The requested open-door position.
    position: Position,
  },
  /// A living actor occupies the open doorway being closed.
  DoorCloseOccupied {
    /// The actor issuing the close command.
    actor: ActorId,
    /// The occupied doorway position.
    position: Position,
    /// The actor preventing the close.
    occupant: ActorId,
  },
  /// A break command did not target an adjacent breakable terrain cell.
  BreakTargetInvalid {
    /// The actor issuing the break command.
    actor: ActorId,
    /// The requested terrain position.
    position: Position,
  },
  /// An enemy cannot chase itself.
  CannotChaseSelf(ActorId),
  /// A retreat request must come from a kiter enemy.
  RetreatRequiresKiter(ActorId),
  /// A kiter cannot retreat from itself.
  CannotRetreatSelf(ActorId),
  /// The retreat target is not adjacent to the kiter.
  RetreatTargetNotAdjacent {
    /// The kiter issuing the retreat.
    actor: ActorId,
    /// The actor that should be adjacent.
    target: ActorId,
  },
  /// No walkable unoccupied tile increases distance from the retreat target.
  RetreatNoEscape(ActorId),
  /// A noise investigation request must come from an enemy actor.
  InvestigateRequiresEnemy(ActorId),
  /// The enemy has no pending one-use noise target.
  NoNoiseToInvestigate(ActorId),
  /// The requested position does not match the enemy's pending noise target.
  InvestigateTargetInvalid {
    /// The enemy issuing the investigation.
    actor: ActorId,
    /// The requested noise position.
    position: Position,
  },
  /// The attack target is outside the attacker's melee reach.
  AttackOutOfRange {
    /// The actor issuing the attack.
    attacker: ActorId,
    /// The actor outside melee range.
    target: ActorId,
  },
  /// The ranged target is not two or three tiles from the attacker.
  RangedAttackOutOfRange {
    /// The actor issuing the ranged attack.
    attacker: ActorId,
    /// The actor outside the bounded ranged interval.
    target: ActorId,
  },
  /// A ranged target is not visible along a clear cardinal ray.
  RangedAttackNoLineOfSight {
    /// The actor issuing the ranged attack.
    attacker: ActorId,
    /// The actor hidden by a diagonal path or blocking terrain.
    target: ActorId,
  },
  /// The actor has no ranged ammunition remaining.
  RangedAttackNoAmmunition(ActorId),
  /// The actor already has the full ranged ammunition capacity.
  ReloadNotNeeded(ActorId),
  /// A throw command must be issued by a player actor.
  ThrowRequiresPlayer(ActorId),
  /// A throw target cannot be the throwing actor.
  CannotThrowSelf(ActorId),
  /// The throw target is outside the bounded cardinal ranged interval.
  ThrowOutOfRange {
    /// The actor issuing the throw.
    attacker: ActorId,
    /// The actor outside throw range.
    target: ActorId,
  },
  /// A throw target is not visible along a clear cardinal ray.
  ThrowNoLineOfSight {
    /// The actor issuing the throw.
    attacker: ActorId,
    /// The actor hidden by geometry.
    target: ActorId,
  },
  /// The actor does not own the requested item.
  ItemNotOwned {
    /// The actor whose inventory was searched.
    actor: ActorId,
    /// The item identity that was not found.
    item: ItemId,
  },
  /// The actor's fixed inventory capacity would be exceeded.
  InventoryFull(ActorId),
  /// The requested item is already equipped.
  ItemAlreadyEquipped {
    /// The actor whose equipment was queried.
    actor: ActorId,
    /// The already equipped item identity.
    item: ItemId,
  },
  /// The actor has no equipped item to remove.
  NothingEquipped(ActorId),
  /// The requested item is equipped and cannot be moved or consumed.
  ItemEquipped {
    /// The actor whose equipment references the item.
    actor: ActorId,
    /// The equipped item identity.
    item: ItemId,
  },
  /// The requested item is equipment and cannot be consumed.
  ItemNotConsumable {
    /// The actor whose inventory was queried.
    actor: ActorId,
    /// The non-consumable item identity.
    item: ItemId,
  },
  /// The requested owned item has no throwable effect.
  ItemNotThrowable {
    /// The actor whose inventory was queried.
    actor: ActorId,
    /// The non-throwable item identity.
    item: ItemId,
  },
  /// The requested item is not in the actor's current ground stack.
  ItemNotOnGround {
    /// The actor whose current ground stack was searched.
    actor: ActorId,
    /// The missing ground item identity.
    item: ItemId,
  },
}

impl fmt::Display for CommandError {
  #[expect(
    clippy::too_many_lines,
    reason = "the command boundary keeps each typed rejection message exhaustive"
  )]
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnknownActor(actor) => write!(formatter, "unknown actor {}", actor.value()),
      Self::ActorNotScheduled {
        requested,
        scheduled,
      } => write!(
        formatter,
        "actor {} is not scheduled; actor {} must act next",
        requested.value(),
        scheduled.value()
      ),
      Self::ScheduleOverflow(actor) => {
        write!(
          formatter,
          "actor {} cannot advance its ready time",
          actor.value()
        )
      }
      Self::ActorDead(actor) => write!(formatter, "actor {} is dead", actor.value()),
      Self::UnknownTarget(target) => write!(formatter, "unknown attack target {}", target.value()),
      Self::TargetDead(target) => write!(formatter, "attack target {} is dead", target.value()),
      Self::CannotAttackSelf(actor) => {
        write!(formatter, "actor {} cannot attack itself", actor.value())
      }
      Self::ChaseRequiresEnemy(actor) => {
        write!(
          formatter,
          "actor {} cannot issue an enemy chase",
          actor.value()
        )
      }
      Self::PickupRequiresPlayer(actor) => {
        write!(
          formatter,
          "actor {} cannot issue a player pickup",
          actor.value()
        )
      }
      Self::DropRequiresPlayer(actor) => {
        write!(
          formatter,
          "actor {} cannot issue a player drop",
          actor.value()
        )
      }
      Self::ReloadRequiresPlayer(actor) => {
        write!(
          formatter,
          "actor {} cannot reload because only players may reload",
          actor.value()
        )
      }
      Self::InteractTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot interact with ({}, {}): target is not an adjacent closed door",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::KickTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot kick ({}, {}): target is not an adjacent closed door",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::CloseTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot close ({}, {}): target is not an adjacent open door",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::DoorCloseOccupied {
        actor,
        position,
        occupant,
      } => write!(
        formatter,
        "actor {} cannot close ({}, {}): actor {} occupies the doorway",
        actor.value(),
        position.x(),
        position.y(),
        occupant.value()
      ),
      Self::BreakTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot break ({}, {}): target is not an adjacent breakable tile",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::CannotChaseSelf(actor) => {
        write!(formatter, "actor {} cannot chase itself", actor.value())
      }
      Self::RetreatRequiresKiter(actor) => write!(
        formatter,
        "actor {} cannot retreat because only kiters may retreat",
        actor.value()
      ),
      Self::CannotRetreatSelf(actor) => {
        write!(
          formatter,
          "actor {} cannot retreat from itself",
          actor.value()
        )
      }
      Self::RetreatTargetNotAdjacent { actor, target } => write!(
        formatter,
        "kiter {} cannot retreat from non-adjacent target {}",
        actor.value(),
        target.value()
      ),
      Self::RetreatNoEscape(actor) => {
        write!(
          formatter,
          "kiter {} has no valid retreat tile",
          actor.value()
        )
      }
      Self::InvestigateRequiresEnemy(actor) => write!(
        formatter,
        "actor {} cannot investigate noise because only enemies may investigate",
        actor.value()
      ),
      Self::NoNoiseToInvestigate(actor) => {
        write!(
          formatter,
          "actor {} has no pending noise to investigate",
          actor.value()
        )
      }
      Self::InvestigateTargetInvalid { actor, position } => write!(
        formatter,
        "actor {} cannot investigate noise at ({}, {}): target is stale",
        actor.value(),
        position.x(),
        position.y()
      ),
      Self::AttackOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot attack non-adjacent target {}",
        attacker.value(),
        target.value()
      ),
      Self::RangedAttackOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot ranged attack target {} outside distance 2..=3",
        attacker.value(),
        target.value()
      ),
      Self::RangedAttackNoLineOfSight { attacker, target } => write!(
        formatter,
        "actor {} cannot ranged attack target {} without a clear cardinal line of sight",
        attacker.value(),
        target.value()
      ),
      Self::RangedAttackNoAmmunition(actor) => write!(
        formatter,
        "actor {} cannot ranged attack without ammunition",
        actor.value()
      ),
      Self::ReloadNotNeeded(actor) => write!(
        formatter,
        "actor {} cannot reload with full ammunition",
        actor.value()
      ),
      Self::ThrowRequiresPlayer(actor) => write!(
        formatter,
        "actor {} cannot throw because only players may throw",
        actor.value()
      ),
      Self::CannotThrowSelf(actor) => {
        write!(formatter, "actor {} cannot throw at itself", actor.value())
      }
      Self::ThrowOutOfRange { attacker, target } => write!(
        formatter,
        "actor {} cannot throw at target {} outside distance 2..=3",
        attacker.value(),
        target.value()
      ),
      Self::ThrowNoLineOfSight { attacker, target } => write!(
        formatter,
        "actor {} cannot throw at target {} without a clear cardinal line of sight",
        attacker.value(),
        target.value()
      ),
      Self::ItemNotOwned { actor, item } => write!(
        formatter,
        "actor {} does not own item {}",
        actor.value(),
        item.value()
      ),
      Self::InventoryFull(actor) => write!(formatter, "actor {} inventory is full", actor.value()),
      Self::ItemAlreadyEquipped { actor, item } => write!(
        formatter,
        "actor {} already equips item {}",
        actor.value(),
        item.value()
      ),
      Self::NothingEquipped(actor) => {
        write!(formatter, "actor {} has no equipped item", actor.value())
      }
      Self::ItemEquipped { actor, item } => write!(
        formatter,
        "actor {} cannot move or consume equipped item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotConsumable { actor, item } => write!(
        formatter,
        "actor {} cannot consume non-consumable item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotThrowable { actor, item } => write!(
        formatter,
        "actor {} cannot throw non-throwable item {}",
        actor.value(),
        item.value()
      ),
      Self::ItemNotOnGround { actor, item } => write!(
        formatter,
        "actor {} does not have item {} on the ground",
        actor.value(),
        item.value()
      ),
    }
  }
}

impl std::error::Error for CommandError {}

/// The observable result of one accepted command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionResult {
  pub(crate) events: Vec<Event>,
  pub(crate) next_actor: Option<ActorId>,
  pub(crate) current_time: ActionTime,
}

impl ActionResult {
  /// Returns semantic events emitted by this command.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the actor selected to act after this command.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the world's minimum ready time after this command.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }
}
