//! MCP evidence for the typed actor melee-reach preparation.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, CommandError, CommandRequest, Damage, Event, HitPoints, MeleeReach, Position,
  Scenario, ScenarioActor, Tile,
};

fn scenario(reach: MeleeReach) -> Scenario {
  Scenario::new(
    3,
    1,
    vec![Tile::Floor; 3],
    vec![
      ScenarioActor::with_melee_reach(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(4),
        reach,
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
fn explicit_melee_reach_is_observable_and_accepts_the_extended_attack() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let reach = MeleeReach::new(2).expect("two is a valid reach");
  session
    .create_scenario(&scenario(reach))
    .expect("scenario should validate");

  let inspected = session
    .inspect(ActorId::new(1))
    .expect("player should remain inspectable");
  assert_eq!(inspected.melee_reach(), reach);
  assert!(session.legal_actions().contains(&CommandRequest::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));

  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("explicit reach should permit the two-tile attack");
  assert_eq!(
    output.events(),
    &[Event::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
      damage: Damage::new(1),
      remaining_hit_points: HitPoints::new(1),
    }]
  );
}

#[test]
fn default_melee_reach_rejects_extended_attack_without_mutating_evidence() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&scenario(MeleeReach::DEFAULT))
    .expect("scenario should validate");
  assert!(!session.legal_actions().contains(&CommandRequest::Attack {
    actor: ActorId::new(1),
    target: ActorId::new(2),
  }));
  let before = session.observe();
  let history = session.get_history();
  let replay = session.get_replay();

  assert_eq!(
    session.act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    }),
    Err(SessionError::CommandRejected(
      CommandError::AttackOutOfRange {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
      }
    ))
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.get_history(), history);
  assert_eq!(session.get_replay(), replay);
}
