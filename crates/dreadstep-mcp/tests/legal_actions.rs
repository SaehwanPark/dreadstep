//! Contract tests for MCP legal-action conversion.

use dreadstep_mcp::Session;
use dreadstep_protocol::{ActorId, CommandRequest, Damage, Direction, Event, HitPoints};

#[test]
fn session_exposes_protocol_actions_for_the_scheduled_player() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let actions = session.legal_actions();

  assert_eq!(actions.len(), 7);
  assert_eq!(
    actions[0],
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::North,
    }
  );
  assert_eq!(
    actions[6],
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }
  );
}

#[test]
fn equivalent_sessions_expose_equal_legal_action_lists() {
  let first = Session::start_run(7).expect("fixed scenario should be valid");
  let second = Session::start_run(7).expect("fixed scenario should be valid");

  assert_eq!(first.legal_actions(), second.legal_actions());
}

#[test]
fn legal_action_discovery_is_read_only_for_world_history_and_replay() {
  let session = Session::start_run(7).expect("fixed scenario should be valid");
  let before_snapshot = session.observe();
  let before_history = session.history();
  let before_replay = session.get_replay();

  let actions = session.legal_actions();

  assert_eq!(actions.len(), 7);
  assert_eq!(session.observe(), before_snapshot);
  assert_eq!(session.history(), before_history);
  assert_eq!(session.get_replay(), before_replay);
}

#[test]
fn session_exposes_adjacent_enemy_attack_through_existing_protocol_contract() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&dreadstep_protocol::Scenario::new(
      2,
      1,
      vec![dreadstep_protocol::Tile::Floor; 2],
      vec![
        dreadstep_protocol::ScenarioActor::new(
          ActorId::new(1),
          dreadstep_protocol::ActorKind::Enemy,
          dreadstep_protocol::Position::new(0, 0),
          dreadstep_protocol::HitPoints::new(4),
        ),
        dreadstep_protocol::ScenarioActor::new(
          ActorId::new(2),
          dreadstep_protocol::ActorKind::Player,
          dreadstep_protocol::Position::new(1, 0),
          dreadstep_protocol::HitPoints::new(4),
        ),
      ],
    ))
    .expect("adjacent enemy scenario should validate");

  assert_eq!(
    session.legal_actions().last(),
    Some(&CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
  );
  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent enemy attack should use the existing action contract");
  assert_eq!(
    output.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::new(1),
      remaining_hit_points: HitPoints::new(3),
    }]
  );
  assert_eq!(
    output
      .snapshot()
      .actors()
      .iter()
      .find(|actor| actor.id() == ActorId::new(2))
      .expect("target remains visible")
      .hit_points(),
    HitPoints::new(3)
  );
  assert_eq!(
    session.history(),
    vec![CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }]
  );
}
