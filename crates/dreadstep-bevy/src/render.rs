//! Read-only sprite, placeholder, and render-plan projections.

use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use bevy::sprite::Sprite;
use dreadstep_core::{Actor, ActorKind, ItemDefinitionId, Tile};

use crate::{SceneActor, SceneGroundItem, SceneInventoryItem, SceneTile};

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
  pub(crate) fn for_actor(actor: &Actor) -> Self {
    if !actor.is_alive() {
      return Self::DeadActor;
    }
    match actor.kind() {
      ActorKind::Player => Self::Player,
      ActorKind::Enemy => Self::Enemy,
    }
  }

  pub(crate) fn for_scene_actor(actor: SceneActor) -> Self {
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
  pub(crate) render_entry: SceneRenderEntry,
  pub(crate) key: SceneSpriteKey,
}

impl SceneSpriteEntry {
  pub(crate) fn from_render_entry(render_entry: SceneRenderEntry) -> Self {
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
  pub(crate) entries: Vec<SceneSpriteEntry>,
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
  pub(crate) fn from_key(key: SceneSpriteKey) -> Self {
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
  pub(crate) sprite: SceneSpriteEntry,
  pub(crate) layer: SceneRenderLayer,
  pub(crate) order: usize,
}

impl SceneRenderCommand {
  pub(crate) fn from_sprite_entry(sprite: SceneSpriteEntry, order: usize) -> Self {
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
  pub(crate) commands: Vec<SceneRenderCommand>,
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
  pub(crate) path: String,
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

  /// Returns whether this reference is rooted in a root, `assets/audio/`, or crate-local `audio/`
  /// directory.
  #[must_use]
  pub fn is_audio_path(&self) -> bool {
    self.path.starts_with("audio/")
      || self.path.starts_with("assets/audio/")
      || matches!(crate_local_media_directory(&self.path), Some("audio"))
  }
}

/// A deterministic complete mapping from typed placeholder families to local asset references.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct PresentationAssetManifest {
  pub(crate) bindings: Vec<(SceneRenderPlaceholder, PresentationAssetReference)>,
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

pub(crate) fn crate_local_media_directory(path: &str) -> Option<&str> {
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

pub(crate) fn is_crate_local_media_path(path: &str) -> bool {
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
  pub(crate) fn from_key(key: SceneSpriteKey) -> Self {
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
  pub(crate) source_entity: Entity,
  pub(crate) key: SceneSpriteKey,
  pub(crate) layer: SceneRenderLayer,
  pub(crate) order: usize,
  pub(crate) pixel_position: Option<ScenePixelPosition>,
  pub(crate) placeholder: SceneRenderPlaceholder,
  pub(crate) visible: bool,
}

impl SceneRenderNode {
  pub(crate) fn from_command(command: SceneRenderCommand, visible: bool) -> Self {
    let sprite = command.sprite_entry();
    Self {
      source_entity: sprite.entity(),
      key: sprite.key(),
      layer: command.layer(),
      order: command.order(),
      pixel_position: command.pixel_position(),
      placeholder: SceneRenderPlaceholder::from_key(sprite.key()),
      visible,
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

  /// Returns whether this node is inside the optional presentation field of view.
  #[must_use]
  pub const fn is_visible(self) -> bool {
    self.visible
  }
}

/// One read-only projection entry for a reconciled placeholder render node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneRenderNodeEntry {
  pub(crate) node_entity: Entity,
  pub(crate) node: SceneRenderNode,
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
  pub(crate) entries: Vec<SceneRenderNodeEntry>,
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
  pub(crate) node: SceneRenderNodeEntry,
  pub(crate) sprite: Sprite,
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
  pub(crate) entries: Vec<SceneBevySpriteEntry>,
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

/// One optional map-space translation joined to a stable render-node entry.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneBevySpriteTransformEntry {
  pub(crate) node: SceneRenderNodeEntry,
  pub(crate) translation: Option<ScenePixelPosition>,
}

impl SceneBevySpriteTransformEntry {
  /// Returns the stable render-node metadata retained by this translation.
  #[must_use]
  pub const fn node(self) -> SceneRenderNodeEntry {
    self.node
  }

  /// Returns the map-space translation, or `None` for unplaced entries.
  #[must_use]
  pub const fn translation(self) -> Option<ScenePixelPosition> {
    self.translation
  }
}

/// An ordered, read-only headless transform projection for placeholder Sprite values.
#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct PresentationBevySpriteTransformProjection {
  pub(crate) entries: Vec<SceneBevySpriteTransformEntry>,
}

impl PresentationBevySpriteTransformProjection {
  /// Creates an empty headless Sprite-transform projection.
  #[must_use]
  pub const fn new() -> Self {
    Self {
      entries: Vec::new(),
    }
  }

  /// Returns transform values in deterministic render-node order.
  #[must_use]
  pub fn entries(&self) -> &[SceneBevySpriteTransformEntry] {
    &self.entries
  }
}

/// One render-node entry joined with its validated local-only asset reference.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneRenderAssetEntry {
  pub(crate) node: SceneRenderNodeEntry,
  pub(crate) reference: PresentationAssetReference,
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
  pub(crate) entries: Vec<SceneRenderAssetEntry>,
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
  pub(crate) entries: Vec<SceneRenderEntry>,
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
  pub(crate) x: u32,
  pub(crate) y: u32,
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
