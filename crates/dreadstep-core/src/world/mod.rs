//! Authoritative world state and command execution.
//!
//! [`WorldState`] is the only owner of map occupancy, inventories, scheduling, and replay-facing
//! transitions. Submodules hold command handlers; they do not create a second source of truth.

use std::collections::BTreeMap;

use crate::{
  ActionCost, ActionResult, ActionTime, Actor, ActorId, ActorKind, Command, CommandError,
  EnemyBehavior, Event, GridMap, GroundItemStack, Position, RunOutcome, StateDigest, StatusKind,
  Tile, WorldError,
  replay::{StableHasher, hash_equipment_effect, hash_item_effect, hash_throwable_effect},
};

mod combat;
mod intent;
mod items;
mod legal;
mod movement;
mod terrain;
mod tester;

#[cfg(test)]
mod overflow_tests;

/// The authoritative deterministic state for the current grid slice.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorldState {
  pub(crate) map: GridMap,
  pub(crate) actors: BTreeMap<ActorId, Actor>,
  pub(crate) ground_items: Vec<GroundItemStack>,
  pub(crate) current_time: ActionTime,
}

impl WorldState {
  /// Validates and creates a world from a map and its initial actors.
  ///
  /// # Errors
  ///
  /// Returns a [`WorldError`] when an actor identity is duplicated, an actor is outside the
  /// map, an actor starts on blocking terrain, an actor starts dead, or two actors overlap.
  pub fn new(map: GridMap, actors: Vec<Actor>) -> Result<Self, WorldError> {
    let mut indexed_actors = BTreeMap::new();
    for actor in actors {
      let actor_id = actor.id();
      let position = actor.position();
      if indexed_actors.contains_key(&actor_id) {
        return Err(WorldError::DuplicateActorId(actor_id));
      }
      if !actor.is_alive() {
        return Err(WorldError::ActorDeadAtStart { actor: actor_id });
      }
      if !map.in_bounds(position) {
        return Err(WorldError::ActorOutOfBounds {
          actor: actor_id,
          position,
        });
      }
      if !map.is_walkable(position) {
        return Err(WorldError::ActorOnBlockedTile {
          actor: actor_id,
          position,
        });
      }
      if let Some(first) = indexed_actors
        .values()
        .find(|existing: &&Actor| existing.position() == position)
      {
        return Err(WorldError::OverlappingActors {
          first: first.id(),
          second: actor_id,
          position,
        });
      }
      indexed_actors.insert(actor_id, actor);
    }
    let current_time = indexed_actors
      .values()
      .map(Actor::ready_at)
      .min()
      .unwrap_or(ActionTime::new(0));
    Ok(Self {
      map,
      actors: indexed_actors,
      ground_items: Vec::new(),
      current_time,
    })
  }
  /// Returns the immutable map owned by this world.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    &self.map
  }

  /// Replaces one map tile for an explicit tester or presentation fixture mutation.
  ///
  /// This operation does not consume time or enter replay evidence. Player-facing terrain
  /// changes still go through semantic commands such as [`Command::Interact`].
  pub fn set_tile(&mut self, position: Position, tile: Tile) -> Option<Tile> {
    self.map.set_tile(position, tile)
  }

  /// Returns an actor by stable identity.
  #[must_use]
  pub fn actor(&self, actor: ActorId) -> Option<&Actor> {
    self.actors.get(&actor)
  }

  /// Returns all actor records in stable [`ActorId`] order.
  ///
  /// Dead actors remain in this read-only projection so adapters can report their final state;
  /// scheduling and occupancy continue to consider living actors only.
  #[must_use = "iterate over the actor records"]
  pub fn actors(&self) -> impl Iterator<Item = &Actor> + '_ {
    self.actors.values()
  }

  /// Returns the deterministic terminal outcome derived from retained actor records.
  ///
  /// Player defeat takes precedence so a world containing no living player can never be reported
  /// as a victory. A world without an enemy remains in progress until authored content provides a
  /// concrete opponent to defeat.
  #[must_use]
  pub fn outcome(&self) -> RunOutcome {
    if self
      .actors
      .values()
      .any(|actor| actor.kind() == ActorKind::Player && !actor.is_alive())
    {
      return RunOutcome::Defeat;
    }
    let has_enemy = self
      .actors
      .values()
      .any(|actor| actor.kind() == ActorKind::Enemy);
    if has_enemy
      && self
        .actors
        .values()
        .filter(|actor| actor.kind() == ActorKind::Enemy)
        .all(|actor| !actor.is_alive())
    {
      RunOutcome::Victory
    } else {
      RunOutcome::InProgress
    }
  }

  /// Returns ground-item stacks in deterministic row-major position order.
  #[must_use = "inspect the ground-item stacks"]
  pub fn ground_items(&self) -> &[GroundItemStack] {
    &self.ground_items
  }

  /// Returns the world's minimum ready time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns a stable digest of all semantic world state.
  ///
  /// The digest includes map dimensions and terrain, current action time, and every actor's
  /// identity, kind, enemy behavior, life, position, current and maximum hit points, ranged ammunition, ready
  /// time, optional one-use hearing target, ordered inventory item identities, definition
  /// references and effects, optional equipped item identity, and ordered ground-item stacks. It
  /// is deterministic regression evidence, not a cryptographic integrity check or serialized state
  /// format.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    let mut hasher = StableHasher::new();
    hasher.write_bytes(b"DREADSTEP-STATE-V8");
    hasher.write_u32(self.map.width());
    hasher.write_u32(self.map.height());
    for tile in self.map.tiles() {
      hasher.write_u8(match tile {
        Tile::Floor => 1,
        Tile::Cover => 2,
        Tile::Wall => 3,
        Tile::Door => 4,
        Tile::OpenDoor => 5,
        Tile::Breakable => 6,
        Tile::Trap => 7,
        Tile::ChillTrap => 8,
      });
    }
    hasher.write_u64(self.current_time.value());
    hasher.write_u64(u64::try_from(self.actors.len()).unwrap_or(u64::MAX));
    for actor in self.actors.values() {
      hasher.write_u32(actor.id().value());
      hasher.write_u8(match actor.kind() {
        ActorKind::Player => 1,
        ActorKind::Enemy => 2,
      });
      hasher.write_u8(match actor.enemy_behavior() {
        EnemyBehavior::Pursuer => 1,
        EnemyBehavior::Kiter => 2,
        EnemyBehavior::Brute => 3,
        EnemyBehavior::Frostcaster => 4,
        EnemyBehavior::Blocker => 5,
        EnemyBehavior::Scavenger => 6,
      });
      hasher.write_i32(actor.position().x());
      hasher.write_i32(actor.position().y());
      hasher.write_u16(actor.hit_points().value());
      hasher.write_u16(actor.max_hit_points().value());
      hasher.write_u8(actor.base_melee_reach().value());
      hasher.write_u16(actor.ranged_ammo());
      hasher.write_u64(actor.ready_at().value());
      match actor.heard_noise() {
        Some(position) => {
          hasher.write_u8(1);
          hasher.write_i32(position.x());
          hasher.write_i32(position.y());
        }
        None => hasher.write_u8(0),
      }
      match actor.status() {
        Some(status) => {
          hasher.write_u8(1);
          hasher.write_u8(match status.kind() {
            StatusKind::Chilled => 1,
          });
          hasher.write_u8(status.remaining_actions());
        }
        None => hasher.write_u8(0),
      }
      hasher.write_u64(u64::try_from(actor.inventory().len()).unwrap_or(u64::MAX));
      for item in actor.inventory() {
        hasher.write_u32(item.id().value());
        hasher.write_u32(item.definition().value());
        hash_item_effect(&mut hasher, item.effect());
        hash_equipment_effect(&mut hasher, item.equipment_effect());
        hash_throwable_effect(&mut hasher, item.throwable_effect());
      }
      match actor.equipped_item() {
        Some(item) => {
          hasher.write_u8(1);
          hasher.write_u32(item.value());
        }
        None => hasher.write_u8(0),
      }
    }
    if !self.ground_items.is_empty() {
      hasher.write_u64(u64::try_from(self.ground_items.len()).unwrap_or(u64::MAX));
      for stack in &self.ground_items {
        hasher.write_i32(stack.position().x());
        hasher.write_i32(stack.position().y());
        hasher.write_u64(u64::try_from(stack.items().len()).unwrap_or(u64::MAX));
        for item in stack.items() {
          hasher.write_u32(item.id().value());
          hasher.write_u32(item.definition().value());
          hash_item_effect(&mut hasher, item.effect());
          hash_equipment_effect(&mut hasher, item.equipment_effect());
          hash_throwable_effect(&mut hasher, item.throwable_effect());
        }
      }
    }
    hasher.finish()
  }

  /// Returns the actor selected by ready time, then stable identity.
  #[must_use]
  pub fn next_actor(&self) -> Option<ActorId> {
    self
      .actors
      .values()
      .filter(|actor| actor.is_alive())
      .min_by_key(|actor| (actor.ready_at(), actor.id()))
      .map(Actor::id)
  }

  /// Returns commands currently available to the scheduled living actor.
  ///
  /// Cardinal movement and waiting are always listed because blocked movement still produces an
  /// accepted semantic action. Each owned item that is not already equipped contributes an Equip
  /// action, and each owned consumable contributes a `UseItem` action; equipment effects are not
  /// consumable. The optional unequip action follows inventory order.
  /// Player attacks include targets within the actor's melee reach and clear cardinal rays two or
  /// three tiles away for the bounded ranged command. Enemies attack adjacent living targets,
  /// include clear ranged targets when ammunition and schedule capacity allow, Frostcasters
  /// replace those ranged attacks with Chilled casts, then consume one-use noise investigations
  /// before retaining chase commands for every distinct living target.
  /// Results follow the fixed direction, inventory, and then stable actor identity order.
  #[must_use]
  pub fn legal_commands(&self) -> Vec<Command> {
    let Some(actor_id) = self.next_actor() else {
      return Vec::new();
    };
    let Some(actor) = self.actors.get(&actor_id) else {
      return Vec::new();
    };
    if self
      .action_cost(actor_id, ActionCost::STANDARD)
      .and_then(|cost| actor.ready_at().checked_add(cost))
      .is_none()
    {
      return Vec::new();
    }

    let mut commands = Vec::new();
    Self::push_standard_moves(actor_id, &mut commands);
    self.push_adjacent_terrain_commands(actor_id, actor, &mut commands);
    self.push_inventory_commands(actor_id, actor, &mut commands);
    let mut living_targets = self
      .actors
      .values()
      .filter(|target| target.is_alive() && target.id() != actor_id)
      .collect::<Vec<_>>();
    living_targets.sort_by_key(|target| target.id());
    if actor.kind() == ActorKind::Enemy {
      self.push_enemy_combat_commands(actor_id, actor, &living_targets, &mut commands);
    } else {
      self.push_player_combat_commands(actor_id, actor, &living_targets, &mut commands);
    }
    commands
  }

  /// Applies one command from the deterministically scheduled actor.
  ///
  /// # Errors
  ///
  /// Returns [`CommandError::UnknownActor`] for an unknown identity,
  /// [`CommandError::ActorDead`] for a dead command actor,
  /// [`CommandError::ActorNotScheduled`] when a different actor must act first, an equipment or
  /// target error for invalid requests, or [`CommandError::ScheduleOverflow`] if the integer
  /// timeline cannot advance.
  #[expect(
    clippy::too_many_lines,
    reason = "command execution keeps shared validation and event ordering together"
  )]
  pub fn execute(&mut self, command: Command) -> Result<ActionResult, CommandError> {
    let actor_id = command.actor();
    let actor = self
      .actors
      .get(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?;
    if !actor.is_alive() {
      return Err(CommandError::ActorDead(actor_id));
    }
    let ready_at = actor.ready_at();
    if let Some(scheduled) = self.next_actor()
      && scheduled != actor_id
    {
      return Err(CommandError::ActorNotScheduled {
        requested: actor_id,
        scheduled,
      });
    }
    if matches!(command, Command::RangedAttack { .. }) && actor.ranged_ammo() == 0 {
      return Err(CommandError::RangedAttackNoAmmunition(actor_id));
    }
    if matches!(command, Command::Reload { .. }) {
      if actor.kind() != ActorKind::Player {
        return Err(CommandError::ReloadRequiresPlayer(actor_id));
      }
      if actor.ranged_ammo() >= Actor::RANGED_AMMO_CAPACITY {
        return Err(CommandError::ReloadNotNeeded(actor_id));
      }
    }
    let status_affected = actor.status().is_some();
    let base_cost = match command {
      Command::RangedAttack { .. } | Command::CastChill { .. } => ActionCost::RANGED,
      _ => ActionCost::STANDARD,
    };
    let action_cost = self
      .action_cost(actor_id, base_cost)
      .ok_or(CommandError::ScheduleOverflow(actor_id))?;
    let next_ready_at = ready_at
      .checked_add(action_cost)
      .ok_or(CommandError::ScheduleOverflow(actor_id))?;
    let mut events = match command {
      Command::Move { direction, .. } => self.move_actor(actor_id, direction)?,
      Command::Wait { .. } => vec![Event::Waited {
        actor: actor_id,
        at: self.current_time,
      }],
      Command::Interact { position, .. } => vec![self.interact(actor_id, position)?],
      Command::Kick { position, .. } => self.kick_door(actor_id, position)?,
      Command::Close { position, .. } => vec![self.close_door(actor_id, position)?],
      Command::Break { position, .. } => vec![self.break_terrain(actor_id, position)?],
      Command::Attack { target, .. } => self.attack(actor_id, target)?,
      Command::RangedAttack { target, .. } => self.ranged_attack(actor_id, target)?,
      Command::CastChill { target, .. } => self.cast_chill(actor_id, target)?,
      Command::Throw { item, target, .. } => self.throw_item(actor_id, item, target)?,
      Command::Retreat { target, .. } => self.retreat(actor_id, target)?,
      Command::Chase { target, .. } => {
        let direction = self.chase_direction(actor_id, target)?;
        self.move_actor(actor_id, direction)?
      }
      Command::Investigate { position, .. } => self.investigate(actor_id, position)?,
      Command::Equip { item, .. } => self.equip_item(actor_id, item)?,
      Command::Unequip { .. } => vec![self.unequip_item(actor_id)?],
      Command::UseItem { item, .. } => vec![self.use_item(actor_id, item)?],
      Command::Pickup { item, .. } => vec![self.pickup_item_command(actor_id, item)?],
      Command::Drop { item, .. } => vec![self.drop_item_command(actor_id, item)?],
      Command::Reload { .. } => {
        self
          .actors
          .get_mut(&actor_id)
          .ok_or(CommandError::UnknownActor(actor_id))?
          .ranged_ammo = Actor::RANGED_AMMO_CAPACITY;
        vec![Event::Reloaded {
          actor: actor_id,
          ammunition: Actor::RANGED_AMMO_CAPACITY,
        }]
      }
    };
    let status_refreshed = events
      .iter()
      .any(|event| matches!(event, Event::StatusApplied { actor, .. } if *actor == actor_id));
    if status_affected
      && !status_refreshed
      && let Some(status) = self
        .actors
        .get_mut(&actor_id)
        .ok_or(CommandError::UnknownActor(actor_id))?
        .consume_status_action()
    {
      events.push(Event::StatusExpired {
        actor: actor_id,
        status,
      });
    }
    if matches!(command, Command::RangedAttack { .. }) {
      self
        .actors
        .get_mut(&actor_id)
        .ok_or(CommandError::UnknownActor(actor_id))?
        .ranged_ammo -= 1;
    }
    self
      .actors
      .get_mut(&actor_id)
      .ok_or(CommandError::UnknownActor(actor_id))?
      .ready_at = next_ready_at;
    self.current_time = self
      .next_actor()
      .and_then(|next| self.actors.get(&next).map(Actor::ready_at))
      .unwrap_or(next_ready_at);
    Ok(ActionResult {
      events,
      next_actor: self.next_actor(),
      current_time: self.current_time,
    })
  }

  pub(super) fn action_cost(&self, actor_id: ActorId, base: ActionCost) -> Option<ActionCost> {
    let extra = u64::from(self.actors.get(&actor_id)?.status().is_some());
    base.value().checked_add(extra).map(ActionCost::new)
  }

  fn actor_at(&self, position: Position) -> Option<ActorId> {
    self
      .actors
      .values()
      .find(|actor| actor.is_alive() && actor.position() == position)
      .map(Actor::id)
  }
}
