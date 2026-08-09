//! Contract tests for MCP tester hit-point mutation.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, HitPoints, LifeState, WorldError};

#[test]
fn set_hp_updates_inspection_without_recording_a_player_command() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let history = session.get_history();
  let replay = session.get_replay();

  session
    .set_hp(ActorId::new(2), HitPoints::new(5))
    .expect("known actor should be updated");

  let actor = session
    .inspect(ActorId::new(2))
    .expect("updated actor should be inspectable");
  assert_eq!(actor.hit_points(), HitPoints::new(5));
  assert_eq!(actor.life(), LifeState::Alive);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn set_hp_rejection_is_typed_and_atomic() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let before = session.inspect_world();
  let history = session.get_history();
  let replay = session.get_replay();
  let unknown = ActorId::new(9);

  assert_eq!(
    session.set_hp(unknown, HitPoints::new(4)),
    Err(SessionError::WorldRejected(WorldError::UnknownActor(
      unknown
    )))
  );
  assert_eq!(session.inspect_world(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
