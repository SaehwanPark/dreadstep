//! Contract tests for the in-memory MCP player session.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, CommandError, CommandRequest, Damage, Event, HitPoints, LifeState, Position,
};

#[test]
fn start_run_and_observe_expose_the_fixed_seeded_world() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");

  assert_eq!(session.seed(), 7);
  let snapshot = session.observe();
  assert_eq!(snapshot.next_actor(), Some(ActorId::new(1)));
  assert_eq!(snapshot.actors().len(), 2);
  assert_eq!(snapshot.actors()[0].kind(), ActorKind::Player);
  assert_eq!(snapshot.actors()[0].position(), Position::new(0, 0));
  assert_eq!(snapshot.actors()[0].life(), LifeState::Alive);
  assert_eq!(snapshot.actors()[1].hit_points(), HitPoints::new(2));
}

#[test]
fn act_delegates_to_core_and_returns_protocol_event_and_snapshot() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent attack should succeed");

  assert_eq!(output.seed(), 7);
  assert_eq!(
    output.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::new(1),
      remaining_hit_points: HitPoints::new(1),
    }]
  );
  assert_eq!(
    output.snapshot().actors()[1].hit_points(),
    HitPoints::new(1)
  );
}

#[test]
fn rejected_unscheduled_action_returns_without_mutating_session() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let before = session.observe();
  let error = session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect_err("actor two should not act before actor one");

  assert_eq!(
    error,
    SessionError::CommandRejected(CommandError::ActorNotScheduled {
      requested: ActorId::new(2),
      scheduled: ActorId::new(1),
    })
  );
  assert_eq!(session.observe(), before);
}

#[test]
fn identical_seed_and_action_sequences_produce_equal_outputs_and_snapshots() {
  let mut first = Session::start_run(7).expect("fixed scenario should be valid");
  let mut second = Session::start_run(7).expect("fixed scenario should be valid");
  let requests = [
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
    CommandRequest::Wait {
      actor: ActorId::new(2),
    },
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
  ];

  for request in requests {
    assert_eq!(first.act(request), second.act(request));
  }
  assert_eq!(first.observe(), second.observe());
}
