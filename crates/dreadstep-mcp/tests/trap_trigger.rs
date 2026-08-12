//! Contract tests for the MCP one-shot trap boundary.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, Event, HitPoints, Position, Scenario, ScenarioActor, Tile,
};

#[test]
fn tester_scenario_preserves_trap_and_projects_trigger_evidence() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor, Tile::Trap, Tile::Floor],
      vec![ScenarioActor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(5),
      )],
    ))
    .expect("trap scenario should validate");

  let before = session.replay_digest();
  let output = session
    .act(CommandRequest::Move {
      actor: ActorId::new(1),
      direction: dreadstep_protocol::Direction::East,
    })
    .expect("entering a trap should be accepted");
  assert_eq!(
    output.events(),
    &[
      Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      },
      Event::TrapTriggered {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        damage: dreadstep_protocol::Damage::new(1),
        remaining_hit_points: HitPoints::new(4),
      },
    ]
  );
  assert_ne!(session.replay_digest(), before);
  assert_eq!(session.get_history().len(), 1);
}
