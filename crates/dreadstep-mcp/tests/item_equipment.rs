//! Contract tests for player-facing item equipment through the MCP session.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, Event, ItemDefinitionId, ItemId};

#[test]
fn equipment_actions_project_events_snapshot_history_and_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("item should be accepted");

  let output = session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("owned item should equip");
  assert_eq!(
    output.events(),
    &[Event::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    output.snapshot().actors()[0].equipped_item(),
    Some(ItemId::new(4))
  );
  assert_eq!(
    session.history(),
    vec![CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  let replay = session.get_replay();

  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect("the other scheduled actor should wait");

  let unequipped = session
    .act(CommandRequest::Unequip {
      actor: ActorId::new(1),
    })
    .expect("equipped item should unequip");
  assert_eq!(
    unequipped.events(),
    &[Event::ItemUnequipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    }]
  );
  assert_eq!(
    session.inspect(ActorId::new(1)).unwrap().equipped_item(),
    None
  );
  assert_ne!(session.get_replay(), replay);
}

#[test]
fn rejected_equipment_preserves_snapshot_history_and_replay() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("item should be accepted");
  let before = session.observe();
  let history = session.history();
  let replay = session.get_replay();

  assert_eq!(
    session.act(CommandRequest::Equip {
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
}
