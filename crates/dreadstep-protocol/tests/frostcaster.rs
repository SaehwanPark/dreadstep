//! Versioned protocol contracts for the Frostcaster archetype.

use dreadstep_core::{
  ActorId as CoreActorId, Command as CoreCommand, CommandError as CoreCommandError,
  EnemyBehavior as CoreEnemyBehavior, Event as CoreEvent,
};
use dreadstep_protocol::{ActorId, CommandError, CommandRequest, EnemyBehavior, Event};
use serde_json::json;

#[test]
fn frostcaster_behavior_has_a_stable_snake_case_projection() {
  assert_eq!(
    EnemyBehavior::from(CoreEnemyBehavior::Frostcaster),
    EnemyBehavior::Frostcaster
  );
  assert_eq!(
    CoreEnemyBehavior::from(EnemyBehavior::Frostcaster),
    CoreEnemyBehavior::Frostcaster
  );
  assert_eq!(
    serde_json::to_value(EnemyBehavior::Frostcaster).expect("behavior should serialize"),
    json!("frostcaster")
  );
}

#[test]
fn cast_chill_request_round_trips_through_core_and_json() {
  let request = CommandRequest::CastChill {
    actor: ActorId::new(2),
    target: ActorId::new(1),
  };
  let core = CoreCommand::CastChill {
    actor: CoreActorId::new(2),
    target: CoreActorId::new(1),
  };

  assert_eq!(CoreCommand::from(request), core);
  assert_eq!(CommandRequest::from(core), request);
  let value = serde_json::to_value(request).expect("request should serialize");
  assert_eq!(value, json!({"cast_chill": {"actor": 2, "target": 1}}));
  assert_eq!(
    serde_json::from_value::<CommandRequest>(value).expect("request should deserialize"),
    request
  );
}

#[test]
fn chill_cast_event_has_a_stable_typed_projection() {
  let event = Event::from(CoreEvent::ChillCast {
    caster: CoreActorId::new(2),
    target: CoreActorId::new(1),
  });

  assert_eq!(
    event,
    Event::ChillCast {
      caster: ActorId::new(2),
      target: ActorId::new(1),
    }
  );
  assert_eq!(
    serde_json::to_value(event).expect("event should serialize"),
    json!({"chill_cast": {"caster": 2, "target": 1}})
  );
}

#[test]
fn every_cast_chill_error_maps_without_losing_actor_identity() {
  let cases = [
    (
      CoreCommandError::CastChillRequiresFrostcaster(CoreActorId::new(2)),
      CommandError::CastChillRequiresFrostcaster(ActorId::new(2)),
    ),
    (
      CoreCommandError::CannotCastChillSelf(CoreActorId::new(2)),
      CommandError::CannotCastChillSelf(ActorId::new(2)),
    ),
    (
      CoreCommandError::CastChillUnknownTarget(CoreActorId::new(9)),
      CommandError::CastChillUnknownTarget(ActorId::new(9)),
    ),
    (
      CoreCommandError::CastChillTargetDead(CoreActorId::new(1)),
      CommandError::CastChillTargetDead(ActorId::new(1)),
    ),
    (
      CoreCommandError::CastChillOutOfRange {
        caster: CoreActorId::new(2),
        target: CoreActorId::new(1),
      },
      CommandError::CastChillOutOfRange {
        caster: ActorId::new(2),
        target: ActorId::new(1),
      },
    ),
    (
      CoreCommandError::CastChillNoLineOfSight {
        caster: CoreActorId::new(2),
        target: CoreActorId::new(1),
      },
      CommandError::CastChillNoLineOfSight {
        caster: ActorId::new(2),
        target: ActorId::new(1),
      },
    ),
  ];

  for (core, expected) in cases {
    assert_eq!(CommandError::from(core), expected);
  }
  let schema = serde_json::to_string(&schemars::schema_for!(CommandError))
    .expect("command error schema should serialize");
  for variant in [
    "cast_chill_requires_frostcaster",
    "cannot_cast_chill_self",
    "cast_chill_unknown_target",
    "cast_chill_target_dead",
    "cast_chill_out_of_range",
    "cast_chill_no_line_of_sight",
  ] {
    assert!(schema.contains(variant), "schema should include {variant}");
  }
}
