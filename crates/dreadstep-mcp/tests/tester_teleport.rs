//! Contract tests for MCP tester teleport.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, HitPoints, Position, Scenario, ScenarioActor, Tile, WorldError,
};

fn session() -> Session {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      4,
      2,
      vec![
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Floor,
        Tile::Floor,
      ],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        ScenarioActor::new(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(2, 0),
          HitPoints::new(10),
        ),
      ],
    ))
    .expect("scenario should be valid");
  session
}

#[test]
fn teleport_updates_inspection_without_recording_player_replay() {
  let mut session = session();
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();

  session
    .teleport(ActorId::new(1), Position::new(3, 1))
    .expect("walkable destination should be accepted");

  let actor = session
    .inspect(ActorId::new(1))
    .expect("actor should be inspectable");
  assert_eq!(actor.position(), Position::new(3, 1));
  assert_ne!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn teleport_rejection_is_typed_and_atomic() {
  let mut session = session();
  let before = session.inspect_world();

  assert_eq!(
    session.teleport(ActorId::new(1), Position::new(4, 0)),
    Err(SessionError::WorldRejected(
      WorldError::TeleportOutOfBounds {
        actor: ActorId::new(1),
        position: Position::new(4, 0),
      }
    ))
  );
  assert_eq!(session.inspect_world(), before);

  assert_eq!(
    session.teleport(ActorId::new(1), Position::new(2, 0)),
    Err(SessionError::WorldRejected(WorldError::TeleportOccupied {
      actor: ActorId::new(1),
      blocker: ActorId::new(2),
      position: Position::new(2, 0),
    }))
  );
  assert_eq!(session.inspect_world(), before);
}
