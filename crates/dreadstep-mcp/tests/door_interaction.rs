//! Contract tests for the MCP door interaction boundary.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, Event, HitPoints, Position, Scenario, ScenarioActor, Tile,
};

#[test]
fn tester_scenario_preserves_door_and_projects_opening_event() {
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

  assert!(session.legal_actions().contains(&CommandRequest::Interact {
    actor: ActorId::new(1),
    position: Position::new(1, 0),
  }));
  let before = session.replay_digest();
  let output = session
    .act(CommandRequest::Interact {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent door should open");
  assert_eq!(
    output.events(),
    &[Event::DoorOpened {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }]
  );
  assert_ne!(session.replay_digest(), before);
  assert_eq!(session.get_history().len(), 1);
}
