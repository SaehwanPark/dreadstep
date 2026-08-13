//! Contract tests for MCP kick-open-door and noise evidence.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, Event, HitPoints, Position, Scenario, ScenarioActor, Tile,
};

#[test]
fn tester_scenario_preserves_door_and_projects_kick_noise_evidence() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor, Tile::Door, Tile::Floor],
      vec![ScenarioActor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(5),
      )],
    ))
    .expect("door scenario should validate");

  let output = session
    .act(CommandRequest::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent closed door should be kickable");
  assert_eq!(
    output.events(),
    &[
      Event::DoorOpened {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
      Event::NoiseCreated {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        radius: 3,
      },
    ]
  );
  assert_eq!(session.get_history().len(), 1);
}

#[test]
fn player_kick_exposes_and_consumes_enemy_investigation() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      4,
      1,
      vec![Tile::Floor, Tile::Door, Tile::Floor, Tile::Floor],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(5),
        ),
        ScenarioActor::new(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(3, 0),
          HitPoints::new(5),
        ),
      ],
    ))
    .expect("noise scenario should validate");
  session
    .act(CommandRequest::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("kick should be accepted");
  let enemy = session
    .inspect(ActorId::new(2))
    .expect("enemy should remain inspectable");
  assert_eq!(enemy.heard_noise(), Some(Position::new(1, 0)));
  let output = session
    .act(CommandRequest::Investigate {
      actor: ActorId::new(2),
      position: Position::new(1, 0),
    })
    .expect("enemy should investigate the pending noise");
  assert_eq!(output.snapshot().actors()[1].heard_noise(), None);
  assert_eq!(
    output.snapshot().actors()[1].position(),
    Position::new(2, 0)
  );
}
