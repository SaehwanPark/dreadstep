//! Protocol contract tests for chilled status projection.

use dreadstep_core::{Actor, ActorId, ActorKind, Command, GridMap, Position, Tile, WorldState};
use dreadstep_protocol::{Event, PROTOCOL_VERSION, StatusKind, WorldSnapshot};
use serde_json::json;

#[test]
fn chilled_status_event_and_snapshot_use_v24_wire_values() {
  let mut world = WorldState::new(
    GridMap::from_tiles(2, 1, vec![Tile::Floor, Tile::ChillTrap]).unwrap(),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .unwrap();
  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: dreadstep_core::Direction::East,
    })
    .unwrap();
  let event = Event::from(result.events()[1]);
  assert!(matches!(
    event,
    Event::StatusApplied {
      status: StatusKind::Chilled,
      remaining_actions: 2,
      ..
    }
  ));
  let snapshot = WorldSnapshot::from_world(&world);
  assert_eq!(snapshot.protocol_version(), PROTOCOL_VERSION);
  assert_eq!(
    snapshot.actors()[0].status().unwrap().remaining_actions(),
    2
  );
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  let expired = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .unwrap();
  let expiry = Event::from(expired.events()[1]);
  assert!(matches!(
    expiry,
    Event::StatusExpired {
      status: StatusKind::Chilled,
      ..
    }
  ));
  assert_eq!(
    serde_json::to_value(expiry).unwrap(),
    json!({"status_expired": {"actor": 1, "status": "chilled"}})
  );
}
