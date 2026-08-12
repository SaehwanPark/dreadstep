//! Contract tests for player-facing item pickup through the MCP session.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, CommandError, CommandRequest, Event, HitPoints, ItemDefinitionId, ItemId,
  Position, Scenario, ScenarioActor, Tile,
};

#[test]
fn pickup_projects_event_snapshot_history_and_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("item should be accepted");
  session
    .drop_item(ActorId::new(1), ItemId::new(4))
    .expect("item should be dropped for the player action");

  assert!(session.legal_actions().contains(&CommandRequest::Pickup {
    actor: ActorId::new(1),
    item: ItemId::new(4),
  }));
  let output = session
    .act(CommandRequest::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("ground item should be picked up");
  assert_eq!(
    output.events(),
    &[Event::ItemPickedUp {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(output.snapshot().ground_items(), &[]);
  assert_eq!(
    output.snapshot().actors()[0].inventory()[0].id(),
    ItemId::new(4)
  );
  assert_eq!(
    session.history(),
    vec![CommandRequest::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(session.get_replay().commands(), session.history());
}

#[test]
fn rejected_pickup_preserves_session_evidence() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  let before = session.observe();
  let history = session.history();
  let replay = session.get_replay();

  assert_eq!(
    session.act(CommandRequest::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }),
    Err(SessionError::CommandRejected(
      CommandError::ItemNotOnGround {
        actor: ActorId::new(1),
        item: ItemId::new(99),
      }
    ))
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn enemy_pickup_is_hidden_and_rejected_without_session_mutation() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .create_scenario(&Scenario::new(
      2,
      1,
      vec![Tile::Floor, Tile::Floor],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Enemy,
          Position::new(0, 0),
          HitPoints::new(3),
        ),
        ScenarioActor::new(
          ActorId::new(2),
          ActorKind::Player,
          Position::new(1, 0),
          HitPoints::new(3),
        ),
      ],
    ))
    .expect("enemy-first scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("enemy item should be accepted");
  session
    .drop_item(ActorId::new(1), ItemId::new(4))
    .expect("enemy item should be dropped");
  assert!(!session.legal_actions().contains(&CommandRequest::Pickup {
    actor: ActorId::new(1),
    item: ItemId::new(4),
  }));
  let before = session.observe();
  let history = session.history();
  let replay = session.get_replay();
  assert_eq!(
    session.act(CommandRequest::Pickup {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }),
    Err(SessionError::CommandRejected(
      CommandError::PickupRequiresPlayer(ActorId::new(1),)
    ))
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.history(), history);
  assert_eq!(session.get_replay(), replay);
}
