//! Contract tests for protocol-owned agent action requests.

use dreadstep_core::{
  ActorId as CoreActorId, Command, Direction as CoreDirection, ItemId as CoreItemId,
};
use dreadstep_protocol::{ActorId, CommandRequest, Direction, ItemId};

#[test]
fn every_request_variant_round_trips_through_core() {
  let requests = [
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::North,
    },
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::South,
    },
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::West,
    },
    CommandRequest::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    },
    CommandRequest::Wait {
      actor: ActorId::new(2),
    },
    CommandRequest::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    },
    CommandRequest::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    },
    CommandRequest::Equip {
      actor: ActorId::new(1),
      item: ItemId::new(9),
    },
    CommandRequest::Unequip {
      actor: ActorId::new(1),
    },
    CommandRequest::UseItem {
      actor: ActorId::new(1),
      item: ItemId::new(9),
    },
  ];

  for request in requests {
    let core_command = Command::from(request);
    assert_eq!(CommandRequest::from(core_command), request);
  }
}

#[test]
fn every_core_command_round_trips_through_protocol() {
  let commands = [
    Command::Move {
      actor: CoreActorId::new(1),
      direction: CoreDirection::North,
    },
    Command::Move {
      actor: CoreActorId::new(1),
      direction: CoreDirection::South,
    },
    Command::Move {
      actor: CoreActorId::new(1),
      direction: CoreDirection::West,
    },
    Command::Move {
      actor: CoreActorId::new(1),
      direction: CoreDirection::East,
    },
    Command::Wait {
      actor: CoreActorId::new(2),
    },
    Command::Attack {
      actor: CoreActorId::new(1),
      target: CoreActorId::new(2),
    },
    Command::Chase {
      actor: CoreActorId::new(2),
      target: CoreActorId::new(1),
    },
    Command::Equip {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(9),
    },
    Command::Unequip {
      actor: CoreActorId::new(1),
    },
    Command::UseItem {
      actor: CoreActorId::new(1),
      item: CoreItemId::new(9),
    },
  ];

  for command in commands {
    let request = CommandRequest::from(command);
    assert_eq!(Command::from(request), command);
  }
}
