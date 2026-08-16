//! Contract tests for player-facing item equipment through the MCP session.

use dreadstep_mcp::{Session, SessionError};
use dreadstep_protocol::{
  ActorId, CommandError, CommandRequest, Event, ItemDefinitionId, ItemId, Position, WorldError,
};

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

#[test]
fn tester_cannot_move_equipped_item_and_preserves_session_evidence() {
  let mut session = Session::start_run(7).expect("fixed scenario should be valid");
  session
    .give_item(ActorId::new(1), ItemId::new(4), ItemDefinitionId::new(104))
    .expect("item should be accepted");
  session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
    .expect("item should equip");
  let before = session.observe();
  let history = session.history();
  let replay = session.get_replay();

  let drop_error = session
    .drop_item(ActorId::new(1), ItemId::new(4))
    .expect_err("equipped drop should be rejected");
  assert_eq!(
    drop_error,
    SessionError::WorldRejected(WorldError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.history(), history);
  assert_eq!(session.get_replay(), replay);

  let transfer_error = session
    .transfer_item(ActorId::new(1), ActorId::new(2), ItemId::new(4))
    .expect_err("equipped transfer should be rejected");
  assert_eq!(
    transfer_error,
    SessionError::WorldRejected(WorldError::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(4),
    })
  );
  assert_eq!(session.observe(), before);
  assert_eq!(session.history(), history);
  assert_eq!(session.get_replay(), replay);
}

#[test]
fn authored_reach_weapon_changes_melee_range_and_rejects_consumption() {
  let mut session = Session::start_item_run(7).expect("authored item scenario should be valid");
  let weapon = session
    .inspect(ActorId::new(1))
    .expect("player should exist")
    .inventory()
    .iter()
    .find(|item| item.id() == ItemId::new(103))
    .copied()
    .expect("authored reach weapon should be present");
  assert!(matches!(
    weapon.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::MinimumMeleeReach { reach }) if reach.value() == 2
  ));
  assert_eq!(
    session.act(CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    }),
    Err(SessionError::CommandRejected(
      CommandError::ItemNotConsumable {
        actor: ActorId::new(1),
        item: ItemId::new(103),
      }
    ))
  );
  session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(103),
    })
    .expect("reach weapon should equip");
  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect("enemy should yield");
  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("equipped reach should enable distance-two melee");
  assert!(matches!(output.events(), [Event::Attacked { .. }]));
}

#[test]
fn authored_damage_weapon_changes_attack_evidence_after_tester_transfer() {
  let mut session = Session::start_item_run(7).expect("authored item scenario should be valid");
  session
    .drop_item(ActorId::new(1), ItemId::new(101))
    .expect("tester should free one inventory slot");
  session
    .transfer_item(ActorId::new(2), ActorId::new(1), ItemId::new(100))
    .expect("authored damage weapon should transfer to the player");
  let weapon = session
    .inspect(ActorId::new(1))
    .expect("player should exist")
    .inventory()
    .iter()
    .find(|item| item.id() == ItemId::new(100))
    .copied()
    .expect("damage weapon should be visible after transfer");
  assert!(matches!(
    weapon.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::MeleeDamage { amount }) if amount.value() == 1
  ));
  session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(100),
    })
    .expect("damage weapon should equip");
  session
    .teleport(ActorId::new(2), Position::new(1, 0))
    .expect("tester should place the target adjacent for a melee check");
  session
    .act(CommandRequest::Wait {
      actor: ActorId::new(2),
    })
    .expect("enemy should yield");
  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("adjacent target should be attackable");
  assert!(matches!(
    output.events().first(),
    Some(Event::Attacked { damage, .. }) if damage.value() == 2
  ));
}

#[test]
fn authored_armor_reduces_attack_evidence_after_tester_transfer() {
  let mut session = Session::start_item_run(7).expect("authored item scenario should be valid");
  session
    .drop_item(ActorId::new(1), ItemId::new(101))
    .expect("tester should free one inventory slot");
  session
    .transfer_item(ActorId::new(2), ActorId::new(1), ItemId::new(105))
    .expect("authored armor should transfer to the player");
  let armor = session
    .inspect(ActorId::new(1))
    .expect("player should exist")
    .inventory()
    .iter()
    .find(|item| item.id() == ItemId::new(105))
    .copied()
    .expect("armor should be visible after transfer");
  assert!(matches!(
    armor.equipment_effect(),
    Some(dreadstep_protocol::EquipmentEffect::DamageReduction { amount }) if amount.value() == 1
  ));
  session
    .act(CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(105),
    })
    .expect("armor should equip");
  session
    .teleport(ActorId::new(2), Position::new(1, 0))
    .expect("tester should place the attacker adjacent");
  let output = session
    .act(CommandRequest::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("adjacent player should be attackable");
  assert!(matches!(
    output.events().first(),
    Some(Event::Attacked { damage, .. }) if damage.value() == 0
  ));
}
