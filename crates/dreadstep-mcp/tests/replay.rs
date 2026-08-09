//! Contract tests for the typed in-memory replay evidence projection.

use dreadstep_core::ReplayTrace;
use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, CommandRequest, ReplayEvidence, StateDigest};

#[test]
fn new_session_exposes_seeded_empty_replay_evidence() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let replay = session.get_replay();

  assert_eq!(
    replay,
    ReplayEvidence::new(
      7,
      Vec::new(),
      StateDigest::new(ReplayTrace::new(7).digest().value())
    )
  );
  assert_eq!(replay.seed(), 7);
  assert!(replay.commands().is_empty());
}

#[test]
fn replay_evidence_keeps_accepted_order_and_ignores_rejection() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let attack = CommandRequest::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  };
  let wait = CommandRequest::Wait {
    actor: ActorId::new(2),
  };
  session
    .act(attack)
    .expect("initial attack should be accepted");
  session.act(wait).expect("enemy wait should be accepted");
  let accepted = session.get_replay();
  assert_eq!(accepted.commands(), &[attack, wait]);

  let rejected = session.act(CommandRequest::Wait {
    actor: ActorId::new(2),
  });
  assert!(matches!(rejected, Err(SessionError::CommandRejected(_))));
  assert_eq!(session.get_replay(), accepted);
}

#[test]
fn equivalent_replay_evidence_matches_but_seed_and_order_change_digest() {
  let attack = CommandRequest::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  };
  let wait_player = CommandRequest::Wait {
    actor: ActorId::new(1),
  };
  let wait_enemy = CommandRequest::Wait {
    actor: ActorId::new(2),
  };

  let mut first = Session::start_run(7).expect("fixed scenario should be valid");
  first
    .act(wait_player)
    .expect("initial player wait should be accepted");
  first
    .act(wait_enemy)
    .expect("enemy wait should be accepted");
  first.act(attack).expect("player attack should be accepted");

  let mut equivalent = Session::start_run(7).expect("fixed scenario should be valid");
  equivalent
    .act(wait_player)
    .expect("initial player wait should be accepted");
  equivalent
    .act(wait_enemy)
    .expect("enemy wait should be accepted");
  equivalent
    .act(attack)
    .expect("player attack should be accepted");

  let mut different_seed = Session::start_run(8).expect("fixed scenario should be valid");
  different_seed
    .act(wait_player)
    .expect("initial player wait should be accepted");
  different_seed
    .act(wait_enemy)
    .expect("enemy wait should be accepted");
  different_seed
    .act(attack)
    .expect("player attack should be accepted");

  let mut different_order = Session::start_run(7).expect("fixed scenario should be valid");
  different_order
    .act(attack)
    .expect("initial attack should be accepted");
  different_order
    .act(wait_enemy)
    .expect("enemy wait should be accepted");
  different_order
    .act(wait_player)
    .expect("player wait should be accepted");

  assert_eq!(first.get_replay(), equivalent.get_replay());
  assert_ne!(
    first.get_replay().digest(),
    different_seed.get_replay().digest()
  );
  assert_eq!(
    first.get_replay().commands(),
    &[wait_player, wait_enemy, attack]
  );
  assert_eq!(
    different_order.get_replay().commands(),
    &[attack, wait_enemy, wait_player]
  );
  assert_ne!(
    first.get_replay().digest(),
    different_order.get_replay().digest()
  );
}
