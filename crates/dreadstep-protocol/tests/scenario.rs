//! Contract tests for protocol-owned tester scenario values and error projection.

use dreadstep_core::{
  ActorId as CoreActorId, MapError as CoreMapError, Position as CorePosition,
  WorldError as CoreWorldError,
};
use dreadstep_protocol::{
  ActorId, MapError, Position, Scenario, ScenarioActor, ScenarioError, Tile, WorldError,
};

#[test]
fn scenario_values_preserve_typed_map_and_actor_inputs() {
  let scenario = Scenario::new(
    3,
    1,
    vec![Tile::Floor, Tile::Cover, Tile::Wall],
    vec![ScenarioActor::new(
      ActorId::new(1),
      dreadstep_protocol::ActorKind::Player,
      Position::new(0, 0),
      dreadstep_protocol::HitPoints::new(4),
    )],
  );

  assert_eq!(scenario.width(), 3);
  assert_eq!(scenario.height(), 1);
  assert_eq!(scenario.tiles(), &[Tile::Floor, Tile::Cover, Tile::Wall]);
  assert_eq!(scenario.actors().len(), 1);
  assert_eq!(scenario.actors()[0].id(), ActorId::new(1));
  assert_eq!(
    scenario.actors()[0].melee_reach(),
    dreadstep_protocol::MeleeReach::DEFAULT
  );
}

#[test]
fn scenario_values_preserve_explicit_melee_reach() {
  let reach = dreadstep_protocol::MeleeReach::new(2).expect("two is a valid reach");
  let scenario = Scenario::new(
    3,
    1,
    vec![Tile::Floor, Tile::Floor, Tile::Floor],
    vec![ScenarioActor::with_melee_reach(
      ActorId::new(1),
      dreadstep_protocol::ActorKind::Player,
      Position::new(0, 0),
      dreadstep_protocol::HitPoints::new(4),
      reach,
    )],
  );

  assert_eq!(scenario.actors()[0].melee_reach(), reach);
}

#[test]
fn scenario_errors_project_core_map_and_world_failures() {
  let map_errors = [
    (CoreMapError::ZeroWidth, MapError::ZeroWidth),
    (CoreMapError::ZeroHeight, MapError::ZeroHeight),
    (
      CoreMapError::TooLarge {
        width: 3,
        height: 4,
      },
      MapError::TooLarge {
        width: 3,
        height: 4,
      },
    ),
    (
      CoreMapError::CoordinateRange {
        width: 3,
        height: 4,
      },
      MapError::CoordinateRange {
        width: 3,
        height: 4,
      },
    ),
    (
      CoreMapError::TileCountMismatch {
        expected: 3,
        actual: 2,
      },
      MapError::TileCountMismatch {
        expected: 3,
        actual: 2,
      },
    ),
  ];
  for (core, protocol) in map_errors {
    assert_eq!(MapError::from(core.clone()), protocol);
    assert_eq!(ScenarioError::from(core), ScenarioError::Map(protocol));
  }
  assert_eq!(
    ScenarioError::from(CoreWorldError::DuplicateActorId(CoreActorId::new(1))),
    ScenarioError::World(WorldError::DuplicateActorId(ActorId::new(1)))
  );
  assert_eq!(
    ScenarioError::from(CoreWorldError::ActorOutOfBounds {
      actor: CoreActorId::new(2),
      position: CorePosition::new(3, 0),
    }),
    ScenarioError::World(WorldError::ActorOutOfBounds {
      actor: ActorId::new(2),
      position: Position::new(3, 0),
    })
  );
}
