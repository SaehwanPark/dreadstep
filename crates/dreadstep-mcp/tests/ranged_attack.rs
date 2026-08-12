//! MCP player-facing ranged attack contract.

use dreadstep_core::ReplayTrace;
use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, CommandError, CommandRequest, Damage, Event, HitPoints, Position,
  ReplayEvidence, Scenario, ScenarioActor, StateDigest, Tile,
};

fn scenario() -> Scenario {
  Scenario::new(
    5,
    1,
    vec![Tile::Floor; 5],
    vec![
      ScenarioActor::new(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(4),
      ),
      ScenarioActor::new(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(2),
      ),
    ],
  )
}

#[test]
fn legal_actions_and_act_expose_ranged_attack_with_replay_evidence() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&scenario())
    .expect("scenario should validate");

  assert!(
    session
      .legal_actions()
      .contains(&CommandRequest::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
  );
  let output = session
    .act(CommandRequest::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("ranged action should be accepted");
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
    output
      .snapshot()
      .actors()
      .iter()
      .find(|actor| actor.id() == ActorId::new(1))
      .expect("attacker remains visible")
      .ready_at()
      .value(),
    2
  );
  assert_eq!(
    session.history(),
    vec![CommandRequest::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }]
  );
  assert_ne!(
    session.get_replay(),
    ReplayEvidence::new(
      7,
      Vec::new(),
      StateDigest::new(ReplayTrace::new(7).digest().value())
    )
  );

  let before = session.observe();
  let replay = session.get_replay();
  assert_eq!(
    session.act(CommandRequest::RangedAttack {
      actor: ActorId::new(2),
      target: ActorId::new(2),
    }),
    Err(SessionError::CommandRejected(
      CommandError::CannotAttackSelf(ActorId::new(2))
    ))
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn blocked_ranged_attack_is_hidden_and_rejected_without_session_mutation() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      4,
      1,
      vec![Tile::Floor, Tile::Wall, Tile::Floor, Tile::Floor],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(4),
        ),
        ScenarioActor::new(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(2, 0),
          HitPoints::new(2),
        ),
      ],
    ))
    .expect("scenario should validate");

  assert!(
    !session
      .legal_actions()
      .contains(&CommandRequest::RangedAttack {
        actor: ActorId::new(1),
        target: ActorId::new(2),
      })
  );
  let before = session.observe();
  let replay = session.get_replay();
  assert_eq!(
    session.act(CommandRequest::RangedAttack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(SessionError::CommandRejected(
      CommandError::RangedAttackNoLineOfSight {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
      }
    ))
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.get_replay(), replay);
  assert!(session.history().is_empty());
}
