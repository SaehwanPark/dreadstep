//! Deterministic replay traces and state digests.
//!
//! Digests are regression evidence, not a cryptographic integrity check. The hasher is process-
//! independent so identical worlds hash identically across platforms.

use crate::{Command, Direction, EquipmentEffect, ItemEffect};

/// A stable, non-cryptographic digest used for deterministic regression evidence.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StateDigest(u64);

impl StateDigest {
  /// Returns the numeric digest value.
  #[must_use]
  pub const fn value(self) -> u64 {
    self.0
  }
}

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;

pub(crate) struct StableHasher {
  state: u64,
}

impl StableHasher {
  pub(crate) const fn new() -> Self {
    Self {
      state: FNV_OFFSET_BASIS,
    }
  }

  pub(crate) fn write_bytes(&mut self, bytes: &[u8]) {
    for byte in bytes {
      self.state ^= u64::from(*byte);
      self.state = self.state.wrapping_mul(FNV_PRIME);
    }
  }

  pub(crate) fn write_u8(&mut self, value: u8) {
    self.write_bytes(&[value]);
  }

  pub(crate) fn write_u16(&mut self, value: u16) {
    self.write_bytes(&value.to_le_bytes());
  }

  pub(crate) fn write_u32(&mut self, value: u32) {
    self.write_bytes(&value.to_le_bytes());
  }

  pub(crate) fn write_i32(&mut self, value: i32) {
    self.write_bytes(&value.to_le_bytes());
  }

  pub(crate) fn write_u64(&mut self, value: u64) {
    self.write_bytes(&value.to_le_bytes());
  }

  pub(crate) const fn finish(self) -> StateDigest {
    StateDigest(self.state)
  }
}

/// An ordered, seeded command trace for deterministic replay evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayTrace {
  seed: u64,
  commands: Vec<Command>,
}

impl ReplayTrace {
  /// Creates an empty trace with an explicit run seed.
  #[must_use]
  pub const fn new(seed: u64) -> Self {
    Self {
      seed,
      commands: Vec::new(),
    }
  }

  /// Returns the seed recorded with this trace.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Appends one semantic command in execution order.
  pub fn record(&mut self, command: Command) {
    self.commands.push(command);
  }

  /// Returns the commands in their recorded order.
  #[must_use]
  pub fn commands(&self) -> &[Command] {
    &self.commands
  }

  /// Returns a deterministic trace identity based on seed and command order.
  ///
  /// This is regression evidence, not a cryptographic integrity check or serialized replay
  /// format. The explicit FNV-1a byte order remains stable across process invocations.
  #[must_use]
  pub fn digest(&self) -> StateDigest {
    let mut hasher = StableHasher::new();
    hasher.write_bytes(b"DREADSTEP-REPLAY-V2");
    hasher.write_u64(self.seed);
    hasher.write_u64(u64::try_from(self.commands.len()).unwrap_or(u64::MAX));
    for command in &self.commands {
      hash_command(&mut hasher, *command);
    }
    hasher.finish()
  }
}

fn hash_command(hasher: &mut StableHasher, command: Command) {
  match command {
    Command::Move { actor, direction } => {
      hasher.write_u8(1);
      hasher.write_u32(actor.value());
      hasher.write_u8(direction_code(direction));
    }
    Command::Wait { actor } => {
      hasher.write_u8(2);
      hasher.write_u32(actor.value());
    }
    Command::Interact { actor, position } => {
      hasher.write_u8(12);
      hasher.write_u32(actor.value());
      hasher.write_i32(position.x());
      hasher.write_i32(position.y());
    }
    Command::Kick { actor, position } => {
      hasher.write_u8(14);
      hasher.write_u32(actor.value());
      hasher.write_i32(position.x());
      hasher.write_i32(position.y());
    }
    Command::Close { actor, position } => {
      hasher.write_u8(18);
      hasher.write_u32(actor.value());
      hasher.write_i32(position.x());
      hasher.write_i32(position.y());
    }
    Command::Break { actor, position } => {
      hasher.write_u8(13);
      hasher.write_u32(actor.value());
      hasher.write_i32(position.x());
      hasher.write_i32(position.y());
    }
    Command::Attack { actor, target } => {
      hasher.write_u8(3);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::RangedAttack { actor, target } => {
      hasher.write_u8(9);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::Throw {
      actor,
      item,
      target,
    } => {
      hasher.write_u8(16);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
      hasher.write_u32(target.value());
    }
    Command::Retreat { actor, target } => {
      hasher.write_u8(17);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::Chase { actor, target } => {
      hasher.write_u8(4);
      hasher.write_u32(actor.value());
      hasher.write_u32(target.value());
    }
    Command::Investigate { actor, position } => {
      hasher.write_u8(15);
      hasher.write_u32(actor.value());
      hasher.write_i32(position.x());
      hasher.write_i32(position.y());
    }
    Command::Equip { actor, item } => {
      hasher.write_u8(5);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
    Command::Unequip { actor } => {
      hasher.write_u8(6);
      hasher.write_u32(actor.value());
    }
    Command::UseItem { actor, item } => {
      hasher.write_u8(7);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
    Command::Pickup { actor, item } => {
      hasher.write_u8(8);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
    Command::Reload { actor } => {
      hasher.write_u8(10);
      hasher.write_u32(actor.value());
    }
    Command::Drop { actor, item } => {
      hasher.write_u8(11);
      hasher.write_u32(actor.value());
      hasher.write_u32(item.value());
    }
  }
}

pub(crate) fn hash_item_effect(hasher: &mut StableHasher, effect: ItemEffect) {
  match effect {
    ItemEffect::None => hasher.write_u8(0),
    ItemEffect::Heal { amount } => {
      hasher.write_u8(1);
      hasher.write_u16(amount.value());
    }
    ItemEffect::RestoreAmmunition { amount } => {
      hasher.write_u8(2);
      hasher.write_u16(amount.value());
    }
  }
}

pub(crate) fn hash_throwable_effect(
  hasher: &mut StableHasher,
  effect: Option<crate::ThrowableEffect>,
) {
  match effect {
    None => hasher.write_u8(0),
    Some(crate::ThrowableEffect::Chill) => hasher.write_u8(1),
  }
}

pub(crate) fn hash_equipment_effect(hasher: &mut StableHasher, effect: Option<EquipmentEffect>) {
  match effect {
    None => hasher.write_u8(0),
    Some(EquipmentEffect::MinimumMeleeReach { reach }) => {
      hasher.write_u8(1);
      hasher.write_u8(reach.value());
    }
  }
}

const fn direction_code(direction: Direction) -> u8 {
  match direction {
    Direction::North => 1,
    Direction::South => 2,
    Direction::West => 3,
    Direction::East => 4,
  }
}
