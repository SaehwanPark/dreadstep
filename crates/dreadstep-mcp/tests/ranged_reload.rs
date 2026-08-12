//! MCP session evidence for the deterministic ranged reload action.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event};

#[test]
fn reload_is_accepted_after_a_ranged_shot_and_enters_history() {
  let mut session = Session::start_run(7).expect("fixed session should start");
  session
    .create_scenario(&dreadstep_protocol::Scenario::new(
      3,
      1,
      vec![
        dreadstep_protocol::Tile::Floor,
        dreadstep_protocol::Tile::Floor,
        dreadstep_protocol::Tile::Floor,
      ],
      vec![
        dreadstep_protocol::ScenarioActor::new(
          ActorId::new(1),
          dreadstep_protocol::ActorKind::Player,
          dreadstep_protocol::Position::new(0, 0),
          dreadstep_protocol::HitPoints::new(10),
        ),
        dreadstep_protocol::ScenarioActor::new(
          ActorId::new(2),
          dreadstep_protocol::ActorKind::Enemy,
          dreadstep_protocol::Position::new(2, 0),
          dreadstep_protocol::HitPoints::new(10),
        ),
      ],
    ))
    .expect("scenario should be valid");
  session
    .act(CommandRequest::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("ranged shot should be accepted");
  session
    .act(CommandRequest::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("enemy should take its scheduled turn");
  session
    .act(CommandRequest::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("enemy should finish its second scheduled turn");
  let output = session
    .act(CommandRequest::Reload {
      actor: ActorId::new(1),
    })
    .expect("partial ammunition should reload");

  assert_eq!(
    output.events(),
    &[Event::Reloaded {
      actor: ActorId::new(1),
      ammunition: 3,
    }]
  );
  assert!(session.get_history().contains(&CommandRequest::Reload {
    actor: ActorId::new(1),
  }));
  assert_eq!(
    output
      .snapshot()
      .actors()
      .iter()
      .find(|actor| actor.id() == ActorId::new(1))
      .expect("player snapshot")
      .ranged_ammo(),
    3
  );
}

#[test]
fn full_ammo_reload_is_projected_as_a_typed_rejection() {
  let mut session = Session::start_run(7).expect("fixed session should start");
  session
    .create_scenario(&dreadstep_protocol::Scenario::new(
      3,
      1,
      vec![
        dreadstep_protocol::Tile::Floor,
        dreadstep_protocol::Tile::Floor,
        dreadstep_protocol::Tile::Floor,
      ],
      vec![
        dreadstep_protocol::ScenarioActor::new(
          ActorId::new(1),
          dreadstep_protocol::ActorKind::Player,
          dreadstep_protocol::Position::new(0, 0),
          dreadstep_protocol::HitPoints::new(10),
        ),
        dreadstep_protocol::ScenarioActor::new(
          ActorId::new(2),
          dreadstep_protocol::ActorKind::Enemy,
          dreadstep_protocol::Position::new(2, 0),
          dreadstep_protocol::HitPoints::new(10),
        ),
      ],
    ))
    .expect("scenario should be valid");
  let before = session.get_replay();
  let error = session
    .act(CommandRequest::Reload {
      actor: ActorId::new(1),
    })
    .expect_err("full ammo should reject");
  assert_eq!(
    error,
    SessionError::CommandRejected(CommandError::ReloadNotNeeded(ActorId::new(1)))
  );
  assert_eq!(session.get_replay(), before);
}
