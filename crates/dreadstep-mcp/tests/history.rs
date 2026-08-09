//! Contract tests for session history and replay digest evidence.

use dreadstep_core::ReplayTrace;
use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, StateDigest};

#[test]
fn new_session_has_empty_protocol_history_and_seeded_replay_digest() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");

  assert!(session.history().is_empty());
  assert_eq!(session.get_history(), session.history());
  assert_eq!(
    session.replay_digest(),
    StateDigest::new(ReplayTrace::new(7).digest().value())
  );
}

#[test]
fn accepted_actions_record_once_but_rejected_actions_do_not() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let initial_digest = session.replay_digest();
  let request = CommandRequest::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  };
  session
    .act(request)
    .expect("adjacent attack should succeed");

  assert_eq!(session.history(), vec![request]);
  assert_eq!(session.get_history(), session.history());
  assert_ne!(session.replay_digest(), initial_digest);

  let mut fresh = Session::start_run(7).expect("fixed scenario should be valid");
  let fresh_digest = fresh.replay_digest();
  assert_eq!(
    fresh.act(CommandRequest::Wait {
      actor: ActorId::new(2),
    }),
    Err(SessionError::CommandRejected(
      CommandError::ActorNotScheduled {
        requested: ActorId::new(2),
        scheduled: ActorId::new(1),
      }
    ))
  );
  assert!(fresh.history().is_empty());
  assert!(fresh.get_history().is_empty());
  assert_eq!(fresh.replay_digest(), fresh_digest);
}

#[test]
fn equivalent_accepted_sequences_have_equal_history_and_replay_digest() {
  let requests = [
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
    CommandRequest::Wait {
      actor: ActorId::new(2),
    },
  ];
  let mut first = Session::start_run(7).expect("fixed scenario should be valid");
  let mut second = Session::start_run(7).expect("fixed scenario should be valid");
  for request in requests {
    first.act(request).expect("request should be accepted");
    second.act(request).expect("request should be accepted");
  }

  assert_eq!(first.history(), second.history());
  assert_eq!(first.get_history(), second.get_history());
  assert_eq!(first.replay_digest(), second.replay_digest());
  let different_seed = Session::start_run(8).expect("fixed scenario should be valid");
  assert_ne!(first.replay_digest(), different_seed.replay_digest());

  let mut first_order = Session::start_run(7).expect("fixed scenario should be valid");
  for request in [
    CommandRequest::Wait {
      actor: ActorId::new(1),
    },
    CommandRequest::Wait {
      actor: ActorId::new(2),
    },
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
  ] {
    first_order
      .act(request)
      .expect("first ordered sequence should be accepted");
  }

  let mut second_order = Session::start_run(7).expect("fixed scenario should be valid");
  for request in [
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
    CommandRequest::Wait {
      actor: ActorId::new(2),
    },
    CommandRequest::Wait {
      actor: ActorId::new(1),
    },
  ] {
    second_order
      .act(request)
      .expect("second ordered sequence should be accepted");
  }
  assert_eq!(
    first_order.history(),
    vec![
      CommandRequest::Wait {
        actor: ActorId::new(1),
      },
      CommandRequest::Wait {
        actor: ActorId::new(2),
      },
      CommandRequest::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      },
    ]
  );
  assert_eq!(
    second_order.history(),
    vec![
      CommandRequest::Attack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      },
      CommandRequest::Wait {
        actor: ActorId::new(2),
      },
      CommandRequest::Wait {
        actor: ActorId::new(1),
      },
    ]
  );
  assert_ne!(first_order.replay_digest(), second_order.replay_digest());
}
