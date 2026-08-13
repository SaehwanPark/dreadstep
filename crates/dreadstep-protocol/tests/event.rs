//! Contract tests for protocol event conversion.

use dreadstep_core::{
  ActionTime as CoreActionTime, ActorId as CoreActorId, AmmunitionResult as CoreAmmunitionResult,
  BlockReason as CoreBlockReason, Damage as CoreDamage, Event as CoreEvent,
  HealingResult as CoreHealingResult, HitPoints as CoreHitPoints, ItemId as CoreItemId,
  Position as CorePosition,
};
use dreadstep_protocol::{
  ActionTime, ActorId, BlockReason, Damage, Event, HitPoints, ItemId, Position,
};

#[test]
#[expect(
  clippy::too_many_lines,
  reason = "the exhaustive conversion fixture documents every semantic event"
)]
fn maps_every_core_event_variant_to_protocol_values() {
  let events = [
    CoreEvent::Moved {
      actor: CoreActorId::new(1),
      from: CorePosition::new(0, 0),
      to: CorePosition::new(1, 0),
    },
    CoreEvent::MovementBlocked {
      actor: CoreActorId::new(1),
      from: CorePosition::new(0, 0),
      to: CorePosition::new(1, 0),
      reason: CoreBlockReason::Terrain,
    },
    CoreEvent::MovementBlocked {
      actor: CoreActorId::new(1),
      from: CorePosition::new(0, 0),
      to: CorePosition::new(1, 0),
      reason: CoreBlockReason::Actor(CoreActorId::new(2)),
    },
    CoreEvent::Waited {
      actor: CoreActorId::new(1),
      at: CoreActionTime::new(3),
    },
    CoreEvent::DoorOpened {
      actor: CoreActorId::new(1),
      position: CorePosition::new(1, 0),
    },
    CoreEvent::NoiseCreated {
      actor: CoreActorId::new(1),
      position: CorePosition::new(1, 0),
      radius: 3,
    },
    CoreEvent::BreakableBroken {
      actor: CoreActorId::new(1),
      position: CorePosition::new(1, 0),
    },
    CoreEvent::TrapTriggered {
      actor: CoreActorId::new(1),
      position: CorePosition::new(2, 0),
      damage: CoreDamage::new(1),
      remaining_hit_points: CoreHitPoints::new(4),
    },
    CoreEvent::Attacked {
      attacker: CoreActorId::new(1),
      target: CoreActorId::new(2),
      damage: CoreDamage::new(1),
      remaining_hit_points: CoreHitPoints::new(1),
    },
    CoreEvent::ItemThrown {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(8),
      target: CoreActorId::new(2),
    },
    CoreEvent::Died {
      actor: CoreActorId::new(2),
    },
    CoreEvent::ItemUnequipped {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(4),
    },
    CoreEvent::ItemEquipped {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(5),
    },
    CoreEvent::ItemConsumed {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(6),
      healing: None,
      ammunition: None,
    },
    CoreEvent::ItemPickedUp {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(7),
    },
  ];

  let projected: Vec<Event> = events.into_iter().map(Event::from).collect();
  assert_eq!(
    projected,
    vec![
      Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      },
      Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Terrain,
      },
      Event::MovementBlocked {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
        reason: BlockReason::Actor(ActorId::new(2)),
      },
      Event::Waited {
        actor: ActorId::new(1),
        at: ActionTime::new(3),
      },
      Event::DoorOpened {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
      Event::NoiseCreated {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        radius: 3,
      },
      Event::BreakableBroken {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
      },
      Event::TrapTriggered {
        actor: ActorId::new(1),
        position: Position::new(2, 0),
        damage: Damage::new(1),
        remaining_hit_points: HitPoints::new(4),
      },
      Event::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::new(1),
        remaining_hit_points: HitPoints::new(1),
      },
      Event::ItemThrown {
        actor: ActorId::new(1),
        item: ItemId::new(8),
        target: ActorId::new(2),
      },
      Event::Died {
        actor: ActorId::new(2),
      },
      Event::ItemUnequipped {
        actor: ActorId::new(1),
        item: ItemId::new(4),
      },
      Event::ItemEquipped {
        actor: ActorId::new(1),
        item: ItemId::new(5),
      },
      Event::ItemConsumed {
        actor: ActorId::new(1),
        item: ItemId::new(6),
        healing: None,
        ammunition: None,
      },
      Event::ItemPickedUp {
        actor: ActorId::new(1),
        item: ItemId::new(7),
      },
    ]
  );
}

#[test]
fn maps_optional_healing_evidence_and_json_shape() {
  let event = Event::from(CoreEvent::ItemConsumed {
    actor: CoreActorId::new(1),
    item: CoreItemId::new(6),
    healing: Some(CoreHealingResult::new(2, CoreHitPoints::new(10))),
    ammunition: None,
  });

  let Event::ItemConsumed { healing, .. } = event else {
    panic!("item consumption should preserve its event variant");
  };
  let healing = healing.expect("healing evidence should cross the protocol boundary");
  assert_eq!(healing.amount(), 2);
  assert_eq!(healing.remaining_hit_points(), HitPoints::new(10));
  assert_eq!(
    serde_json::to_value(event).expect("healing event should serialize"),
    serde_json::json!({
      "item_consumed": {
        "actor": 1,
        "item": 6,
        "healing": {"amount": 2, "remaining_hit_points": 10},
        "ammunition": null
      }
    })
  );
}

#[test]
fn maps_optional_ammunition_evidence_and_json_shape() {
  let event = Event::from(CoreEvent::ItemConsumed {
    actor: CoreActorId::new(1),
    item: CoreItemId::new(7),
    healing: None,
    ammunition: Some(CoreAmmunitionResult::new(2, 3)),
  });

  let Event::ItemConsumed { ammunition, .. } = event else {
    panic!("item consumption should preserve its event variant");
  };
  let ammunition = ammunition.expect("ammunition evidence should cross the protocol boundary");
  assert_eq!(ammunition.amount(), 2);
  assert_eq!(ammunition.remaining_ammunition(), 3);
  assert_eq!(
    serde_json::to_value(event).expect("ammunition event should serialize"),
    serde_json::json!({
      "item_consumed": {
        "actor": 1,
        "item": 7,
        "healing": null,
        "ammunition": {"amount": 2, "remaining_ammunition": 3}
      }
    })
  );
}
