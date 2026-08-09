//! Deterministic Bevy presentation boundary for Dreadstep.
//!
//! [`PresentationState`] owns a core world and translates human-client intent into the same
//! semantic commands used by headless and agent adapters. It deliberately exposes only immutable
//! projections and core outcomes; rendering, ECS storage, windowing, and presentation effects
//! remain outside authoritative game state.

#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use bevy::app::{App, Plugin, Update};
use bevy::color::Color;
use bevy::ecs::{
  component::Component,
  entity::Entity,
  query::{Or, With},
  resource::Resource,
  world::World,
};
use bevy::input::{ButtonInput, keyboard::KeyCode};
use bevy::math::Vec2;
use bevy::sprite::Sprite;
use dreadstep_content::{ContentError, starter_floor, starter_item_floor};
use dreadstep_core::{
  ActionTime, Actor, ActorId, ActorKind, BlockReason, Command, CommandError, Damage, Direction,
  Event, GridMap, GroundItemStack, HitPoints, Item, ItemDefinitionId, ItemId, Position,
  ReplayTrace, StateDigest, Tile, WorldState,
};

/// A supported keyboard intent before it is addressed to one core actor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KeyboardIntent {
  /// Move one tile in a cardinal direction.
  Move(Direction),
  /// Spend one standard action without moving.
  Wait,
}

impl KeyboardIntent {
  /// Converts supported arrow/WASD and wait keys into presentation intent.
  #[must_use]
  pub const fn from_key(key: KeyCode) -> Option<Self> {
    match key {
      KeyCode::ArrowUp | KeyCode::KeyW => Some(Self::Move(Direction::North)),
      KeyCode::ArrowDown | KeyCode::KeyS => Some(Self::Move(Direction::South)),
      KeyCode::ArrowLeft | KeyCode::KeyA => Some(Self::Move(Direction::West)),
      KeyCode::ArrowRight | KeyCode::KeyD => Some(Self::Move(Direction::East)),
      KeyCode::Enter | KeyCode::Space => Some(Self::Wait),
      _ => None,
    }
  }

  /// Addresses this intent to an explicit actor as a canonical core command.
  #[must_use]
  pub const fn command(self, actor: ActorId) -> Command {
    match self {
      Self::Move(direction) => Command::Move { actor, direction },
      Self::Wait => Command::Wait { actor },
    }
  }
}

/// Selects the core actor addressed by keyboard intents.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationInput {
  actor: ActorId,
}

impl PresentationInput {
  /// Creates keyboard control for one explicit actor identity.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self { actor }
  }

  /// Returns the actor addressed by keyboard intents.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }
}

/// A disposable focus projection for future camera and viewport systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationFocus {
  actor: ActorId,
  position: Option<Position>,
}

impl PresentationFocus {
  /// Creates an empty focus projection for one controlled actor.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      position: None,
    }
  }

  /// Returns the actor whose position is being projected.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the latest known core position, or `None` for an unknown actor.
  #[must_use]
  pub const fn position(self) -> Option<Position> {
    self.position
  }
}

/// A disposable camera-anchor projection for future viewport systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationCamera {
  actor: ActorId,
  center: Option<Position>,
}

impl PresentationCamera {
  /// Creates a camera anchor for one controlled actor before a runtime projection exists.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      center: None,
    }
  }

  /// Returns the actor whose position supplies this camera anchor.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the latest authoritative center, or `None` for an unknown actor.
  #[must_use]
  pub const fn center(self) -> Option<Position> {
    self.center
  }
}

/// A deterministic tile viewport requested by a presentation client.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationViewport {
  width: u32,
  height: u32,
  origin: Option<Position>,
  effective_width: u32,
  effective_height: u32,
}

impl PresentationViewport {
  /// Creates a non-empty viewport request.
  #[must_use]
  pub const fn new(width: u32, height: u32) -> Option<Self> {
    if width == 0 || height == 0 {
      return None;
    }
    Some(Self {
      width,
      height,
      origin: None,
      effective_width: 0,
      effective_height: 0,
    })
  }

  /// Returns the requested viewport width in tiles.
  #[must_use]
  pub const fn width(self) -> u32 {
    self.width
  }

  /// Returns the requested viewport height in tiles.
  #[must_use]
  pub const fn height(self) -> u32 {
    self.height
  }

  /// Returns the clamped row-major map origin, or `None` without an authoritative center.
  #[must_use]
  pub const fn origin(self) -> Option<Position> {
    self.origin
  }

  /// Returns the effective in-map width after clamping to the current map.
  #[must_use]
  pub const fn effective_width(self) -> u32 {
    self.effective_width
  }

  /// Returns the effective in-map height after clamping to the current map.
  #[must_use]
  pub const fn effective_height(self) -> u32 {
    self.effective_height
  }
}

/// A validated logical window request for a future desktop presentation client.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationWindow {
  logical_width: u32,
  logical_height: u32,
  pixel_scale: u32,
  physical_width: u32,
  physical_height: u32,
}

impl PresentationWindow {
  /// Creates a non-empty request with checked integer pixel dimensions.
  #[must_use]
  pub const fn new(logical_width: u32, logical_height: u32, pixel_scale: u32) -> Option<Self> {
    if logical_width == 0 || logical_height == 0 || pixel_scale == 0 {
      return None;
    }
    let Some(physical_width) = logical_width.checked_mul(pixel_scale) else {
      return None;
    };
    let Some(physical_height) = logical_height.checked_mul(pixel_scale) else {
      return None;
    };
    Some(Self {
      logical_width,
      logical_height,
      pixel_scale,
      physical_width,
      physical_height,
    })
  }

  /// Returns the logical width before pixel scaling.
  #[must_use]
  pub const fn logical_width(self) -> u32 {
    self.logical_width
  }

  /// Returns the logical height before pixel scaling.
  #[must_use]
  pub const fn logical_height(self) -> u32 {
    self.logical_height
  }

  /// Returns the integer scale from logical to physical pixels.
  #[must_use]
  pub const fn pixel_scale(self) -> u32 {
    self.pixel_scale
  }

  /// Returns the checked physical width.
  #[must_use]
  pub const fn physical_width(self) -> u32 {
    self.physical_width
  }

  /// Returns the checked physical height.
  #[must_use]
  pub const fn physical_height(self) -> u32 {
    self.physical_height
  }
}

/// A caller-selected logical tile extent for the future renderer.
///
/// The proposal keeps 24×24 and 32×32 as asset-experiment candidates, so this resource does not
/// choose a project-wide default. It only validates the dimensions supplied by a presentation
/// client and provides checked conversion from map coordinates to logical pixel coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationTileSize {
  width: u32,
  height: u32,
}

impl PresentationTileSize {
  /// Creates a non-empty logical tile extent.
  #[must_use]
  pub const fn new(width: u32, height: u32) -> Option<Self> {
    if width == 0 || height == 0 {
      return None;
    }
    Some(Self { width, height })
  }

  /// Returns the logical tile width.
  #[must_use]
  pub const fn width(self) -> u32 {
    self.width
  }

  /// Returns the logical tile height.
  #[must_use]
  pub const fn height(self) -> u32 {
    self.height
  }

  /// Converts an in-map coordinate into a checked logical-pixel origin.
  #[must_use]
  pub fn pixel_position(self, position: Position) -> Option<ScenePixelPosition> {
    let x = u32::try_from(position.x()).ok()?;
    let y = u32::try_from(position.y()).ok()?;
    Some(ScenePixelPosition {
      x: x.checked_mul(self.width)?,
      y: y.checked_mul(self.height)?,
    })
  }

  fn sprite_size(self) -> Vec2 {
    // Bevy's Sprite API stores custom dimensions as f32; the selected presentation tile sizes
    // are small logical pixel extents (24×24/32×32), so this adapter conversion is intentional.
    #[allow(clippy::cast_precision_loss)]
    {
      Vec2::new(self.width as f32, self.height as f32)
    }
  }
}

/// A typed status projection for a future HUD.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Resource)]
pub struct PresentationHud {
  actor: ActorId,
  kind: Option<ActorKind>,
  position: Option<Position>,
  hit_points: Option<HitPoints>,
  ready_at: Option<ActionTime>,
}

impl PresentationHud {
  /// Creates an empty HUD projection for one controlled actor identity.
  #[must_use]
  pub const fn new(actor: ActorId) -> Self {
    Self {
      actor,
      kind: None,
      position: None,
      hit_points: None,
      ready_at: None,
    }
  }

  /// Returns the actor whose status is being projected.
  #[must_use]
  pub const fn actor(self) -> ActorId {
    self.actor
  }

  /// Returns the controlled actor kind, or `None` for an unknown actor.
  #[must_use]
  pub const fn kind(self) -> Option<ActorKind> {
    self.kind
  }

  /// Returns the controlled actor position, or `None` for an unknown actor.
  #[must_use]
  pub const fn position(self) -> Option<Position> {
    self.position
  }

  /// Returns the controlled actor hit points, or `None` for an unknown actor.
  #[must_use]
  pub const fn hit_points(self) -> Option<HitPoints> {
    self.hit_points
  }

  /// Returns the controlled actor's next-ready time, or `None` for an unknown actor.
  #[must_use]
  pub const fn ready_at(self) -> Option<ActionTime> {
    self.ready_at
  }
}

/// A typed message projection of one core semantic event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationMessage {
  /// An actor entered a new map position.
  Moved {
    /// The actor that moved.
    actor: ActorId,
    /// The position before movement.
    from: Position,
    /// The position after movement.
    to: Position,
  },
  /// An actor attempted movement but remained in place.
  MovementBlocked {
    /// The actor that attempted movement.
    actor: ActorId,
    /// The position before the attempt.
    from: Position,
    /// The requested destination.
    to: Position,
    /// Why the destination could not be entered.
    reason: BlockReason,
  },
  /// An actor spent a standard action without moving.
  Waited {
    /// The actor that waited.
    actor: ActorId,
    /// The action time at which the wait began.
    at: ActionTime,
  },
  /// A melee attack reduced a target's hit points.
  Attacked {
    /// The actor that attacked.
    attacker: ActorId,
    /// The actor that was hit.
    target: ActorId,
    /// The fixed damage applied.
    damage: Damage,
    /// The target's hit points after damage.
    remaining_hit_points: HitPoints,
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
}

impl PresentationMessage {
  fn from_event(event: Event) -> Self {
    match event {
      Event::Moved { actor, from, to } => Self::Moved { actor, from, to },
      Event::MovementBlocked {
        actor,
        from,
        to,
        reason,
      } => Self::MovementBlocked {
        actor,
        from,
        to,
        reason,
      },
      Event::Waited { actor, at } => Self::Waited { actor, at },
      Event::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      } => Self::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      },
      Event::Died { actor } => Self::Died { actor },
      Event::ItemEquipped { actor, item } => Self::ItemEquipped { actor, item },
      Event::ItemUnequipped { actor, item } => Self::ItemUnequipped { actor, item },
      Event::ItemConsumed { actor, item } => Self::ItemConsumed { actor, item },
    }
  }
}

/// A disposable ordered buffer of typed messages derived from the latest runtime output.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationMessages {
  messages: Vec<PresentationMessage>,
}

impl PresentationMessages {
  /// Creates an empty message projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      messages: Vec::new(),
    }
  }

  /// Returns messages in the core event order of the latest runtime output.
  #[must_use]
  pub fn messages(&self) -> &[PresentationMessage] {
    &self.messages
  }
}

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
  /// A melee attack reduced a target's hit points.
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
}

impl PresentationAudioCue {
  fn from_event(event: Event) -> Self {
    match event {
      Event::Moved { actor, .. } => Self::Moved { actor },
      Event::MovementBlocked { actor, reason, .. } => Self::MovementBlocked { actor, reason },
      Event::Waited { actor, .. } => Self::Waited { actor },
      Event::Attacked {
        attacker, target, ..
      } => Self::Attacked { attacker, target },
      Event::Died { actor } => Self::Died { actor },
      Event::ItemEquipped { actor, item } => Self::ItemEquipped { actor, item },
      Event::ItemUnequipped { actor, item } => Self::ItemUnequipped { actor, item },
      Event::ItemConsumed { actor, item } => Self::ItemConsumed { actor, item },
    }
  }
}

/// A disposable ordered buffer of typed audio placeholders derived from the latest runtime output.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationAudioCues {
  cues: Vec<PresentationAudioCue>,
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
      PresentationAudioCue::ItemConsumed { .. } => Self::ItemConsumed,
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
  bindings: Vec<(PresentationAudioCueKind, PresentationAssetReference)>,
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
  cue: PresentationAudioCue,
  reference: PresentationAssetReference,
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
  entries: Vec<PresentationAudioAssetEntry>,
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

/// A typed animation signal derived from one core semantic event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationAnimationCue {
  /// An actor entered a new map position.
  Moved {
    /// The actor that moved.
    actor: ActorId,
    /// The position before movement.
    from: Position,
    /// The position after movement.
    to: Position,
  },
  /// An actor attempted movement but remained in place.
  MovementBlocked {
    /// The actor that attempted movement.
    actor: ActorId,
    /// The position before the attempt.
    from: Position,
    /// The requested destination.
    to: Position,
    /// Why the destination could not be entered.
    reason: BlockReason,
  },
  /// An actor spent a standard action without moving.
  Waited {
    /// The actor that waited.
    actor: ActorId,
    /// The action time at which the wait began.
    at: ActionTime,
  },
  /// A melee attack reduced a target's hit points.
  Attacked {
    /// The actor that attacked.
    attacker: ActorId,
    /// The actor that was hit.
    target: ActorId,
    /// The fixed damage applied.
    damage: Damage,
    /// The target's hit points after damage.
    remaining_hit_points: HitPoints,
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
}

impl PresentationAnimationCue {
  fn from_event(event: Event) -> Self {
    match event {
      Event::Moved { actor, from, to } => Self::Moved { actor, from, to },
      Event::MovementBlocked {
        actor,
        from,
        to,
        reason,
      } => Self::MovementBlocked {
        actor,
        from,
        to,
        reason,
      },
      Event::Waited { actor, at } => Self::Waited { actor, at },
      Event::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      } => Self::Attacked {
        attacker,
        target,
        damage,
        remaining_hit_points,
      },
      Event::Died { actor } => Self::Died { actor },
      Event::ItemEquipped { actor, item } => Self::ItemEquipped { actor, item },
      Event::ItemUnequipped { actor, item } => Self::ItemUnequipped { actor, item },
      Event::ItemConsumed { actor, item } => Self::ItemConsumed { actor, item },
    }
  }
}

/// A disposable ordered buffer of typed animation signals from the latest runtime output.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationAnimationCues {
  cues: Vec<PresentationAnimationCue>,
}

impl PresentationAnimationCues {
  /// Creates an empty animation-cue projection.
  #[must_use]
  pub const fn new() -> Self {
    Self { cues: Vec::new() }
  }

  /// Returns cues in the core event order of the latest runtime output.
  #[must_use]
  pub fn cues(&self) -> &[PresentationAnimationCue] {
    &self.cues
  }
}

/// A typed role for an existing scene mirror, consumed by a future sprite renderer.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneSpriteRole {
  /// A projected map tile, whose terrain remains in [`SceneTile`].
  Terrain,
  /// A living player actor, whose typed data remains in [`SceneActor`].
  Player,
  /// A living enemy actor, whose typed data remains in [`SceneActor`].
  Enemy,
  /// A retained dead actor record, whose typed data remains in [`SceneActor`].
  DeadActor,
  /// A ground item, whose typed data remains in [`SceneGroundItem`].
  GroundItem,
  /// An inventory item, whose typed data remains in [`SceneInventoryItem`].
  InventoryItem,
}

/// A stable typed content selector for one future sprite family.
///
/// These keys are metadata only: they do not name files, load assets, or imply a renderer. Item
/// selectors retain the opaque definition identity so a later content boundary can resolve it
/// without copying catalog data into the simulation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneSpriteKey {
  /// A terrain sprite family selected by the typed tile value.
  Terrain(Tile),
  /// The living player sprite family.
  Player,
  /// The living enemy sprite family.
  Enemy,
  /// The retained dead-actor sprite family.
  DeadActor,
  /// A ground-item sprite family selected by opaque definition identity.
  GroundItem(ItemDefinitionId),
  /// An inventory-item sprite family selected by opaque definition identity.
  InventoryItem(ItemDefinitionId),
}

impl SceneSpriteRole {
  fn for_actor(actor: &Actor) -> Self {
    if !actor.is_alive() {
      return Self::DeadActor;
    }
    match actor.kind() {
      ActorKind::Player => Self::Player,
      ActorKind::Enemy => Self::Enemy,
    }
  }

  fn for_scene_actor(actor: SceneActor) -> Self {
    if !actor.is_alive() {
      return Self::DeadActor;
    }
    match actor.kind() {
      ActorKind::Player => Self::Player,
      ActorKind::Enemy => Self::Enemy,
    }
  }
}

/// One deterministic, disposable entry prepared for a future renderer.
///
/// The entry copies the complete typed value from its keyed scene mirror and retains that mirror's
/// Bevy [`Entity`] so a renderer can preserve identity without treating this projection as authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneRenderEntry {
  /// A terrain mirror with optional logical-pixel placement metadata.
  Terrain {
    /// The retained keyed scene entity.
    entity: Entity,
    /// The complete typed terrain mirror.
    tile: SceneTile,
    /// The typed role consumed by a future sprite renderer.
    role: SceneSpriteRole,
    /// The logical-pixel origin, when tile-size configuration is available.
    pixel_position: Option<ScenePixelPosition>,
  },
  /// An actor mirror with optional logical-pixel placement metadata.
  Actor {
    /// The retained keyed scene entity.
    entity: Entity,
    /// The complete typed actor mirror.
    actor: SceneActor,
    /// The typed role consumed by a future sprite renderer.
    role: SceneSpriteRole,
    /// The logical-pixel origin, when tile-size configuration is available.
    pixel_position: Option<ScenePixelPosition>,
  },
  /// A ground-item mirror with optional logical-pixel placement metadata.
  GroundItem {
    /// The retained keyed scene entity.
    entity: Entity,
    /// The complete typed ground-item mirror.
    item: SceneGroundItem,
    /// The typed role consumed by a future sprite renderer.
    role: SceneSpriteRole,
    /// The logical-pixel origin, when tile-size configuration is available.
    pixel_position: Option<ScenePixelPosition>,
  },
  /// An inventory-item mirror that intentionally has no map-pixel placement.
  InventoryItem {
    /// The retained keyed scene entity.
    entity: Entity,
    /// The complete typed inventory-item mirror.
    item: SceneInventoryItem,
    /// The typed role consumed by a future sprite renderer.
    role: SceneSpriteRole,
  },
}

impl SceneRenderEntry {
  /// Returns the typed content selector derived from this complete render entry.
  #[must_use]
  pub const fn sprite_key(self) -> SceneSpriteKey {
    match self {
      Self::Terrain { tile, .. } => SceneSpriteKey::Terrain(tile.terrain()),
      Self::Actor { actor, .. } => {
        if actor.is_alive() {
          match actor.kind() {
            ActorKind::Player => SceneSpriteKey::Player,
            ActorKind::Enemy => SceneSpriteKey::Enemy,
          }
        } else {
          SceneSpriteKey::DeadActor
        }
      }
      Self::GroundItem { item, .. } => SceneSpriteKey::GroundItem(item.definition()),
      Self::InventoryItem { item, .. } => SceneSpriteKey::InventoryItem(item.definition()),
    }
  }
}

/// One ordered typed sprite selector retaining its complete render-boundary entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneSpriteEntry {
  render_entry: SceneRenderEntry,
  key: SceneSpriteKey,
}

impl SceneSpriteEntry {
  fn from_render_entry(render_entry: SceneRenderEntry) -> Self {
    Self {
      key: render_entry.sprite_key(),
      render_entry,
    }
  }

  /// Returns the complete render-boundary entry retained by this selector.
  #[must_use]
  pub const fn render_entry(self) -> SceneRenderEntry {
    self.render_entry
  }

  /// Returns the typed content selector.
  #[must_use]
  pub const fn key(self) -> SceneSpriteKey {
    self.key
  }

  /// Returns the retained ECS entity from the complete render entry.
  #[must_use]
  pub const fn entity(self) -> Entity {
    match self.render_entry {
      SceneRenderEntry::Terrain { entity, .. }
      | SceneRenderEntry::Actor { entity, .. }
      | SceneRenderEntry::GroundItem { entity, .. }
      | SceneRenderEntry::InventoryItem { entity, .. } => entity,
    }
  }
}

/// An ordered, read-only projection of typed sprite selectors for a future renderer.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationSpriteProjection {
  entries: Vec<SceneSpriteEntry>,
}

impl PresentationSpriteProjection {
  /// Creates an empty sprite-key projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Returns selectors in the same deterministic order as the render projection.
  #[must_use]
  pub fn entries(&self) -> &[SceneSpriteEntry] {
    &self.entries
  }
}

/// The deterministic draw layer assigned to one typed render command.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SceneRenderLayer {
  /// Map terrain, drawn below all entity and inventory commands.
  Terrain,
  /// Ground items, drawn above terrain and below actors.
  GroundItem,
  /// Living or dead actors, drawn above map items.
  Actor,
  /// Inventory entries, which remain unplaced metadata for a future inventory view.
  InventoryItem,
}

impl SceneRenderLayer {
  fn from_key(key: SceneSpriteKey) -> Self {
    match key {
      SceneSpriteKey::Terrain(_) => Self::Terrain,
      SceneSpriteKey::GroundItem(_) => Self::GroundItem,
      SceneSpriteKey::Player | SceneSpriteKey::Enemy | SceneSpriteKey::DeadActor => Self::Actor,
      SceneSpriteKey::InventoryItem(_) => Self::InventoryItem,
    }
  }
}

/// One deterministic draw command prepared from a complete sprite projection entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneRenderCommand {
  sprite: SceneSpriteEntry,
  layer: SceneRenderLayer,
  order: usize,
}

impl SceneRenderCommand {
  fn from_sprite_entry(sprite: SceneSpriteEntry, order: usize) -> Self {
    Self {
      layer: SceneRenderLayer::from_key(sprite.key()),
      sprite,
      order,
    }
  }

  /// Returns the complete sprite projection entry retained by this command.
  #[must_use]
  pub const fn sprite_entry(self) -> SceneSpriteEntry {
    self.sprite
  }

  /// Returns the typed draw layer for this command.
  #[must_use]
  pub const fn layer(self) -> SceneRenderLayer {
    self.layer
  }

  /// Returns the source order from the deterministic sprite projection.
  #[must_use]
  pub const fn order(self) -> usize {
    self.order
  }

  /// Returns the optional map placement, which is absent for inventory entries.
  #[must_use]
  pub const fn pixel_position(self) -> Option<ScenePixelPosition> {
    match self.sprite.render_entry() {
      SceneRenderEntry::Terrain { pixel_position, .. }
      | SceneRenderEntry::Actor { pixel_position, .. }
      | SceneRenderEntry::GroundItem { pixel_position, .. } => pixel_position,
      SceneRenderEntry::InventoryItem { .. } => None,
    }
  }
}

/// An ordered, read-only plan of typed draw commands for a future renderer.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationRenderCommandPlan {
  commands: Vec<SceneRenderCommand>,
}

impl PresentationRenderCommandPlan {
  /// Creates an empty render-command plan.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      commands: Vec::new(),
    }
  }

  /// Returns commands in deterministic layer order, retaining source order within each layer.
  #[must_use]
  pub fn commands(&self) -> &[SceneRenderCommand] {
    &self.commands
  }
}

/// The placeholder family used by the renderer bootstrap before production assets are loaded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneRenderPlaceholder {
  /// A terrain placeholder.
  Terrain,
  /// A living player placeholder.
  Player,
  /// A living enemy placeholder.
  Enemy,
  /// A retained dead-actor placeholder.
  DeadActor,
  /// A ground-item placeholder.
  GroundItem,
  /// An inventory-item placeholder.
  InventoryItem,
}

/// A validated relative path to a local-only presentation asset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationAssetReference {
  path: String,
}

impl PresentationAssetReference {
  /// Creates a non-empty path rooted in an ignored local-media directory, without traversal or
  /// platform prefixes.
  #[must_use]
  pub fn new(path: impl Into<String>) -> Option<Self> {
    let path = path.into();
    if path.is_empty()
      || path.contains('\\')
      || path.contains('\0')
      || path.starts_with('/')
      || path.starts_with(':')
      || path
        .split('/')
        .any(|segment| segment.is_empty() || segment == ".." || segment == ".")
      || path.split_once(':').is_some()
      || (!path.starts_with("assets/")
        && !path.starts_with("art/")
        && !path.starts_with("audio/")
        && !is_crate_local_media_path(&path))
    {
      return None;
    }
    Some(Self { path })
  }

  /// Returns the validated relative path without reading or loading it.
  #[must_use]
  pub fn path(&self) -> &str {
    &self.path
  }

  /// Returns whether this reference is rooted in a root or crate-local `audio/` directory.
  #[must_use]
  pub fn is_audio_path(&self) -> bool {
    self.path.starts_with("audio/")
      || matches!(crate_local_media_directory(&self.path), Some("audio"))
  }
}

/// A deterministic complete mapping from typed placeholder families to local asset references.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct PresentationAssetManifest {
  bindings: Vec<(SceneRenderPlaceholder, PresentationAssetReference)>,
}

impl PresentationAssetManifest {
  /// Creates a complete six-family manifest, rejecting duplicates or missing families.
  #[must_use]
  pub fn new(bindings: Vec<(SceneRenderPlaceholder, PresentationAssetReference)>) -> Option<Self> {
    if bindings.len() != 6 {
      return None;
    }
    let mut seen = [false; 6];
    for (family, _) in &bindings {
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
  pub fn bindings(&self) -> &[(SceneRenderPlaceholder, PresentationAssetReference)] {
    &self.bindings
  }

  /// Returns the local-only reference for one placeholder family.
  ///
  /// # Panics
  ///
  /// Panics only if the private complete-manifest invariant has been violated. Every manifest
  /// constructed through [`Self::new`] contains all six families, so valid callers cannot trigger
  /// this panic.
  #[must_use]
  pub fn reference(&self, family: SceneRenderPlaceholder) -> &PresentationAssetReference {
    self
      .bindings
      .iter()
      .find(|(candidate, _)| *candidate == family)
      .map(|(_, reference)| reference)
      .expect("validated asset manifests contain every placeholder family")
  }
}

fn crate_local_media_directory(path: &str) -> Option<&str> {
  let crate_relative = path.strip_prefix("crates/")?;
  let mut segments = crate_relative.split('/');
  let crate_name = segments.next();
  let media_directory = segments.next();
  let asset_path = segments.next();
  if crate_name.is_some() && asset_path.is_some() {
    media_directory
  } else {
    None
  }
}

fn is_crate_local_media_path(path: &str) -> bool {
  matches!(
    crate_local_media_directory(path),
    Some("assets" | "art" | "audio")
  )
}

impl SceneRenderPlaceholder {
  const fn index(self) -> usize {
    match self {
      Self::Terrain => 0,
      Self::Player => 1,
      Self::Enemy => 2,
      Self::DeadActor => 3,
      Self::GroundItem => 4,
      Self::InventoryItem => 5,
    }
  }
}

impl SceneRenderPlaceholder {
  fn from_key(key: SceneSpriteKey) -> Self {
    match key {
      SceneSpriteKey::Terrain(_) => Self::Terrain,
      SceneSpriteKey::Player => Self::Player,
      SceneSpriteKey::Enemy => Self::Enemy,
      SceneSpriteKey::DeadActor => Self::DeadActor,
      SceneSpriteKey::GroundItem(_) => Self::GroundItem,
      SceneSpriteKey::InventoryItem(_) => Self::InventoryItem,
    }
  }
}

/// A stable ECS placeholder node reconciled from one typed render command.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneRenderNode {
  source_entity: Entity,
  key: SceneSpriteKey,
  layer: SceneRenderLayer,
  order: usize,
  pixel_position: Option<ScenePixelPosition>,
  placeholder: SceneRenderPlaceholder,
}

impl SceneRenderNode {
  fn from_command(command: SceneRenderCommand) -> Self {
    let sprite = command.sprite_entry();
    Self {
      source_entity: sprite.entity(),
      key: sprite.key(),
      layer: command.layer(),
      order: command.order(),
      pixel_position: command.pixel_position(),
      placeholder: SceneRenderPlaceholder::from_key(sprite.key()),
    }
  }

  /// Returns the authoritative scene mirror entity represented by this node.
  #[must_use]
  pub const fn source_entity(self) -> Entity {
    self.source_entity
  }

  /// Returns the typed sprite selector represented by this node.
  #[must_use]
  pub const fn key(self) -> SceneSpriteKey {
    self.key
  }

  /// Returns the deterministic draw layer.
  #[must_use]
  pub const fn layer(self) -> SceneRenderLayer {
    self.layer
  }

  /// Returns the retained source order within the render command plan.
  #[must_use]
  pub const fn order(self) -> usize {
    self.order
  }

  /// Returns optional checked map placement, absent for inventory entries.
  #[must_use]
  pub const fn pixel_position(self) -> Option<ScenePixelPosition> {
    self.pixel_position
  }

  /// Returns the placeholder family used until production assets are available.
  #[must_use]
  pub const fn placeholder(self) -> SceneRenderPlaceholder {
    self.placeholder
  }
}

/// One read-only projection entry for a reconciled placeholder render node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneRenderNodeEntry {
  node_entity: Entity,
  node: SceneRenderNode,
}

impl SceneRenderNodeEntry {
  /// Returns the stable ECS entity carrying the placeholder node.
  #[must_use]
  pub const fn node_entity(self) -> Entity {
    self.node_entity
  }

  /// Returns the typed node metadata.
  #[must_use]
  pub const fn node(self) -> SceneRenderNode {
    self.node
  }
}

/// An ordered, read-only projection of deterministic placeholder render nodes.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationRenderNodeProjection {
  entries: Vec<SceneRenderNodeEntry>,
}

impl PresentationRenderNodeProjection {
  /// Creates an empty placeholder-node projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Returns placeholder nodes in layer/source-order command order.
  #[must_use]
  pub fn entries(&self) -> &[SceneRenderNodeEntry] {
    &self.entries
  }
}

/// One placeholder [`Sprite`] value joined to a stable render-node entry.
///
/// The Sprite uses a deterministic solid color and an unset/default image handle. It is a
/// headless API value for a later renderer; no Sprite plugin, texture loading, or transform is
/// installed by this projection.
#[derive(Clone, Debug)]
pub struct SceneBevySpriteEntry {
  node: SceneRenderNodeEntry,
  sprite: Sprite,
}

impl SceneBevySpriteEntry {
  /// Returns the stable render-node metadata retained by this Sprite value.
  #[must_use]
  pub const fn node(&self) -> SceneRenderNodeEntry {
    self.node
  }

  /// Returns the deterministic placeholder Sprite value.
  #[must_use]
  pub const fn sprite(&self) -> &Sprite {
    &self.sprite
  }
}

impl PartialEq for SceneBevySpriteEntry {
  fn eq(&self, other: &Self) -> bool {
    self.node == other.node
      && self.sprite.image == other.sprite.image
      && self.sprite.texture_atlas == other.sprite.texture_atlas
      && self.sprite.color == other.sprite.color
      && self.sprite.flip_x == other.sprite.flip_x
      && self.sprite.flip_y == other.sprite.flip_y
      && self.sprite.custom_size == other.sprite.custom_size
      && self.sprite.rect == other.sprite.rect
      && self.sprite.image_mode == other.sprite.image_mode
  }
}

/// An ordered, read-only projection of Bevy Sprite API values for a future renderer.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct PresentationBevySpriteProjection {
  entries: Vec<SceneBevySpriteEntry>,
}

impl PresentationBevySpriteProjection {
  /// Creates an empty headless Sprite projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Returns Sprite values in the deterministic render-node order.
  #[must_use]
  pub fn entries(&self) -> &[SceneBevySpriteEntry] {
    &self.entries
  }
}

/// One render-node entry joined with its validated local-only asset reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneRenderAssetEntry {
  node: SceneRenderNodeEntry,
  reference: PresentationAssetReference,
}

impl SceneRenderAssetEntry {
  /// Returns the reconciled placeholder node.
  #[must_use]
  pub const fn node(&self) -> SceneRenderNodeEntry {
    self.node
  }

  /// Returns the validated local-only reference for this node's placeholder family.
  #[must_use]
  pub fn reference(&self) -> &PresentationAssetReference {
    &self.reference
  }
}

/// An ordered, read-only projection joining placeholder nodes to local-only asset metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationRenderAssetProjection {
  entries: Vec<SceneRenderAssetEntry>,
}

impl PresentationRenderAssetProjection {
  /// Creates an empty asset-reference projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Returns joined entries in node order without loading any referenced file.
  #[must_use]
  pub fn entries(&self) -> &[SceneRenderAssetEntry] {
    &self.entries
  }
}

/// An ordered, read-only projection for a future renderer boundary.
///
/// Entries are derived from the current keyed scene mirrors after scene and pixel synchronization.
/// This resource never mutates or replaces core authority, runtime snapshots, replay history, or
/// the scene mirrors themselves.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct PresentationRenderProjection {
  entries: Vec<SceneRenderEntry>,
}

impl PresentationRenderProjection {
  /// Creates an empty render-boundary projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Returns entries in deterministic terrain/actor/ground/inventory key order.
  #[must_use]
  pub fn entries(&self) -> &[SceneRenderEntry] {
    &self.entries
  }
}

/// A disposable logical-pixel origin for a map-backed scene mirror.
///
/// This is placement metadata only. It is not a Bevy transform and does not imply a window,
/// camera, texture, asset handle, or rendering plugin.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScenePixelPosition {
  x: u32,
  y: u32,
}

impl ScenePixelPosition {
  /// Returns the logical horizontal pixel origin.
  #[must_use]
  pub const fn x(self) -> u32 {
    self.x
  }

  /// Returns the logical vertical pixel origin.
  #[must_use]
  pub const fn y(self) -> u32 {
    self.y
  }
}

/// A disposable ECS mirror of one projected map tile.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneTile {
  position: dreadstep_core::Position,
  terrain: Tile,
}

impl SceneTile {
  fn new(position: dreadstep_core::Position, terrain: Tile) -> Self {
    Self { position, terrain }
  }

  /// Returns the core position represented by this scene tile.
  #[must_use]
  pub const fn position(self) -> dreadstep_core::Position {
    self.position
  }

  /// Returns the projected terrain value.
  #[must_use]
  pub const fn terrain(self) -> Tile {
    self.terrain
  }
}

/// A disposable ECS mirror of one projected actor record.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneActor {
  id: ActorId,
  kind: dreadstep_core::ActorKind,
  position: dreadstep_core::Position,
  hit_points: dreadstep_core::HitPoints,
  ready_at: dreadstep_core::ActionTime,
  equipped_item: Option<ItemId>,
  alive: bool,
}

impl SceneActor {
  fn from_core(actor: &Actor) -> Self {
    Self {
      id: actor.id(),
      kind: actor.kind(),
      position: actor.position(),
      hit_points: actor.hit_points(),
      ready_at: actor.ready_at(),
      equipped_item: actor.equipped_item(),
      alive: actor.is_alive(),
    }
  }

  /// Returns the stable actor identity.
  #[must_use]
  pub const fn id(self) -> ActorId {
    self.id
  }

  /// Returns the actor kind.
  #[must_use]
  pub const fn kind(self) -> dreadstep_core::ActorKind {
    self.kind
  }

  /// Returns the projected actor position.
  #[must_use]
  pub const fn position(self) -> dreadstep_core::Position {
    self.position
  }

  /// Returns the projected hit points.
  #[must_use]
  pub const fn hit_points(self) -> dreadstep_core::HitPoints {
    self.hit_points
  }

  /// Returns the projected core scheduler readiness time.
  #[must_use]
  pub const fn ready_at(self) -> dreadstep_core::ActionTime {
    self.ready_at
  }

  /// Returns the optional equipped item identity, which remains in the actor inventory mirror.
  #[must_use]
  pub const fn equipped_item(self) -> Option<ItemId> {
    self.equipped_item
  }

  /// Returns whether the projected actor is living.
  #[must_use]
  pub const fn is_alive(self) -> bool {
    self.alive
  }
}

/// A disposable ECS mirror of one opaque item projected on the ground.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneGroundItem {
  id: ItemId,
  definition: ItemDefinitionId,
  position: Position,
  stack_index: usize,
}

impl SceneGroundItem {
  fn from_core(position: Position, stack_index: usize, item: Item) -> Self {
    Self {
      id: item.id(),
      definition: item.definition(),
      position,
      stack_index,
    }
  }

  /// Returns the globally unique item identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the opaque content reference carried by this item instance.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }

  /// Returns the map position where this item is projected.
  #[must_use]
  pub const fn position(self) -> Position {
    self.position
  }

  /// Returns this item's zero-based insertion-order index within its ground stack.
  #[must_use]
  pub const fn stack_index(self) -> usize {
    self.stack_index
  }
}

/// A disposable ECS mirror of one opaque item in an actor inventory.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneInventoryItem {
  id: ItemId,
  owner: ActorId,
  definition: ItemDefinitionId,
  inventory_index: usize,
}

impl SceneInventoryItem {
  fn from_core(owner: ActorId, inventory_index: usize, item: Item) -> Self {
    Self {
      id: item.id(),
      owner,
      definition: item.definition(),
      inventory_index,
    }
  }

  /// Returns the globally unique item identity.
  #[must_use]
  pub const fn id(self) -> ItemId {
    self.id
  }

  /// Returns the actor that currently owns this item instance.
  #[must_use]
  pub const fn owner(self) -> ActorId {
    self.owner
  }

  /// Returns the opaque content reference carried by this item instance.
  #[must_use]
  pub const fn definition(self) -> ItemDefinitionId {
    self.definition
  }

  /// Returns this item's zero-based insertion-order index in its owner's inventory.
  #[must_use]
  pub const fn inventory_index(self) -> usize {
    self.inventory_index
  }
}

/// A marker for the keyed scene entity representing the selected actor.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SceneFocus;

/// A disposable ECS mirror of the projected camera center.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneCamera {
  center: Position,
}

impl SceneCamera {
  /// Creates a disposable camera projection for a known core center.
  #[must_use]
  pub const fn new(center: Position) -> Self {
    Self { center }
  }

  /// Returns the authoritative map position represented by this camera anchor.
  #[must_use]
  pub const fn center(self) -> Position {
    self.center
  }
}

#[derive(Default, Resource)]
struct SceneCameraState {
  entity: Option<Entity>,
}

/// A disposable ECS mirror of one effective in-map viewport rectangle.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneViewport {
  origin: Position,
  width: u32,
  height: u32,
}

impl SceneViewport {
  const fn new(origin: Position, width: u32, height: u32) -> Self {
    Self {
      origin,
      width,
      height,
    }
  }

  /// Returns the row-major map origin of this viewport.
  #[must_use]
  pub const fn origin(self) -> Position {
    self.origin
  }

  /// Returns the effective viewport width in tiles.
  #[must_use]
  pub const fn width(self) -> u32 {
    self.width
  }

  /// Returns the effective viewport height in tiles.
  #[must_use]
  pub const fn height(self) -> u32 {
    self.height
  }
}

#[derive(Default, Resource)]
struct SceneViewportState {
  entity: Option<Entity>,
}

/// A deterministic read-only projection consumed by future map, actor, ground-item, and inventory
/// renderers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationSnapshot {
  width: u32,
  height: u32,
  tiles: Vec<Tile>,
  actors: Vec<Actor>,
  ground_items: Vec<GroundItemStack>,
  current_time: ActionTime,
  next_actor: Option<ActorId>,
  digest: StateDigest,
}

impl PresentationSnapshot {
  fn from_world(world: &WorldState) -> Self {
    Self {
      width: world.map().width(),
      height: world.map().height(),
      tiles: world.map().tiles().to_vec(),
      actors: world.actors().cloned().collect(),
      ground_items: world.ground_items().to_vec(),
      current_time: world.current_time(),
      next_actor: world.next_actor(),
      digest: world.digest(),
    }
  }

  /// Returns the map width in tiles.
  #[must_use]
  pub const fn width(&self) -> u32 {
    self.width
  }

  /// Returns the map height in tiles.
  #[must_use]
  pub const fn height(&self) -> u32 {
    self.height
  }

  /// Returns immutable row-major terrain for the projected map.
  #[must_use]
  pub fn tiles(&self) -> &[Tile] {
    &self.tiles
  }

  /// Returns immutable actor records in stable [`ActorId`] order.
  #[must_use]
  pub fn actors(&self) -> &[Actor] {
    &self.actors
  }

  /// Returns immutable ground-item stacks in core-provided row-major and insertion order.
  #[must_use]
  pub fn ground_items(&self) -> &[GroundItemStack] {
    &self.ground_items
  }

  /// Returns the current core action time.
  #[must_use]
  pub const fn current_time(&self) -> ActionTime {
    self.current_time
  }

  /// Returns the actor currently selected by core scheduling.
  #[must_use]
  pub const fn next_actor(&self) -> Option<ActorId> {
    self.next_actor
  }

  /// Returns the stable core state digest for this projection.
  #[must_use]
  pub const fn digest(&self) -> StateDigest {
    self.digest
  }
}

/// Evidence returned after one accepted presentation command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationOutput {
  events: Vec<Event>,
  snapshot: PresentationSnapshot,
}

impl PresentationOutput {
  /// Returns semantic core events in deterministic execution order.
  #[must_use]
  pub fn events(&self) -> &[Event] {
    &self.events
  }

  /// Returns the post-command presentation projection.
  #[must_use]
  pub const fn snapshot(&self) -> &PresentationSnapshot {
    &self.snapshot
  }
}

/// A deterministic presentation adapter around one core world and replay trace.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresentationState {
  seed: u64,
  world: WorldState,
  trace: ReplayTrace,
}

impl PresentationState {
  /// Starts a presentation state from the shared authored starter floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core validation.
  pub fn start_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, starter_floor()?))
  }

  /// Starts a presentation state from the shared authored starter-item scenario.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored item floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self::new(seed, starter_item_floor()?))
  }

  /// Creates a presentation state around an already validated core world.
  #[must_use]
  pub fn new(seed: u64, world: WorldState) -> Self {
    Self {
      seed,
      world,
      trace: ReplayTrace::new(seed),
    }
  }

  /// Returns the explicit run seed preserved by this presentation boundary.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.seed
  }

  /// Returns a stable read-only projection of the current core world.
  #[must_use]
  pub fn snapshot(&self) -> PresentationSnapshot {
    PresentationSnapshot::from_world(&self.world)
  }

  /// Returns the deterministic digest of accepted presentation commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.trace.digest()
  }

  /// Executes one canonical command through core and projects its semantic outcome.
  ///
  /// Rejected commands are not recorded. Core validates scheduling and gameplay, so this adapter
  /// does not duplicate those rules or mutate presentation state on an error.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is not accepted.
  pub fn execute(&mut self, command: Command) -> Result<PresentationOutput, CommandError> {
    let result = self.world.execute(command)?;
    self.trace.record(command);
    Ok(PresentationOutput {
      events: result.events().to_vec(),
      snapshot: self.snapshot(),
    })
  }

  /// Returns the immutable core map for adapters that need map-specific inspection.
  #[must_use]
  pub const fn map(&self) -> &GridMap {
    self.world.map()
  }
}

/// The Bevy-owned handle for one deterministic presentation run.
///
/// The wrapped [`PresentationState`] remains the only source of simulation truth. Bevy systems
/// may read snapshots or submit explicit core commands through this resource, while ECS scene
/// components remain disposable projections.
#[derive(Debug, Eq, PartialEq, Resource)]
pub struct PresentationRuntime {
  state: PresentationState,
  output: Option<PresentationOutput>,
}

impl PresentationRuntime {
  /// Starts a runtime from the shared authored starter floor.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored floor fails core validation.
  pub fn start_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_run(seed)?,
      output: None,
    })
  }

  /// Starts a runtime from the shared authored starter-item scenario.
  ///
  /// # Errors
  ///
  /// Returns [`ContentError`] when the authored item floor fails core or catalog validation.
  pub fn start_item_run(seed: u64) -> Result<Self, ContentError> {
    Ok(Self {
      state: PresentationState::start_item_run(seed)?,
      output: None,
    })
  }

  /// Wraps an already validated presentation state as an app resource.
  #[must_use]
  pub fn new(state: PresentationState) -> Self {
    Self {
      state,
      output: None,
    }
  }

  /// Returns the explicit seed preserved by the runtime.
  #[must_use]
  pub const fn seed(&self) -> u64 {
    self.state.seed()
  }

  /// Returns a read-only snapshot of the authoritative core world.
  #[must_use]
  pub fn snapshot(&self) -> PresentationSnapshot {
    self.state.snapshot()
  }

  /// Returns the deterministic digest of accepted runtime commands.
  #[must_use]
  pub fn replay_digest(&self) -> StateDigest {
    self.state.replay_digest()
  }

  /// Returns the latest accepted command output without consuming it.
  #[must_use]
  pub const fn output(&self) -> Option<&PresentationOutput> {
    self.output.as_ref()
  }

  /// Takes the latest accepted command output, if one is pending.
  pub fn take_output(&mut self) -> Option<PresentationOutput> {
    self.output.take()
  }

  /// Delegates one command to the wrapped presentation state and core simulation.
  ///
  /// # Errors
  ///
  /// Returns the core [`CommandError`] when the command is rejected. Rejected commands do not
  /// mutate the core world or replay trace, and clear any stale output so consumers never observe
  /// an earlier command as feedback for a rejected one.
  pub fn execute(&mut self, command: Command) -> Result<PresentationOutput, CommandError> {
    self.output = None;
    let output = self.state.execute(command)?;
    self.output = Some(output.clone());
    Ok(output)
  }
}

/// A headless Bevy plugin that keeps disposable scene mirrors synchronized with runtime state.
///
/// The plugin expects [`PresentationRuntime`] to be inserted by the application. Until a runtime
/// is present, its update system is a safe no-op so app construction can install plugins before
/// selecting or restoring a run. Keyboard dispatch is also optional: it runs only when the app
/// provides [`PresentationInput`] and `ButtonInput<KeyCode>` resources.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
  fn build(&self, app: &mut App) {
    app.add_systems(Update, update_presentation);
  }
}

const KEY_PRIORITY: [KeyCode; 10] = [
  KeyCode::ArrowUp,
  KeyCode::ArrowDown,
  KeyCode::ArrowLeft,
  KeyCode::ArrowRight,
  KeyCode::KeyW,
  KeyCode::KeyS,
  KeyCode::KeyA,
  KeyCode::KeyD,
  KeyCode::Enter,
  KeyCode::Space,
];

fn update_presentation(world: &mut World) {
  dispatch_keyboard_input(world);
  sync_runtime_scene(world);
  sync_scene_pixel_positions(world);
  sync_render_projection(world);
  sync_sprite_projection(world);
  sync_render_command_plan(world);
  sync_render_nodes(world);
  sync_bevy_sprite_projection(world);
  sync_render_asset_projection(world);
  sync_focus(world);
  sync_scene_focus(world);
  sync_camera(world);
  sync_scene_camera(world);
  sync_viewport(world);
  sync_scene_viewport(world);
  sync_hud(world);
  sync_messages(world);
  sync_audio_cues(world);
  sync_audio_asset_projection(world);
  sync_animation_cues(world);
}

fn dispatch_keyboard_input(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(key) = world
    .get_resource::<ButtonInput<KeyCode>>()
    .and_then(|input| {
      KEY_PRIORITY
        .iter()
        .copied()
        .find(|key| input.just_pressed(*key))
    })
  else {
    return;
  };
  let Some(intent) = KeyboardIntent::from_key(key) else {
    return;
  };
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  {
    let Some(mut input) = world.get_resource_mut::<ButtonInput<KeyCode>>() else {
      return;
    };
    for supported_key in KEY_PRIORITY {
      input.clear_just_pressed(supported_key);
    }
  }
  let _ = world
    .resource_mut::<PresentationRuntime>()
    .execute(intent.command(actor));
}

fn sync_runtime_scene(world: &mut World) {
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  sync_scene(world, &snapshot);
}

fn sync_scene_pixel_positions(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let Some(tile_size) = world.get_resource::<PresentationTileSize>().copied() else {
    return;
  };
  let placements = {
    let mut query = world.query_filtered::<(
      Entity,
      Option<&SceneTile>,
      Option<&SceneActor>,
      Option<&SceneGroundItem>,
    ), Or<(With<SceneTile>, With<SceneActor>, With<SceneGroundItem>)>>();
    query
      .iter(world)
      .map(|(entity, tile, actor, ground_item)| {
        let position = tile
          .map(|tile| tile.position())
          .or_else(|| actor.map(|actor| actor.position()))
          .or_else(|| ground_item.map(|item| item.position()));
        (
          entity,
          position.and_then(|position| tile_size.pixel_position(position)),
        )
      })
      .collect::<Vec<_>>()
  };
  for (entity, pixel_position) in placements {
    let mut entity = world.entity_mut(entity);
    if let Some(pixel_position) = pixel_position {
      entity.insert(pixel_position);
    } else {
      entity.remove::<ScenePixelPosition>();
    }
  }
}

fn sync_render_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let entries = {
    let tile_size = world.get_resource::<PresentationTileSize>().copied();
    let mut query = world.query::<(
      Entity,
      Option<&SceneTile>,
      Option<&SceneActor>,
      Option<&SceneGroundItem>,
      Option<&SceneInventoryItem>,
      Option<&ScenePixelPosition>,
    )>();
    let mut keyed: BTreeMap<_, (Entity, SceneRenderEntry)> = BTreeMap::new();
    for (entity, tile, actor, ground_item, inventory_item, pixel_position) in query.iter(world) {
      for (key, entry) in render_entries(
        entity,
        tile.copied(),
        actor.copied(),
        ground_item.copied(),
        inventory_item.copied(),
        pixel_position.copied(),
        tile_size,
      ) {
        match keyed.entry(key) {
          std::collections::btree_map::Entry::Occupied(mut retained) => {
            if entity < retained.get().0 {
              retained.insert((entity, entry));
            }
          }
          std::collections::btree_map::Entry::Vacant(slot) => {
            slot.insert((entity, entry));
          }
        }
      }
    }
    keyed
      .into_values()
      .map(|(_, entry)| entry)
      .collect::<Vec<_>>()
  };
  let Some(mut projection) = world.get_resource_mut::<PresentationRenderProjection>() else {
    return;
  };
  projection.entries = entries;
}

fn sync_sprite_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let entries = world
    .get_resource::<PresentationRenderProjection>()
    .map(|projection| {
      projection
        .entries()
        .iter()
        .copied()
        .map(SceneSpriteEntry::from_render_entry)
        .collect::<Vec<_>>()
    });
  let Some(entries) = entries else {
    return;
  };
  let Some(mut projection) = world.get_resource_mut::<PresentationSpriteProjection>() else {
    return;
  };
  projection.entries = entries;
}

fn sync_render_command_plan(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none() {
    return;
  }
  let commands = world
    .get_resource::<PresentationSpriteProjection>()
    .map(|projection| {
      let mut commands = projection
        .entries()
        .iter()
        .copied()
        .enumerate()
        .map(|(order, entry)| SceneRenderCommand::from_sprite_entry(entry, order))
        .collect::<Vec<_>>();
      commands.sort_by_key(|command| (command.layer(), command.order()));
      commands
    });
  let Some(commands) = commands else {
    return;
  };
  let Some(mut plan) = world.get_resource_mut::<PresentationRenderCommandPlan>() else {
    return;
  };
  plan.commands = commands;
}

fn sync_render_nodes(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world
      .get_resource::<PresentationRenderCommandPlan>()
      .is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
  {
    return;
  }
  let commands = world
    .resource::<PresentationRenderCommandPlan>()
    .commands()
    .to_vec();
  let existing = {
    let mut query = world.query::<(Entity, &SceneRenderNode)>();
    query
      .iter(world)
      .map(|(entity, node)| (entity, *node))
      .collect::<Vec<_>>()
  };
  let mut retained = Vec::new();
  let mut entries = Vec::with_capacity(commands.len());
  for command in commands {
    let node = SceneRenderNode::from_command(command);
    let retained_entity = existing
      .iter()
      .find(|(entity, existing_node)| {
        !retained.contains(entity)
          && existing_node.source_entity() == node.source_entity()
          && existing_node.layer() == node.layer()
      })
      .map(|(entity, _)| *entity);
    let node_entity = retained_entity.unwrap_or_else(|| world.spawn(node).id());
    world.entity_mut(node_entity).insert(node);
    retained.push(node_entity);
    entries.push(SceneRenderNodeEntry { node_entity, node });
  }
  for (entity, _) in existing {
    if !retained.contains(&entity) {
      let _ = world.despawn(entity);
    }
  }
  world
    .resource_mut::<PresentationRenderNodeProjection>()
    .entries = entries;
}

fn sync_bevy_sprite_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
    || world
      .get_resource::<PresentationBevySpriteProjection>()
      .is_none()
  {
    return;
  }
  let tile_size = world.get_resource::<PresentationTileSize>().copied();
  let entries = world
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .iter()
    .copied()
    .map(|node| SceneBevySpriteEntry {
      sprite: placeholder_sprite(node.node().placeholder(), tile_size),
      node,
    })
    .collect::<Vec<_>>();
  world
    .resource_mut::<PresentationBevySpriteProjection>()
    .entries = entries;
}

fn placeholder_sprite(
  placeholder: SceneRenderPlaceholder,
  tile_size: Option<PresentationTileSize>,
) -> Sprite {
  let color = match placeholder {
    SceneRenderPlaceholder::Terrain => Color::srgb(0.18, 0.18, 0.18),
    SceneRenderPlaceholder::Player => Color::srgb(0.1, 0.8, 0.3),
    SceneRenderPlaceholder::Enemy => Color::srgb(0.8, 0.2, 0.2),
    SceneRenderPlaceholder::DeadActor => Color::srgb(0.35, 0.35, 0.35),
    SceneRenderPlaceholder::GroundItem => Color::srgb(0.8, 0.65, 0.15),
    SceneRenderPlaceholder::InventoryItem => Color::srgb(0.2, 0.5, 0.9),
  };
  Sprite {
    color,
    custom_size: tile_size.map(PresentationTileSize::sprite_size),
    ..Default::default()
  }
}

fn sync_render_asset_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world
      .get_resource::<PresentationRenderNodeProjection>()
      .is_none()
    || world.get_resource::<PresentationAssetManifest>().is_none()
  {
    return;
  }
  let nodes = world
    .resource::<PresentationRenderNodeProjection>()
    .entries()
    .to_vec();
  let manifest = world.resource::<PresentationAssetManifest>();
  let entries = nodes
    .into_iter()
    .map(|node| SceneRenderAssetEntry {
      reference: manifest.reference(node.node().placeholder()).clone(),
      node,
    })
    .collect::<Vec<_>>();
  let Some(mut projection) = world.get_resource_mut::<PresentationRenderAssetProjection>() else {
    return;
  };
  projection.entries = entries;
}

fn render_entries(
  entity: Entity,
  tile: Option<SceneTile>,
  actor: Option<SceneActor>,
  ground_item: Option<SceneGroundItem>,
  inventory_item: Option<SceneInventoryItem>,
  pixel_position: Option<ScenePixelPosition>,
  tile_size: Option<PresentationTileSize>,
) -> Vec<((u8, i32, i32, u32), SceneRenderEntry)> {
  let pixel_position_for = |position: Position| {
    tile_size.map_or(pixel_position, |tile_size| {
      tile_size.pixel_position(position)
    })
  };
  let mut entries = Vec::new();
  if let Some(tile) = tile {
    entries.push((
      (0, tile.position().x(), tile.position().y(), 0),
      SceneRenderEntry::Terrain {
        entity,
        tile,
        role: SceneSpriteRole::Terrain,
        pixel_position: pixel_position_for(tile.position()),
      },
    ));
  }
  if let Some(actor) = actor {
    entries.push((
      (1, 0, 0, actor.id().value()),
      SceneRenderEntry::Actor {
        entity,
        actor,
        role: SceneSpriteRole::for_scene_actor(actor),
        pixel_position: pixel_position_for(actor.position()),
      },
    ));
  }
  if let Some(item) = ground_item {
    entries.push((
      (2, 0, 0, item.id().value()),
      SceneRenderEntry::GroundItem {
        entity,
        item,
        role: SceneSpriteRole::GroundItem,
        pixel_position: pixel_position_for(item.position()),
      },
    ));
  }
  if let Some(item) = inventory_item {
    entries.push((
      (3, 0, 0, item.id().value()),
      SceneRenderEntry::InventoryItem {
        entity,
        item,
        role: SceneSpriteRole::InventoryItem,
      },
    ));
  }
  entries
}

fn sync_focus(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let position = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position);
  let Some(mut focus) = world.get_resource_mut::<PresentationFocus>() else {
    return;
  };
  focus.actor = actor;
  focus.position = position;
}

fn sync_scene_focus(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  if world.get_resource::<PresentationFocus>().is_none() {
    return;
  }
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let actor_is_known = snapshot.actors().iter().any(|record| record.id() == actor);
  let (marked_entities, target_entity) = {
    let mut query = world.query::<(Entity, Option<&SceneActor>, Option<&SceneFocus>)>();
    query.iter(world).fold(
      (Vec::new(), None),
      |(mut marked_entities, target_entity), (entity, scene_actor, marker)| {
        if marker.is_some() {
          marked_entities.push(entity);
        }
        let target_entity = target_entity.or_else(|| {
          actor_is_known
            .then(|| {
              scene_actor
                .filter(|record| record.id() == actor)
                .map(|_| entity)
            })
            .flatten()
        });
        (marked_entities, target_entity)
      },
    )
  };
  for entity in marked_entities {
    if Some(entity) != target_entity {
      world.entity_mut(entity).remove::<SceneFocus>();
    }
  }
  if let Some(entity) = target_entity {
    world.entity_mut(entity).insert(SceneFocus);
  }
}

fn sync_camera(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let center = snapshot
    .actors()
    .iter()
    .find(|record| record.id() == actor)
    .map(Actor::position);
  let Some(mut camera) = world.get_resource_mut::<PresentationCamera>() else {
    return;
  };
  camera.actor = actor;
  camera.center = center;
}

fn sync_scene_camera(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none()
    || world.get_resource::<PresentationRuntime>().is_none()
  {
    return;
  }
  let Some(camera) = world.get_resource::<PresentationCamera>() else {
    return;
  };
  let center = camera.center();
  let Some(center) = center else {
    let mut query = world.query::<(Entity, &SceneCamera)>();
    let stale_entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    for entity in stale_entities {
      let _ = world.despawn(entity);
    }
    world.insert_resource(SceneCameraState::default());
    return;
  };
  let mut query = world.query::<(Entity, &SceneCamera)>();
  let mut existing = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  existing.sort_unstable();
  let retained = world
    .get_resource::<SceneCameraState>()
    .and_then(|state| state.entity)
    .filter(|entity| existing.contains(entity))
    .or_else(|| existing.first().copied());
  if let Some(entity) = retained {
    world.entity_mut(entity).insert(SceneCamera::new(center));
    for stale in existing
      .into_iter()
      .filter(|candidate| *candidate != entity)
    {
      let _ = world.despawn(stale);
    }
    world.insert_resource(SceneCameraState {
      entity: Some(entity),
    });
  } else {
    let entity = world.spawn(SceneCamera::new(center)).id();
    world.insert_resource(SceneCameraState {
      entity: Some(entity),
    });
  }
}

fn sync_viewport(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none() {
    return;
  }
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let Some(camera) = world.get_resource::<PresentationCamera>() else {
    return;
  };
  let Some(center) = camera.center() else {
    let Some(mut viewport) = world.get_resource_mut::<PresentationViewport>() else {
      return;
    };
    viewport.origin = None;
    viewport.effective_width = 0;
    viewport.effective_height = 0;
    return;
  };
  let Some((requested_width, requested_height)) = world
    .get_resource::<PresentationViewport>()
    .map(|viewport| (viewport.width(), viewport.height()))
  else {
    return;
  };
  let effective_width = requested_width.min(snapshot.width());
  let effective_height = requested_height.min(snapshot.height());
  let origin = Position::new(
    clamped_viewport_axis(center.x(), effective_width, snapshot.width()),
    clamped_viewport_axis(center.y(), effective_height, snapshot.height()),
  );
  let Some(mut viewport) = world.get_resource_mut::<PresentationViewport>() else {
    return;
  };
  viewport.origin = Some(origin);
  viewport.effective_width = effective_width;
  viewport.effective_height = effective_height;
}

fn clamped_viewport_axis(center: i32, extent: u32, map_extent: u32) -> i32 {
  let half_extent = i64::from(extent / 2);
  let desired = i64::from(center) - half_extent;
  let maximum = i64::from(map_extent.saturating_sub(extent));
  i32::try_from(desired.clamp(0, maximum)).unwrap_or_default()
}

fn sync_scene_viewport(world: &mut World) {
  if world.get_resource::<PresentationInput>().is_none()
    || world.get_resource::<PresentationRuntime>().is_none()
  {
    return;
  }
  let Some(viewport) = world.get_resource::<PresentationViewport>() else {
    return;
  };
  let Some(origin) = viewport.origin() else {
    let mut query = world.query::<(Entity, &SceneViewport)>();
    let stale_entities = query
      .iter(world)
      .map(|(entity, _)| entity)
      .collect::<Vec<_>>();
    for entity in stale_entities {
      let _ = world.despawn(entity);
    }
    world.insert_resource(SceneViewportState::default());
    return;
  };
  let dimensions = (viewport.effective_width(), viewport.effective_height());
  let mut query = world.query::<(Entity, &SceneViewport)>();
  let mut existing = query
    .iter(world)
    .map(|(entity, _)| entity)
    .collect::<Vec<_>>();
  existing.sort_unstable();
  let retained = world
    .get_resource::<SceneViewportState>()
    .and_then(|state| state.entity)
    .filter(|entity| existing.contains(entity))
    .or_else(|| existing.first().copied());
  let scene_viewport = SceneViewport::new(origin, dimensions.0, dimensions.1);
  if let Some(entity) = retained {
    world.entity_mut(entity).insert(scene_viewport);
    for stale in existing
      .into_iter()
      .filter(|candidate| *candidate != entity)
    {
      let _ = world.despawn(stale);
    }
    world.insert_resource(SceneViewportState {
      entity: Some(entity),
    });
  } else {
    let entity = world.spawn(scene_viewport).id();
    world.insert_resource(SceneViewportState {
      entity: Some(entity),
    });
  }
}

fn sync_hud(world: &mut World) {
  let Some(actor) = world
    .get_resource::<PresentationInput>()
    .map(|input| input.actor())
  else {
    return;
  };
  let Some(snapshot) = world
    .get_resource::<PresentationRuntime>()
    .map(PresentationRuntime::snapshot)
  else {
    return;
  };
  let Some(mut hud) = world.get_resource_mut::<PresentationHud>() else {
    return;
  };
  hud.actor = actor;
  if let Some(record) = snapshot.actors().iter().find(|record| record.id() == actor) {
    hud.kind = Some(record.kind());
    hud.position = Some(record.position());
    hud.hit_points = Some(record.hit_points());
    hud.ready_at = Some(record.ready_at());
  } else {
    hud.kind = None;
    hud.position = None;
    hud.hit_points = None;
    hud.ready_at = None;
  }
}

fn sync_messages(world: &mut World) {
  let Some(projected) = world.get_resource::<PresentationRuntime>().map(|runtime| {
    runtime
      .output()
      .map(|output| {
        output
          .events()
          .iter()
          .copied()
          .map(PresentationMessage::from_event)
          .collect()
      })
      .unwrap_or_default()
  }) else {
    return;
  };
  let Some(mut messages) = world.get_resource_mut::<PresentationMessages>() else {
    return;
  };
  messages.messages = projected;
}

fn sync_audio_cues(world: &mut World) {
  let Some(projected) = world.get_resource::<PresentationRuntime>().map(|runtime| {
    runtime
      .output()
      .map(|output| {
        output
          .events()
          .iter()
          .copied()
          .map(PresentationAudioCue::from_event)
          .collect()
      })
      .unwrap_or_default()
  }) else {
    return;
  };
  let Some(mut cues) = world.get_resource_mut::<PresentationAudioCues>() else {
    return;
  };
  cues.cues = projected;
}

fn sync_audio_asset_projection(world: &mut World) {
  if world.get_resource::<PresentationRuntime>().is_none()
    || world.get_resource::<PresentationAudioCues>().is_none()
    || world
      .get_resource::<PresentationAudioAssetManifest>()
      .is_none()
  {
    return;
  }
  let cues = world.resource::<PresentationAudioCues>().cues().to_vec();
  let manifest = world.resource::<PresentationAudioAssetManifest>();
  let projection = PresentationAudioAssetProjection::from_cues(&cues, manifest);
  let Some(mut destination) = world.get_resource_mut::<PresentationAudioAssetProjection>() else {
    return;
  };
  destination.entries = projection.entries;
}

fn sync_animation_cues(world: &mut World) {
  let Some(projected) = world.get_resource::<PresentationRuntime>().map(|runtime| {
    runtime
      .output()
      .map(|output| {
        output
          .events()
          .iter()
          .copied()
          .map(PresentationAnimationCue::from_event)
          .collect()
      })
      .unwrap_or_default()
  }) else {
    return;
  };
  let Some(mut cues) = world.get_resource_mut::<PresentationAnimationCues>() else {
    return;
  };
  cues.cues = projected;
}

fn tile_key(position: dreadstep_core::Position) -> (i32, i32) {
  (position.x(), position.y())
}

fn scene_position(index: usize, width: usize) -> Option<dreadstep_core::Position> {
  if width == 0 {
    return None;
  }
  Some(dreadstep_core::Position::new(
    i32::try_from(index % width).ok()?,
    i32::try_from(index / width).ok()?,
  ))
}

/// Synchronizes a complete core projection into disposable Bevy scene entities.
///
/// Tile entities are keyed by position, actor entities by [`ActorId`], and ground or inventory-item
/// entities by globally unique [`ItemId`]. Existing entities keep their Bevy identity when their key
/// remains in the snapshot; stale entities are despawned before new keys are spawned. The ECS world
/// is only a presentation mirror and cannot change core state.
pub fn sync_scene(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_tiles: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneTile)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, tile)| {
        entities
          .entry(tile_key(tile.position()))
          .or_default()
          .push(entity);
        entities
      })
  };
  for entities in existing_tiles.values_mut() {
    entities.sort_unstable();
  }
  let Ok(width) = usize::try_from(snapshot.width()) else {
    return;
  };
  let Some(positions) = snapshot
    .tiles()
    .iter()
    .enumerate()
    .map(|(index, _)| scene_position(index, width))
    .collect::<Option<Vec<_>>>()
  else {
    return;
  };
  let expected_tiles: BTreeSet<_> = positions.iter().copied().map(tile_key).collect();
  for (key, entities) in &existing_tiles {
    if expected_tiles.contains(key) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for (position, terrain) in positions
    .iter()
    .copied()
    .zip(snapshot.tiles().iter().copied())
  {
    if let Some(entity) = existing_tiles
      .get(&tile_key(position))
      .and_then(|entities| entities.first())
    {
      scene
        .entity_mut(*entity)
        .insert((SceneTile::new(position, terrain), SceneSpriteRole::Terrain));
    } else {
      scene.spawn((SceneTile::new(position, terrain), SceneSpriteRole::Terrain));
    }
  }

  let mut existing_actors: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneActor)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, actor)| {
        entities.entry(actor.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_actors.values_mut() {
    entities.sort_unstable();
  }
  let expected_actors: BTreeSet<_> = snapshot.actors().iter().map(Actor::id).collect();
  for (actor_id, entities) in &existing_actors {
    if expected_actors.contains(actor_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for actor in snapshot.actors() {
    let scene_actor = SceneActor::from_core(actor);
    let sprite_role = SceneSpriteRole::for_actor(actor);
    if let Some(entity) = existing_actors
      .get(&actor.id())
      .and_then(|entities| entities.first())
    {
      scene.entity_mut(*entity).insert((scene_actor, sprite_role));
    } else {
      scene.spawn((scene_actor, sprite_role));
    }
  }

  sync_ground_items(scene, snapshot);
  sync_inventory_items(scene, snapshot);
}

fn sync_ground_items(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_ground_items: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneGroundItem)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, item)| {
        entities.entry(item.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_ground_items.values_mut() {
    entities.sort_unstable();
  }
  let expected_ground_items: BTreeSet<_> = snapshot
    .ground_items()
    .iter()
    .flat_map(|stack| stack.items().iter())
    .map(|item| item.id())
    .collect();
  for (item_id, entities) in &existing_ground_items {
    if expected_ground_items.contains(item_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for stack in snapshot.ground_items() {
    for (stack_index, item) in stack.items().iter().enumerate() {
      let scene_item = SceneGroundItem::from_core(stack.position(), stack_index, *item);
      if let Some(entity) = existing_ground_items
        .get(&item.id())
        .and_then(|entities| entities.first())
      {
        scene
          .entity_mut(*entity)
          .insert((scene_item, SceneSpriteRole::GroundItem));
      } else {
        scene.spawn((scene_item, SceneSpriteRole::GroundItem));
      }
    }
  }
}

fn sync_inventory_items(scene: &mut World, snapshot: &PresentationSnapshot) {
  let mut existing_inventory_items: BTreeMap<_, Vec<_>> = {
    let mut query = scene.query::<(Entity, &SceneInventoryItem)>();
    query
      .iter(scene)
      .fold(BTreeMap::new(), |mut entities, (entity, item)| {
        entities.entry(item.id()).or_default().push(entity);
        entities
      })
  };
  for entities in existing_inventory_items.values_mut() {
    entities.sort_unstable();
  }
  let expected_inventory_items: BTreeSet<_> = snapshot
    .actors()
    .iter()
    .flat_map(|actor| actor.inventory().iter())
    .map(|item| item.id())
    .collect();
  for (item_id, entities) in &existing_inventory_items {
    if expected_inventory_items.contains(item_id) {
      for entity in entities.iter().skip(1) {
        let _ = scene.despawn(*entity);
      }
    } else {
      for entity in entities {
        let _ = scene.despawn(*entity);
      }
    }
  }
  for actor in snapshot.actors() {
    for (inventory_index, item) in actor.inventory().iter().enumerate() {
      let scene_item = SceneInventoryItem::from_core(actor.id(), inventory_index, *item);
      if let Some(entity) = existing_inventory_items
        .get(&item.id())
        .and_then(|entities| entities.first())
      {
        scene
          .entity_mut(*entity)
          .insert((scene_item, SceneSpriteRole::InventoryItem));
      } else {
        scene.spawn((scene_item, SceneSpriteRole::InventoryItem));
      }
    }
  }
}
