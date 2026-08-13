//! Disposable ECS mirrors of tiles, actors, items, camera, window, and viewport.

use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::ecs::resource::Resource;
use dreadstep_core::{Actor, ActorId, Item, ItemDefinitionId, ItemId, Position, Tile};

use crate::PresentationWindow;

/// A disposable ECS mirror of one projected map tile.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneTile {
  pub(crate) position: dreadstep_core::Position,
  pub(crate) terrain: Tile,
}

impl SceneTile {
  pub(crate) fn new(position: dreadstep_core::Position, terrain: Tile) -> Self {
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
  pub(crate) id: ActorId,
  pub(crate) kind: dreadstep_core::ActorKind,
  pub(crate) position: dreadstep_core::Position,
  pub(crate) hit_points: dreadstep_core::HitPoints,
  pub(crate) melee_reach: dreadstep_core::MeleeReach,
  pub(crate) ready_at: dreadstep_core::ActionTime,
  pub(crate) equipped_item: Option<ItemId>,
  pub(crate) alive: bool,
}

impl SceneActor {
  pub(crate) fn from_core(actor: &Actor) -> Self {
    Self {
      id: actor.id(),
      kind: actor.kind(),
      position: actor.position(),
      hit_points: actor.hit_points(),
      melee_reach: actor.melee_reach(),
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

  /// Returns the projected actor's non-zero Manhattan melee reach.
  #[must_use]
  pub const fn melee_reach(self) -> dreadstep_core::MeleeReach {
    self.melee_reach
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
  pub(crate) id: ItemId,
  pub(crate) definition: ItemDefinitionId,
  pub(crate) position: Position,
  pub(crate) stack_index: usize,
}

impl SceneGroundItem {
  pub(crate) fn from_core(position: Position, stack_index: usize, item: Item) -> Self {
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
  pub(crate) id: ItemId,
  pub(crate) owner: ActorId,
  pub(crate) definition: ItemDefinitionId,
  pub(crate) inventory_index: usize,
}

impl SceneInventoryItem {
  pub(crate) fn from_core(owner: ActorId, inventory_index: usize, item: Item) -> Self {
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
  pub(crate) center: Position,
}

impl SceneCamera {
  /// Creates a disposable camera projection for a known core center.
  #[must_use]
  pub const fn new(center: Position) -> Self {
    Self { center }
  }

  /// Returns the projected map position copied from the authoritative camera state.
  #[must_use]
  pub const fn center(self) -> Position {
    self.center
  }
}

#[derive(Default, Resource)]
pub(crate) struct SceneCameraState {
  pub(crate) entity: Option<Entity>,
}

/// A disposable ECS mirror of one validated window configuration request.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneWindow {
  pub(crate) logical_width: u32,
  pub(crate) logical_height: u32,
  pub(crate) pixel_scale: u32,
  pub(crate) physical_width: u32,
  pub(crate) physical_height: u32,
}

impl SceneWindow {
  /// Creates a disposable window projection from a validated request.
  #[must_use]
  pub const fn new(request: PresentationWindow) -> Self {
    Self {
      logical_width: request.logical_width(),
      logical_height: request.logical_height(),
      pixel_scale: request.pixel_scale(),
      physical_width: request.physical_width(),
      physical_height: request.physical_height(),
    }
  }

  /// Returns the requested logical width.
  #[must_use]
  pub const fn logical_width(self) -> u32 {
    self.logical_width
  }

  /// Returns the requested logical height.
  #[must_use]
  pub const fn logical_height(self) -> u32 {
    self.logical_height
  }

  /// Returns the requested integer pixel scale.
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

#[derive(Default, Resource)]
pub(crate) struct SceneWindowState {
  pub(crate) entity: Option<Entity>,
}

/// A disposable ECS mirror of one effective in-map viewport rectangle.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneViewport {
  pub(crate) origin: Position,
  pub(crate) width: u32,
  pub(crate) height: u32,
}

impl SceneViewport {
  pub(crate) const fn new(origin: Position, width: u32, height: u32) -> Self {
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
pub(crate) struct SceneViewportState {
  pub(crate) entity: Option<Entity>,
}
