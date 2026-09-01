//! Contract tests for MCP tester scenario replacement.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, BlockReason, CommandRequest, Direction, Event, HitPoints, MapError, Position,
  Scenario, ScenarioActor, ScenarioError, Tile, WorldError,
};

fn scenario() -> Scenario {
  Scenario::new(
    3,
    1,
    vec![Tile::Floor, Tile::Wall, Tile::Floor],
    vec![
      ScenarioActor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(4),
      ),
      ScenarioActor::new(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(3),
      ),
    ],
  )
}

#[test]
fn create_scenario_replaces_world_and_resets_trace_for_the_same_seed() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .act(dreadstep_protocol::CommandRequest::Wait {
      actor: ActorId::new(1),
    })
    .expect("initial action should be accepted");
  let previous_digest = session.inspect_world().digest();

  session
    .create_scenario(&scenario())
    .expect("valid scenario should replace the world");

  assert_eq!(session.seed(), 7);
  assert_ne!(session.inspect_world().digest(), previous_digest);
  assert_eq!(session.get_history(), Vec::new());
  assert_eq!(session.get_replay().seed(), 7);
  assert!(session.get_replay().commands().is_empty());
  let enemy = session
    .inspect(ActorId::new(2))
    .expect("new actor should be inspectable");
  assert_eq!(enemy.position(), Position::new(2, 0));
  assert_eq!(enemy.hit_points(), HitPoints::new(3));

  let blocked = session
    .act(CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("the submitted wall should block movement");
  assert_eq!(
    blocked.events(),
    &[Event::MovementBlocked {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
      reason: BlockReason::Terrain,
    }]
  );
}

#[test]
fn stairs_scenario_remains_walkable_through_mcp_session() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      2,
      1,
      vec![Tile::Floor, Tile::Stairs],
      vec![ScenarioActor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(4),
      )],
    ))
    .expect("stairs scenario should validate");

  let moved = session
    .act(CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("the player should be able to enter stairs");
  assert_eq!(
    moved.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
    }]
  );
  assert_eq!(
    session
      .inspect(ActorId::new(1))
      .expect("player should remain inspectable")
      .position(),
    Position::new(1, 0)
  );
}

#[test]
fn invalid_scenario_is_typed_and_atomic_for_map_and_world_errors() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .act(dreadstep_protocol::CommandRequest::Wait {
      actor: ActorId::new(1),
    })
    .expect("initial action should be accepted");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.create_scenario(&Scenario::new(3, 1, vec![Tile::Floor, Tile::Floor], vec![],)),
    Err(SessionError::Scenario(ScenarioError::Map(
      MapError::TileCountMismatch {
        expected: 3,
        actual: 2,
      },
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);

  assert_eq!(
    session.create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor, Tile::Floor, Tile::Floor],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(4),
        ),
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Enemy,
          Position::new(2, 0),
          HitPoints::new(3),
        ),
      ],
    )),
    Err(SessionError::Scenario(ScenarioError::World(
      WorldError::DuplicateActorId(ActorId::new(1)),
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
