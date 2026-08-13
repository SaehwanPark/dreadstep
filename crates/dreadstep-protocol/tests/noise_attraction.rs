//! Protocol evidence for kick-noise investigation.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command as CoreCommand, GridMap, Position, Tile, WorldState,
};
use dreadstep_protocol::{
  ActorId as ProtocolActorId, CommandRequest, Position as ProtocolPosition, WorldSnapshot,
};

#[test]
fn investigate_request_round_trips_and_snapshot_projects_hearing() {
  let request = CommandRequest::Investigate {
    actor: ProtocolActorId::new(2),
    position: ProtocolPosition::new(2, 1),
  };
  let core = dreadstep_core::Command::from(request);
  assert_eq!(
    core,
    CoreCommand::Investigate {
      actor: ActorId::new(2),
      position: Position::new(2, 1),
    }
  );
  assert_eq!(CommandRequest::from(core), request);

  let mut world = WorldState::new(
    GridMap::from_tiles(
      4,
      1,
      vec![Tile::Floor, Tile::Door, Tile::Floor, Tile::Floor],
    )
    .expect("protocol map validates"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(3, 0)),
    ],
  )
  .expect("protocol world validates");
  world
    .execute(CoreCommand::Kick {
      actor: ActorId::new(1),
      position: Position::new(1, 0),
    })
    .expect("kick should arm hearing");
  let snapshot = WorldSnapshot::from_world(&world);
  assert_eq!(
    snapshot
      .actors()
      .iter()
      .find(|actor| actor.id() == ProtocolActorId::new(2))
      .expect("enemy snapshot exists")
      .heard_noise(),
    Some(ProtocolPosition::new(1, 0))
  );
}
