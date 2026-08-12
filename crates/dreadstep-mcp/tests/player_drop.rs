//! MCP session evidence for the scheduled player-facing item drop.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, ActorKind, CommandError, CommandRequest, Event, HitPoints, ItemDefinitionId, ItemId,
  Position, Scenario, ScenarioActor, Tile,
};

fn session_with_item() -> Session {
  let mut session = Session::start_run(7).expect("fixed session should start");
  session
    .create_scenario(&Scenario::new(
      3,
      1,
      vec![Tile::Floor, Tile::Floor, Tile::Floor],
      vec![
        ScenarioActor::new(
          ActorId::new(1),
          ActorKind::Player,
          Position::new(0, 0),
          HitPoints::new(10),
        ),
        ScenarioActor::new(
          ActorId::new(2),
          ActorKind::Enemy,
          Position::new(2, 0),
          HitPoints::new(10),
        ),
      ],
    ))
    .expect("scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(101), ItemDefinitionId::new(1))
    .expect("item should be owned");
  session
}

#[test]
fn drop_projects_event_snapshot_history_and_replay() {
  let mut session = session_with_item();
  let output = session
    .act(CommandRequest::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    })
    .expect("player drop should be accepted");

  assert_eq!(
    output.events(),
    &[Event::ItemDropped {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    }]
  );
  assert!(output.snapshot().actors()[0].inventory().is_empty());
  assert_eq!(
    output.snapshot().ground_items()[0].items()[0].id(),
    ItemId::new(101)
  );
  assert_eq!(
    session.history(),
    &[CommandRequest::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    }]
  );
  assert_eq!(session.get_replay().commands().len(), 1);
}

#[test]
fn equipped_drop_rejection_preserves_session_evidence() {
  let mut session = session_with_item();
  session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    })
    .expect("item should equip");
  session
    .act(CommandRequest::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("enemy should take its scheduled turn");
  let before = session.snapshot();
  let history = session.history().clone();
  let replay = session.get_replay();
  let error = session
    .act(CommandRequest::Drop {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    })
    .expect_err("equipped item should reject");
  assert_eq!(
    error,
    SessionError::CommandRejected(CommandError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    })
  );
  assert_eq!(session.snapshot(), before);
  assert_eq!(session.history(), history);
  assert_eq!(session.get_replay(), replay);
}
