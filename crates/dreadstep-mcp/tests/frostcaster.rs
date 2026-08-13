//! Frostcaster behavior and action projection through the bounded MCP session.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, EnemyBehavior, Event, HitPoints, MeleeReach, Position,
  Scenario, ScenarioActor, StatusKind, Tile,
};

#[test]
fn scenario_preserves_frostcaster_and_act_projects_its_chill_cast() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      4,
      1,
      vec![Tile::Floor; 4],
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
          Position::new(3, 0),
          HitPoints::new(5),
          MeleeReach::DEFAULT,
          EnemyBehavior::Frostcaster,
        ),
      ],
    ))
    .expect("Frostcaster scenario should validate");
  assert_eq!(
    session
      .inspect(ActorId::new(2))
      .expect("Frostcaster should exist")
      .behavior(),
    EnemyBehavior::Frostcaster
  );

  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield the schedule");
  let cast = CommandRequest::CastChill {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  };
  assert!(session.legal_actions().contains(&cast));
  let output = session.act(cast).expect("Frostcaster cast should succeed");
  assert_eq!(
    output.events(),
    &[
      Event::ChillCast {
        caster: ActorId::new(2),
        target: ActorId::new(1),
      },
      Event::StatusApplied {
        actor: ActorId::new(1),
        status: StatusKind::Chilled,
        remaining_actions: 2,
      },
    ]
  );
  assert_eq!(session.history().last(), Some(&cast));
  assert_eq!(session.get_replay().commands().last(), Some(&cast));
}
