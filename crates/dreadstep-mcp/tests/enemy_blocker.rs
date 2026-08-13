//! MCP scenario projection for the stationary Blocker archetype.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, EnemyBehavior, Event, HitPoints, Position, Scenario,
  ScenarioActor, Tile,
};

#[test]
fn scenario_preserves_blocker_and_projects_its_stationary_wait() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor; 3],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(5),
        ),
        ScenarioActor::with_melee_reach_and_behavior(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(2, 0),
          HitPoints::new(5),
          dreadstep_protocol::MeleeReach::DEFAULT,
          EnemyBehavior::Blocker,
        ),
      ],
    ))
    .expect("Blocker scenario should validate");

  assert_eq!(
    session
      .inspect(ActorId::new(2))
      .expect("Blocker should exist")
      .behavior(),
    EnemyBehavior::Blocker
  );

  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield the schedule");
  let wait = CommandRequest::Wait {
    actor: ActorId::new(2),
  };
  assert!(session.legal_actions().contains(&wait));
  let output = session.act(wait).expect("Blocker wait should succeed");
  assert_eq!(
    output.events(),
    &[Event::Waited {
      actor: ActorId::new(2),
      at: dreadstep_protocol::ActionTime::new(0),
    }]
  );
  assert_eq!(session.history().last(), Some(&wait));
}
