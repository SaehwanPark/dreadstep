//! Contract tests for protocol event conversion.

use dreadstep_core::{
  ActionTime as CoreActionTime, ActorId as CoreActorId, BlockReason as CoreBlockReason,
  Damage as CoreDamage, Event as CoreEvent, HitPoints as CoreHitPoints, Position as CorePosition,
};
use dreadstep_protocol::{ActionTime, ActorId, BlockReason, Damage, Event, HitPoints, Position};

#[test]
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
    CoreEvent::Attacked {
      attacker: CoreActorId::new(1),
      target: CoreActorId::new(2),
      damage: CoreDamage::new(1),
      remaining_hit_points: CoreHitPoints::new(1),
    },
    CoreEvent::Died {
      actor: CoreActorId::new(2),
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
      Event::Attacked {
        attacker: ActorId::new(1),
        target: ActorId::new(2),
        damage: Damage::new(1),
        remaining_hit_points: HitPoints::new(1),
      },
      Event::Died {
        actor: ActorId::new(2),
      },
    ]
  );
}
