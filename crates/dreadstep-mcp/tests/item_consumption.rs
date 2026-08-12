//! Contract tests for single-item consumption through the MCP player session.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, CommandError, CommandRequest, Event, HitPoints, ItemDefinitionId, ItemId,
};

#[test]
fn consumption_updates_snapshot_history_replay_and_typed_event() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("first item should be accepted");
  session
    .give_item(ActorId::new(1), ItemId::new(5), ItemDefinitionId::new(105))
    .expect("second item should be accepted");

  let before_replay = session.get_replay();
  let output = session
    .act(CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("owned unequipped item should be consumed");
  assert_eq!(
    output.events(),
    &[Event::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(4),
      healing: None,
    }]
  );
  let actor_snapshot = session
    .inspect(ActorId::new(1))
    .expect("actor should exist");
  let inventory = actor_snapshot.inventory();
  assert_eq!(inventory.len(), 1);
  assert_eq!(inventory[0].id(), ItemId::new(5));
  assert_eq!(inventory[0].definition(), ItemDefinitionId::new(105));
  assert_eq!(
    session.history(),
    vec![CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_ne!(session.get_replay(), before_replay);
}

#[test]
fn consumption_rejects_unknown_and_equipped_items_without_session_mutation() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("item should be accepted");
  let before = session.observe();
  let history = session.history();
  let replay = session.get_replay();
  assert_eq!(
    session.act(CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }),
    Err(SessionError::CommandRejected(CommandError::ItemNotOwned {
      actor: ActorId::new(1),
      item: ItemId::new(99),
    }))
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.history(), history);
  assert_eq!(session.get_replay(), replay);

  session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("item should equip");
  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect("other actor should wait");
  let equipped_before = session.observe();
  let equipped_history = session.history();
  let equipped_replay = session.get_replay();
  assert_eq!(
    session.act(CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }),
    Err(SessionError::CommandRejected(CommandError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }))
  );
  assert_eq!(session.observe(), equipped_before);
  assert_eq!(session.history(), equipped_history);
  assert_eq!(session.get_replay(), equipped_replay);
}

#[test]
fn authored_healing_item_reports_capped_recovery_through_player_output() {
  let mut session = Session::start_item_run(7).expect("authored item scenario should be valid");
  session
    .set_hp(ActorId::new(1), HitPoints::new(8))
    .expect("tester damage should be accepted");

  let output = session
    .act(CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(101),
    })
    .expect("authored healing item should be consumable");

  let [Event::ItemConsumed { healing, .. }] = output.events() else {
    panic!("healing use should emit one item-consumption event");
  };
  let healing = healing.expect("authored item should report healing evidence");
  assert_eq!(healing.amount(), 2);
  assert_eq!(healing.remaining_hit_points(), HitPoints::new(10));
  assert_eq!(
    output.snapshot().actors()[0].hit_points(),
    HitPoints::new(10)
  );
}
