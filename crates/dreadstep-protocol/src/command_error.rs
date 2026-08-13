//! Typed command rejections in the versioned protocol boundary.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{ActorId, ItemId, Position};

/// Protocol command-rejection errors converted from core.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandError {
  /// The command addresses no actor in the world.
  UnknownActor(ActorId),
  /// A different actor is scheduled to act first.
  ActorNotScheduled {
    /// The actor addressed by the request.
    requested: ActorId,
    /// The actor selected by the scheduler.
    scheduled: ActorId,
  },
  /// The actor's ready time cannot advance.
  ScheduleOverflow(ActorId),
  /// The command actor is dead.
  ActorDead(ActorId),
  /// The attack target is not present.
  UnknownTarget(ActorId),
  /// The attack target is dead.
  TargetDead(ActorId),
  /// An actor cannot attack itself.
  CannotAttackSelf(ActorId),
  /// A chase request must come from an enemy.
  ChaseRequiresEnemy(ActorId),
  /// A pickup request must come from a player actor.
  PickupRequiresPlayer(ActorId),
  /// A drop request must come from a player actor.
  DropRequiresPlayer(ActorId),
  /// A reload request must come from a player actor.
  ReloadRequiresPlayer(ActorId),
  /// The requested target is not an adjacent closed door.
  InteractTargetInvalid {
    /// The actor issuing the interaction.
    actor: ActorId,
    /// The requested interaction position.
    position: Position,
  },
  /// The requested target is not an adjacent closed door for a kick.
  KickTargetInvalid {
    /// The actor issuing the kick.
    actor: ActorId,
    /// The requested door position.
    position: Position,
  },
  /// The requested target is not an adjacent breakable terrain cell.
  BreakTargetInvalid {
    /// The actor issuing the break command.
    actor: ActorId,
    /// The requested interaction position.
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
    /// The kiter issuing the request.
    actor: ActorId,
    /// The requested target.
    target: ActorId,
  },
  /// No unoccupied walkable tile increases distance from the target.
  RetreatNoEscape(ActorId),
  /// A noise investigation must come from an enemy actor.
  InvestigateRequiresEnemy(ActorId),
  /// The enemy has no pending noise target.
  NoNoiseToInvestigate(ActorId),
  /// The requested noise target is stale or does not match the pending target.
  InvestigateTargetInvalid {
    /// The enemy issuing the request.
    actor: ActorId,
    /// The requested noise position.
    position: Position,
  },
  /// The attack target is not adjacent.
  AttackOutOfRange {
    /// The actor issuing the attack.
    attacker: ActorId,
    /// The actor outside melee range.
    target: ActorId,
  },
  /// The ranged target is not two or three tiles away.
  RangedAttackOutOfRange {
    /// The actor issuing the ranged attack.
    attacker: ActorId,
    /// The actor outside the bounded ranged interval.
    target: ActorId,
  },
  /// The ranged target is not visible along a clear cardinal ray.
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
  /// A throw request must come from a player actor.
  ThrowRequiresPlayer(ActorId),
  /// An actor cannot throw an item at itself.
  CannotThrowSelf(ActorId),
  /// The throw target is not two or three tiles away.
  ThrowOutOfRange {
    /// The actor issuing the throw.
    attacker: ActorId,
    /// The actor outside the bounded throw interval.
    target: ActorId,
  },
  /// The throw target is not visible along a clear cardinal ray.
  ThrowNoLineOfSight {
    /// The actor issuing the throw.
    attacker: ActorId,
    /// The actor hidden by a diagonal path or blocking terrain.
    target: ActorId,
  },
  /// The actor does not own the requested item.
  ItemNotOwned {
    /// The actor whose inventory was searched.
    actor: ActorId,
    /// The requested item identity.
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
  /// The actor has no equipment to remove.
  NothingEquipped(ActorId),
  /// The requested item is equipped and therefore cannot be moved or consumed.
  ItemEquipped {
    /// The actor whose inventory was queried.
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
    /// The requested item identity.
    item: ItemId,
  },
}
