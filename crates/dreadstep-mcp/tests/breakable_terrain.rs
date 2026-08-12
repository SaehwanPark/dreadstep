//! Contract tests for the MCP breakable-terrain boundary.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, Event, HitPoints, Position, Scenario, ScenarioActor, Tile,
};

#[test]
fn tester_scenario_preserves_breakable_terrain_and_projects_break_event() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor, Tile::Breakable, Tile::Floor],
      vec![ScenarioActor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(5),
      )],
    ))
    .expect("breakable scenario should validate");

  let output = session
    .act(CommandRequest::Break {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("adjacent breakable terrain should break");
  assert_eq!(
    output.events(),
    &[Event::BreakableBroken {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    }]
  );
  assert_eq!(session.get_history().len(), 1);
}
