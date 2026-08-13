//! Deterministic legal-command projection helpers.
//!
//! These builders exist only to keep discovery order explicit and readable. Player and enemy
//! combat policies stay separate because their advertised command families differ.

use crate::{
  ActionCost, Actor, ActorId, ActorKind, Command, Direction, EnemyBehavior, Tile, WorldState,
};

impl WorldState {
  pub(super) fn push_standard_moves(actor_id: ActorId, commands: &mut Vec<Command>) {
    commands.extend([
      Command::Move {
        actor: actor_id,
        direction: Direction::North,
      },
      Command::Move {
        actor: actor_id,
        direction: Direction::South,
      },
      Command::Move {
        actor: actor_id,
        direction: Direction::West,
      },
      Command::Move {
        actor: actor_id,
        direction: Direction::East,
      },
      Command::Wait { actor: actor_id },
    ]);
  }

  pub(super) fn push_adjacent_terrain_commands(
    &self,
    actor_id: ActorId,
    actor: &Actor,
    commands: &mut Vec<Command>,
  ) {
    for direction in [
      Direction::North,
      Direction::South,
      Direction::West,
      Direction::East,
    ] {
      let position = actor.position().translated(direction);
      if self.map.tile_at(position) == Some(Tile::Door) {
        commands.push(Command::Interact {
          actor: actor_id,
          position,
        });
        commands.push(Command::Kick {
          actor: actor_id,
          position,
        });
      }
      if self.map.tile_at(position) == Some(Tile::Breakable) {
        commands.push(Command::Break {
          actor: actor_id,
          position,
        });
      }
    }
  }

  pub(super) fn push_inventory_commands(
    &self,
    actor_id: ActorId,
    actor: &Actor,
    commands: &mut Vec<Command>,
  ) {
    if actor.kind() == ActorKind::Player && actor.ranged_ammo() < Actor::RANGED_AMMO_CAPACITY {
      commands.push(Command::Reload { actor: actor_id });
    }
    if actor.kind() == ActorKind::Player
      && actor.inventory().len() < Actor::INVENTORY_CAPACITY
      && let Some(stack) = self
        .ground_items
        .iter()
        .find(|stack| stack.position() == actor.position())
    {
      for item in stack.items() {
        commands.push(Command::Pickup {
          actor: actor_id,
          item: item.id(),
        });
      }
    }
    if actor.kind() == ActorKind::Player {
      for item in actor.inventory() {
        if actor.equipped_item() != Some(item.id()) {
          commands.push(Command::Drop {
            actor: actor_id,
            item: item.id(),
          });
        }
      }
    }
    for item in actor.inventory() {
      if actor.equipped_item() != Some(item.id()) {
        commands.push(Command::Equip {
          actor: actor_id,
          item: item.id(),
        });
        if item.equipment_effect().is_none() {
          commands.push(Command::UseItem {
            actor: actor_id,
            item: item.id(),
          });
        }
      }
    }
    if actor.equipped_item().is_some() {
      commands.push(Command::Unequip { actor: actor_id });
    }
  }

  pub(super) fn push_enemy_combat_commands(
    &self,
    actor_id: ActorId,
    actor: &Actor,
    living_targets: &[&Actor],
    commands: &mut Vec<Command>,
  ) {
    if actor.enemy_behavior() == EnemyBehavior::Kiter {
      for target in living_targets {
        if Self::manhattan_distance(actor.position(), target.position()) == 1
          && self.retreat_direction(actor_id, target.id()).is_ok()
        {
          commands.push(Command::Retreat {
            actor: actor_id,
            target: target.id(),
          });
        }
      }
    }
    for target in living_targets {
      if Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach()) {
        commands.push(Command::Attack {
          actor: actor_id,
          target: target.id(),
        });
      }
    }
    for target in living_targets {
      if !Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach())
        && Self::is_ranged_distance(actor.position(), target.position())
        && self.has_ranged_line_of_sight(actor.position(), target.position())
        && actor.ranged_ammo() > 0
        && self
          .action_cost(actor_id, ActionCost::RANGED)
          .and_then(|cost| actor.ready_at().checked_add(cost))
          .is_some()
      {
        commands.push(Command::RangedAttack {
          actor: actor_id,
          target: target.id(),
        });
      }
    }
    if let Some(position) = actor.heard_noise()
      && actor.position() != position
    {
      commands.push(Command::Investigate {
        actor: actor_id,
        position,
      });
    }
    for target in living_targets {
      if !Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach()) {
        commands.push(Command::Chase {
          actor: actor_id,
          target: target.id(),
        });
      }
    }
  }

  pub(super) fn push_player_combat_commands(
    &self,
    actor_id: ActorId,
    actor: &Actor,
    living_targets: &[&Actor],
    commands: &mut Vec<Command>,
  ) {
    for target in living_targets {
      if Self::is_melee_distance(actor.position(), target.position(), actor.melee_reach()) {
        commands.push(Command::Attack {
          actor: actor_id,
          target: target.id(),
        });
      } else if Self::is_ranged_distance(actor.position(), target.position())
        && self.has_ranged_line_of_sight(actor.position(), target.position())
        && actor.ranged_ammo() > 0
        && self
          .action_cost(actor_id, ActionCost::RANGED)
          .and_then(|cost| actor.ready_at().checked_add(cost))
          .is_some()
      {
        commands.push(Command::RangedAttack {
          actor: actor_id,
          target: target.id(),
        });
      }
    }
    if actor.kind() == ActorKind::Player {
      let mut throwable_items = actor
        .inventory()
        .iter()
        .filter(|item| {
          actor.equipped_item() != Some(item.id()) && item.throwable_effect().is_some()
        })
        .copied()
        .collect::<Vec<_>>();
      throwable_items.sort_by_key(|item| item.id());
      for item in throwable_items {
        for target in living_targets {
          if Self::is_ranged_distance(actor.position(), target.position())
            && self.has_ranged_line_of_sight(actor.position(), target.position())
          {
            commands.push(Command::Throw {
              actor: actor_id,
              item: item.id(),
              target: target.id(),
            });
          }
        }
      }
    }
  }
}
