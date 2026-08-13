//! MCP scenario projection for the Brute archetype.

use dreadstep_mcp::Session;
use dreadstep_protocol::{
  ActorId, ActorKind, EnemyBehavior, HitPoints, Position, Scenario, ScenarioActor, Tile,
};

#[test]
fn scenario_preserves_brute_behavior() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor, Tile::Breakable, Tile::Floor],
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
          EnemyBehavior::Brute,
        ),
      ],
    ))
    .expect("brute scenario should validate");
  assert_eq!(
    session
      .inspect(ActorId::new(2))
      .expect("brute exists")
      .behavior(),
    EnemyBehavior::Brute
  );
}
