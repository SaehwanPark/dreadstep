//! Contract tests for authored kiter retreat through the MCP boundary.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, CommandRequest, EnemyBehavior, Event, HitPoints, Position, Scenario,
  ScenarioActor, Tile,
};

#[test]
fn scenario_projects_kiter_and_legal_retreat_before_attack() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      4,
      3,
      vec![Tile::Floor; 12],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(2, 1),
          HitPoints::new(5),
        ),
        ScenarioActor::with_melee_reach_and_behavior(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(1, 1),
          HitPoints::new(5),
          dreadstep_protocol::MeleeReach::DEFAULT,
          EnemyBehavior::Kiter,
        ),
      ],
    ))
    .expect("kiter scenario should validate");
  assert_eq!(
    session
      .inspect(ActorId::new(2))
      .expect("kiter exists")
      .behavior(),
    EnemyBehavior::Kiter
  );

  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to the kiter");
  let legal = session.legal_actions();
  let retreat = CommandRequest::Retreat {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  };
  assert_eq!(
    legal.first(),
    Some(&CommandRequest::Move {
      actor: ActorId::new(2),
      direction: dreadstep_protocol::Direction::North,
    })
  );
  assert!(legal.contains(&retreat));
  assert!(
    legal
      .iter()
      .position(|command| command == &retreat)
      .expect("retreat should be legal")
      < legal
        .iter()
        .position(|command| matches!(command, CommandRequest::Attack { .. }))
        .expect("attack fallback should remain legal")
  );

  let output = session.act(retreat).expect("kiter should retreat");
  assert_eq!(
    output.events(),
    &[Event::Moved {
      actor: ActorId::new(2),
      from: Position::new(1, 1),
      to: Position::new(1, 0),
    }]
  );
  assert_eq!(
    session.history(),
    vec![
      CommandRequest::Wait {
        actor: ActorId::new(1)
      },
      retreat,
    ]
  );
}
