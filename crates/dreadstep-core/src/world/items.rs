//! Inventory, ground-stack, equipment, and item-use transitions.
//!
//! Tester item mutations and scheduled player item commands share the same core-owned stores so
//! capacity and identity invariants cannot diverge by adapter.

use crate::{
  Actor, ActorId, ActorKind, AmmunitionResult, CommandError, Event, GroundItemStack, HealingResult,
  HitPoints, Item, ItemEffect, ItemId, WorldError, WorldState,
};

impl WorldState {
  /// Gives one opaque item instance to an existing actor for an explicit tester operation.
  ///
  /// Item ownership is recorded in insertion order. The instance identity is global across all
  /// actor inventories; item effects remain outside this slice and the fixed actor capacity is
  /// enforced before insertion.
  /// Explicit tester transfers are handled separately by [`Self::transfer_item`]. Dead actor
  /// records remain valid ownership targets because the mutation does not alter scheduling or
  /// occupancy.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the target identity is absent,
  /// [`WorldError::DuplicateItemId`] when any actor inventory or ground stack already owns the
  /// item identity, or [`WorldError::InventoryFull`] when the target has no free slot. A rejected
  /// item is not inserted.
  pub fn give_item(&mut self, actor_id: ActorId, item: Item) -> Result<(), WorldError> {
    if !self.actors.contains_key(&actor_id) {
      return Err(WorldError::UnknownActor(actor_id));
    }
    if self.actors.values().any(|actor| {
      actor
        .inventory()
        .iter()
        .any(|owned| owned.id() == item.id())
    }) || self
      .ground_items
      .iter()
      .any(|stack| stack.items().iter().any(|owned| owned.id() == item.id()))
    {
      return Err(WorldError::DuplicateItemId(item.id()));
    }
    if self
      .actors
      .get(&actor_id)
      .is_some_and(|actor| actor.inventory().len() >= Actor::INVENTORY_CAPACITY)
    {
      return Err(WorldError::InventoryFull(actor_id));
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?
      .inventory
      .push(item);
    Ok(())
  }

  /// Drops one opaque item at an actor's current position for an explicit tester operation.
  ///
  /// The item is removed from the actor's ordered inventory and appended unchanged to the
  /// position's ground stack. Dead actor records remain valid sources because their retained
  /// positions remain part of the inspectable world state.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the actor identity is absent,
  /// [`WorldError::ItemNotOwned`] when the actor does not own the requested item, or
  /// [`WorldError::ItemEquipped`] when moving the requested item would invalidate the equipment
  /// reference. Rejected drops leave the world unchanged.
  pub fn drop_item(&mut self, actor_id: ActorId, item_id: ItemId) -> Result<(), WorldError> {
    if !self.actors.contains_key(&actor_id) {
      return Err(WorldError::UnknownActor(actor_id));
    }
    let Some(item_index) = self
      .actors
      .get(&actor_id)
      .and_then(|actor| actor.inventory.iter().position(|item| item.id() == item_id))
    else {
      return Err(WorldError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      });
    };
    if self.actors.get(&actor_id).and_then(Actor::equipped_item) == Some(item_id) {
      return Err(WorldError::ItemEquipped {
        actor: actor_id,
        item: item_id,
      });
    }
    let position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let mut actor = self
      .actors
      .remove(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let item = actor.inventory.remove(item_index);
    self.actors.insert(actor_id, actor);

    let position_key = (position.y(), position.x());
    match self
      .ground_items
      .binary_search_by_key(&position_key, |stack| {
        (stack.position().y(), stack.position().x())
      }) {
      Ok(index) => self.ground_items[index].items.push(item),
      Err(index) => self
        .ground_items
        .insert(index, GroundItemStack::new(position, item)),
    }
    Ok(())
  }

  /// Picks one opaque item from an actor's current ground stack for an explicit tester operation.
  ///
  /// The item is removed while preserving the remaining stack order, and appended unchanged to
  /// the actor's ordered inventory. Empty ground stacks are removed. Dead actor records remain
  /// valid sources because their retained positions remain part of the inspectable world state.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when the actor identity is absent,
  /// [`WorldError::ItemNotOnGround`] when the actor's current stack does not contain the item, or
  /// [`WorldError::InventoryFull`] when the actor has no free slot. Rejected pickups leave the
  /// world unchanged.
  pub fn pickup_item(&mut self, actor_id: ActorId, item_id: ItemId) -> Result<(), WorldError> {
    if !self.actors.contains_key(&actor_id) {
      return Err(WorldError::UnknownActor(actor_id));
    }
    let position = self
      .actors
      .get(&actor_id)
      .map(Actor::position)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let position_key = (position.y(), position.x());
    let Ok(stack_index) = self
      .ground_items
      .binary_search_by_key(&position_key, |stack| {
        (stack.position().y(), stack.position().x())
      })
    else {
      return Err(WorldError::ItemNotOnGround {
        actor: actor_id,
        item: item_id,
      });
    };
    let Some(item_index) = self.ground_items[stack_index]
      .items()
      .iter()
      .position(|item| item.id() == item_id)
    else {
      return Err(WorldError::ItemNotOnGround {
        actor: actor_id,
        item: item_id,
      });
    };

    if self
      .actors
      .get(&actor_id)
      .is_some_and(|actor| actor.inventory().len() >= Actor::INVENTORY_CAPACITY)
    {
      return Err(WorldError::InventoryFull(actor_id));
    }

    let mut actor = self
      .actors
      .remove(&actor_id)
      .ok_or(WorldError::UnknownActor(actor_id))?;
    let item = self.ground_items[stack_index].items.remove(item_index);
    if self.ground_items[stack_index].items.is_empty() {
      self.ground_items.remove(stack_index);
    }
    actor.inventory.push(item);
    self.actors.insert(actor_id, actor);
    Ok(())
  }

  /// Transfers one opaque item between existing actor records for an explicit tester operation.
  ///
  /// Cross-actor transfer removes the item from the source while preserving the relative order of
  /// remaining items, then appends the unchanged item to the target. Same-actor transfer is an
  /// idempotent no-op after ownership validation. Dead actor records remain valid endpoints because
  /// this mutation does not affect scheduling or occupancy.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when either actor identity is absent,
  /// [`WorldError::ItemNotOwned`] when the source does not own the requested item, or
  /// [`WorldError::ItemEquipped`] when moving the requested item would invalidate the equipment
  /// reference, or [`WorldError::InventoryFull`] when the target has no free slot. Rejected
  /// transfers leave the world unchanged.
  pub fn transfer_item(
    &mut self,
    source_actor: ActorId,
    target_actor: ActorId,
    item_id: ItemId,
  ) -> Result<(), WorldError> {
    if !self.actors.contains_key(&source_actor) {
      return Err(WorldError::UnknownActor(source_actor));
    }
    if !self.actors.contains_key(&target_actor) {
      return Err(WorldError::UnknownActor(target_actor));
    }
    let Some(item_index) = self
      .actors
      .get(&source_actor)
      .and_then(|actor| actor.inventory.iter().position(|item| item.id() == item_id))
    else {
      return Err(WorldError::ItemNotOwned {
        actor: source_actor,
        item: item_id,
      });
    };
    if self
      .actors
      .get(&source_actor)
      .and_then(Actor::equipped_item)
      == Some(item_id)
    {
      return Err(WorldError::ItemEquipped {
        actor: source_actor,
        item: item_id,
      });
    }
    if source_actor == target_actor {
      return Ok(());
    }
    if self
      .actors
      .get(&target_actor)
      .is_some_and(|actor| actor.inventory().len() >= Actor::INVENTORY_CAPACITY)
    {
      return Err(WorldError::InventoryFull(target_actor));
    }
    let Some(mut source) = self.actors.remove(&source_actor) else {
      return Err(WorldError::UnknownActor(source_actor));
    };
    let Some(mut target) = self.actors.remove(&target_actor) else {
      self.actors.insert(source_actor, source);
      return Err(WorldError::UnknownActor(target_actor));
    };
    let item = source.inventory.remove(item_index);
    target.inventory.push(item);
    self.actors.insert(source_actor, source);
    self.actors.insert(target_actor, target);
    Ok(())
  }

  /// Teleports one existing actor for an explicit tester operation.
  ///
  /// Teleport preserves the actor's identity, life, hit points, inventory, and ready time, and
  /// does not alter the world's current action time. Living actors occupy destinations; dead
  /// records do not, so a dead actor may be positioned on a living actor's tile until it is
  /// revived. The destination must remain a walkable map position.
  ///
  /// # Errors
  ///
  /// Returns [`WorldError::UnknownActor`] when no actor has the requested identity,
  /// [`WorldError::TeleportOutOfBounds`] or [`WorldError::TeleportOnBlockedTile`] when the
  /// destination is invalid, or [`WorldError::TeleportOccupied`] when a living actor would
  pub(super) fn equip_item(
    &mut self,
    actor_id: ActorId,
    item_id: ItemId,
  ) -> Result<Vec<Event>, CommandError> {
    let actor = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    if !actor.inventory().iter().any(|item| item.id() == item_id) {
      return Err(CommandError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      });
    }
    if actor.equipped_item() == Some(item_id) {
      return Err(CommandError::ItemAlreadyEquipped {
        actor: actor_id,
        item: item_id,
      });
    }
    let previous = actor.equipped_item();
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .equipped = Some(item_id);
    let mut events = Vec::with_capacity(2);
    if let Some(previous) = previous {
      events.push(Event::ItemUnequipped {
        actor: actor_id,
        item: previous,
      });
    }
    events.push(Event::ItemEquipped {
      actor: actor_id,
      item: item_id,
    });
    Ok(events)
  }

  pub(super) fn unequip_item(&mut self, actor_id: ActorId) -> Result<Event, CommandError> {
    let item = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .equipped_item()
      .ok_or(CommandError::NothingEquipped(actor_id))?;
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .equipped = None;
    Ok(Event::ItemUnequipped {
      actor: actor_id,
      item,
    })
  }

  pub(super) fn use_item(
    &mut self,
    actor_id: ActorId,
    item_id: ItemId,
  ) -> Result<Event, CommandError> {
    let actor = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    let Some(item) = actor.inventory().iter().find(|item| item.id() == item_id) else {
      return Err(CommandError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      });
    };
    if item.equipment_effect().is_some() {
      return Err(CommandError::ItemNotConsumable {
        actor: actor_id,
        item: item_id,
      });
    }
    if actor.equipped_item() == Some(item_id) {
      return Err(CommandError::ItemEquipped {
        actor: actor_id,
        item: item_id,
      });
    }
    let item_index = actor
      .inventory()
      .iter()
      .position(|candidate| candidate.id() == item_id)
      .ok_or(CommandError::ItemNotOwned {
        actor: actor_id,
        item: item_id,
      })?;
    let effect = item.effect();
    let current_hit_points = actor.hit_points();
    let maximum_hit_points = actor.max_hit_points();
    let current_ammunition = actor.ranged_ammo();
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .inventory
      .remove(item_index);
    let (healing, ammunition) = match effect {
      ItemEffect::None => (None, None),
      ItemEffect::Heal { amount } => {
        let restored = if current_hit_points >= maximum_hit_points {
          current_hit_points.value()
        } else {
          current_hit_points
            .value()
            .saturating_add(amount.value())
            .min(maximum_hit_points.value())
        };
        let actual = restored.saturating_sub(current_hit_points.value());
        self
          .actors
          .get_mut(&actor_id)
          .ok_or(CommandError::UnknownActor(actor_id))?
          .hit_points = HitPoints::new(restored);
        (
          Some(HealingResult::new(actual, HitPoints::new(restored))),
          None,
        )
      }
      ItemEffect::RestoreAmmunition { amount } => {
        let restored = current_ammunition
          .saturating_add(amount.value())
          .min(Actor::RANGED_AMMO_CAPACITY);
        let actual = restored.saturating_sub(current_ammunition);
        self
          .actors
          .get_mut(&actor_id)
          .ok_or(CommandError::UnknownActor(actor_id))?
          .ranged_ammo = restored;
        (None, Some(AmmunitionResult::new(actual, restored)))
      }
    };
    Ok(Event::ItemConsumed {
      actor: actor_id,
      item: item_id,
      healing,
      ammunition,
    })
  }

  pub(super) fn pickup_item_command(
    &mut self,
    actor_id: ActorId,
    item_id: ItemId,
  ) -> Result<Event, CommandError> {
    if self
      .actors
      .get(&actor_id)
      .is_some_and(|actor| actor.kind() != ActorKind::Player)
    {
      return Err(CommandError::PickupRequiresPlayer(actor_id));
    }
    self
      .pickup_item(actor_id, item_id)
      .map_err(|error| match error {
        WorldError::UnknownActor(actor) => CommandError::UnknownActor(actor),
        WorldError::InventoryFull(actor) => CommandError::InventoryFull(actor),
        WorldError::ItemNotOnGround { actor, item } => {
          CommandError::ItemNotOnGround { actor, item }
        }
        _ => CommandError::ItemNotOnGround {
          actor: actor_id,
          item: item_id,
        },
      })?;
    Ok(Event::ItemPickedUp {
      actor: actor_id,
      item: item_id,
    })
  }

  pub(super) fn drop_item_command(
    &mut self,
    actor_id: ActorId,
    item_id: ItemId,
  ) -> Result<Event, CommandError> {
    if self
      .actors
      .get(&actor_id)
      .is_some_and(|actor| actor.kind() != ActorKind::Player)
    {
      return Err(CommandError::DropRequiresPlayer(actor_id));
    }
    self
      .drop_item(actor_id, item_id)
      .map_err(|error| match error {
        WorldError::UnknownActor(actor) => CommandError::UnknownActor(actor),
        WorldError::ItemNotOwned { actor, item } => CommandError::ItemNotOwned { actor, item },
        WorldError::ItemEquipped { actor, item } => CommandError::ItemEquipped { actor, item },
        _ => CommandError::ItemNotOwned {
          actor: actor_id,
          item: item_id,
        },
      })?;
    Ok(Event::ItemDropped {
      actor: actor_id,
      item: item_id,
    })
  }
}
