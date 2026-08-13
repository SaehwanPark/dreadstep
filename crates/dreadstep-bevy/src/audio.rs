//! Audio cue families and local-only audio asset manifests.

use bevy::ecs::resource::Resource;
use dreadstep_core::{ActorId, BlockReason, Event, ItemId};

use crate::PresentationAssetReference;

/// A typed placeholder cue derived from one core semantic event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationAudioCue {
  /// An actor entered a new map position.
  Moved {
    /// The actor that moved.
    actor: ActorId,
  },
  /// An actor attempted movement but remained in place.
  MovementBlocked {
    /// The actor that attempted movement.
    actor: ActorId,
    /// Why the destination could not be entered.
    reason: BlockReason,
  },
  /// An actor spent a standard action without moving.
  Waited {
    /// The actor that waited.
    actor: ActorId,
  },
  /// An attack reduced a target's hit points.
  Attacked {
    /// The actor that attacked.
    attacker: ActorId,
    /// The actor that was hit.
    target: ActorId,
  },
  /// An actor reached zero hit points.
  Died {
    /// The actor that died.
    actor: ActorId,
  },
  /// An actor equipped an owned item.
  ItemEquipped {
    /// The actor whose equipment changed.
    actor: ActorId,
    /// The item now equipped.
    item: ItemId,
  },
  /// An actor removed its equipped item reference.
  ItemUnequipped {
    /// The actor whose equipment changed.
    actor: ActorId,
    /// The item that was unequipped.
    item: ItemId,
  },
  /// An actor consumed an owned, unequipped item instance.
  ItemConsumed {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The item instance removed from inventory.
    item: ItemId,
  },
  /// An actor picked one item from its current ground stack.
  ItemPickedUp {
    /// The actor whose inventory changed.
    actor: ActorId,
    /// The item instance moved into inventory.
    item: ItemId,
  },
}

impl PresentationAudioCue {
  pub(crate) fn from_event(event: Event) -> Option<Self> {
    match event {
      Event::Moved { actor, .. } => Some(Self::Moved { actor }),
      Event::MovementBlocked { actor, reason, .. } => Some(Self::MovementBlocked { actor, reason }),
      Event::Waited { actor, .. } => Some(Self::Waited { actor }),
      Event::Attacked {
        attacker, target, ..
      } => Some(Self::Attacked { attacker, target }),
      Event::Died { actor } => Some(Self::Died { actor }),
      Event::ItemEquipped { actor, item } => Some(Self::ItemEquipped { actor, item }),
      Event::ItemUnequipped { actor, item } => Some(Self::ItemUnequipped { actor, item }),
      Event::ItemConsumed { actor, item, .. } => Some(Self::ItemConsumed { actor, item }),
      Event::ItemPickedUp { actor, item } => Some(Self::ItemPickedUp { actor, item }),
      Event::ItemDropped { .. }
      | Event::Reloaded { .. }
      | Event::DoorOpened { .. }
      | Event::NoiseCreated { .. }
      | Event::BreakableBroken { .. }
      | Event::TrapTriggered { .. } => None,
    }
  }
}

/// A disposable ordered buffer of typed audio placeholders derived from the latest runtime output.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationAudioCues {
  pub(crate) cues: Vec<PresentationAudioCue>,
}

impl PresentationAudioCues {
  /// Creates an empty audio-cue projection.
  #[must_use]
  pub const fn new() -> Self {
    Self { cues: Vec::new() }
  }

  /// Returns cues in the core event order of the latest runtime output.
  #[must_use]
  pub fn cues(&self) -> &[PresentationAudioCue] {
    &self.cues
  }
}

/// The family key used to bind one typed audio cue to a local-only asset reference.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationAudioCueKind {
  /// A successful movement cue.
  Moved,
  /// A blocked movement cue.
  MovementBlocked,
  /// A wait cue.
  Waited,
  /// An attack cue.
  Attacked,
  /// A death cue.
  Died,
  /// An equip cue.
  ItemEquipped,
  /// An unequip cue.
  ItemUnequipped,
  /// An item-consumption cue.
  ItemConsumed,
}

impl PresentationAudioCueKind {
  /// Derives the closed family key without inspecting or changing cue payloads.
  #[must_use]
  pub const fn from_cue(cue: PresentationAudioCue) -> Self {
    match cue {
      PresentationAudioCue::Moved { .. } => Self::Moved,
      PresentationAudioCue::MovementBlocked { .. } => Self::MovementBlocked,
      PresentationAudioCue::Waited { .. } => Self::Waited,
      PresentationAudioCue::Attacked { .. } => Self::Attacked,
      PresentationAudioCue::Died { .. } => Self::Died,
      PresentationAudioCue::ItemEquipped { .. } => Self::ItemEquipped,
      PresentationAudioCue::ItemUnequipped { .. } => Self::ItemUnequipped,
      // Reuse the existing item-consumption asset family without changing the typed cue identity.
      PresentationAudioCue::ItemConsumed { .. } | PresentationAudioCue::ItemPickedUp { .. } => {
        Self::ItemConsumed
      }
    }
  }

  const fn index(self) -> usize {
    match self {
      Self::Moved => 0,
      Self::MovementBlocked => 1,
      Self::Waited => 2,
      Self::Attacked => 3,
      Self::Died => 4,
      Self::ItemEquipped => 5,
      Self::ItemUnequipped => 6,
      Self::ItemConsumed => 7,
    }
  }
}

/// A complete mapping from typed audio cue families to local-only audio references.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct PresentationAudioAssetManifest {
  pub(crate) bindings: Vec<(PresentationAudioCueKind, PresentationAssetReference)>,
}

impl PresentationAudioAssetManifest {
  /// Creates a complete eight-family audio manifest, rejecting non-audio paths and duplicates.
  #[must_use]
  pub fn new(
    bindings: Vec<(PresentationAudioCueKind, PresentationAssetReference)>,
  ) -> Option<Self> {
    if bindings.len() != 8 {
      return None;
    }
    let mut seen = [false; 8];
    for (family, reference) in &bindings {
      if !reference.is_audio_path() {
        return None;
      }
      let slot = family.index();
      if seen[slot] {
        return None;
      }
      seen[slot] = true;
    }
    Some(Self { bindings })
  }

  /// Returns bindings in authored deterministic order.
  #[must_use]
  pub fn bindings(&self) -> &[(PresentationAudioCueKind, PresentationAssetReference)] {
    &self.bindings
  }

  /// Returns the validated audio reference for one closed cue family.
  ///
  /// # Panics
  ///
  /// Panics only if the private complete-manifest invariant has been violated. Every manifest
  /// constructed through [`Self::new`] contains all eight families, so valid callers cannot
  /// trigger this panic.
  #[must_use]
  pub fn reference(&self, family: PresentationAudioCueKind) -> &PresentationAssetReference {
    self
      .bindings
      .iter()
      .find(|(candidate, _)| *candidate == family)
      .map(|(_, reference)| reference)
      .expect("validated audio manifests contain every cue family")
  }
}

/// One typed audio cue joined with its validated local-only audio reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationAudioAssetEntry {
  pub(crate) cue: PresentationAudioCue,
  pub(crate) reference: PresentationAssetReference,
}

impl PresentationAudioAssetEntry {
  /// Returns the complete typed cue payload.
  #[must_use]
  pub fn cue(&self) -> PresentationAudioCue {
    self.cue
  }

  /// Returns the validated local-only audio reference.
  #[must_use]
  pub fn reference(&self) -> &PresentationAssetReference {
    &self.reference
  }
}

/// An ordered projection joining typed audio cues to local-only metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationAudioAssetProjection {
  pub(crate) entries: Vec<PresentationAudioAssetEntry>,
}

impl PresentationAudioAssetProjection {
  /// Creates an empty audio asset projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Derives a complete ordered projection without reading or loading any referenced file.
  #[must_use]
  pub fn from_cues(
    cues: &[PresentationAudioCue],
    manifest: &PresentationAudioAssetManifest,
  ) -> Self {
    let entries = cues
      .iter()
      .copied()
      .map(|cue| PresentationAudioAssetEntry {
        cue,
        reference: manifest
          .reference(PresentationAudioCueKind::from_cue(cue))
          .clone(),
      })
      .collect();
    Self { entries }
  }

  /// Returns entries in the source cue order.
  #[must_use]
  pub fn entries(&self) -> &[PresentationAudioAssetEntry] {
    &self.entries
  }
}
