//! MCP observation evidence for canonical run outcomes.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, ActorKind, CommandRequest, HitPoints, Position, RunOutcome};

#[test]
fn accepted_actions_project_core_victory_without_changing_history_shape() {
  let mut session = Session::start_run(7).expect("fixed session should start");
  assert_eq!(session.observe().outcome(), RunOutcome::InProgress);

  session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("first attack should succeed");
  session
    .act(CommandRequest::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("enemy attack should succeed");
  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("last attack should succeed");

  assert_eq!(output.snapshot().outcome(), RunOutcome::Victory);
  assert_eq!(session.get_replay().commands().len(), 3);
}

#[test]
fn tester_hit_point_mutation_projects_player_defeat() {
  let mut session = Session::start_run(7).expect("fixed session should start");
  session
    .set_hp(ActorId::new(1), HitPoints::new(1))
    .expect("player hp mutation should succeed");
  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield the scheduled turn");
  session
    .act(CommandRequest::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("enemy attack should succeed");

  let player = session
    .inspect(ActorId::new(1))
    .expect("player snapshot should remain inspectable");
  assert_eq!(player.kind(), ActorKind::Player);
  assert_eq!(player.position(), Position::new(0, 0));
  assert_eq!(session.observe().outcome(), RunOutcome::Defeat);
}
