//! Deterministic presentation bridge behavior.

use bevy::input::keyboard::KeyCode;
use dreadstep_bevy::{KeyboardIntent, PresentationState};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, Event, GridMap, HitPoints, Position, ReplayTrace,
  Tile, WorldState,
};

fn world() -> WorldState {
  let map = GridMap::from_tiles(
    4,
    1,
    vec![Tile::Floor, Tile::Floor, Tile::Floor, Tile::Wall],
  )
  .expect("map should validate");
  WorldState::new(
    map,
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(3),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(2),
      ),
    ],
  )
  .expect("world should validate")
}

#[test]
fn snapshot_projects_map_and_actors_deterministically() {
  let source = world();
  let expected_current_time = source.current_time();
  let expected_digest = source.digest();
  let state = PresentationState::new(7, source);
  let snapshot = state.snapshot();

  assert_eq!(snapshot.width(), 4);
  assert_eq!(snapshot.height(), 1);
  assert_eq!(
    snapshot.tiles(),
    &[Tile::Floor, Tile::Floor, Tile::Floor, Tile::Wall]
  );
  assert_eq!(snapshot.actors().len(), 2);
  assert_eq!(snapshot.actors()[0].id(), ActorId::new(1));
  assert_eq!(snapshot.actors()[1].id(), ActorId::new(2));
  assert_eq!(snapshot.current_time(), expected_current_time);
  assert_eq!(snapshot.next_actor(), Some(ActorId::new(1)));
  assert_eq!(snapshot.digest(), expected_digest);
  assert_eq!(snapshot, PresentationState::new(7, world()).snapshot());
}

#[test]
fn supported_keyboard_intent_maps_to_core_commands() {
  let actor = ActorId::new(1);

  assert_eq!(
    KeyboardIntent::from_key(KeyCode::ArrowLeft).map(|intent| intent.command(actor)),
    Some(Command::Move {
      actor,
      direction: Direction::West,
    })
  );
  assert_eq!(
    KeyboardIntent::from_key(KeyCode::KeyW).map(|intent| intent.command(actor)),
    Some(Command::Move {
      actor,
      direction: Direction::North,
    })
  );
  assert_eq!(
    KeyboardIntent::from_key(KeyCode::Space).map(|intent| intent.command(actor)),
    Some(Command::Wait { actor })
  );
  assert_eq!(KeyboardIntent::from_key(KeyCode::KeyQ), None);
}

#[test]
fn accepted_command_emits_events_and_rejected_command_is_atomic() {
  let mut state = PresentationState::new(7, world());
  let accepted = state
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("scheduled player should act");
  assert_eq!(
    accepted.events(),
    &[Event::Moved {
      actor: ActorId::new(1),
      from: Position::new(0, 0),
      to: Position::new(1, 0),
    }]
  );
  assert_eq!(
    accepted.snapshot().actors()[0].position(),
    Position::new(1, 0)
  );
  let mut expected_trace = ReplayTrace::new(7);
  expected_trace.record(Command::Move {
    actor: ActorId::new(1),
    direction: Direction::East,
  });
  assert_eq!(state.replay_digest(), expected_trace.digest());
  let before_rejection = (state.snapshot(), state.replay_digest());

  let error = state
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect_err("the enemy is scheduled after the player's action");
  assert!(error.to_string().contains("not scheduled"));
  assert_eq!(state.snapshot(), before_rejection.0);
  assert_eq!(state.replay_digest(), before_rejection.1);
}

#[test]
fn full_inventory_hides_player_pickup_from_presentation_legal_commands() {
  let mut source = world();
  for id in 1..=u32::try_from(Actor::INVENTORY_CAPACITY).expect("test capacity fits item ids") {
    source
      .give_item(
        ActorId::new(1),
        dreadstep_core::Item::new(
          dreadstep_core::ItemId::new(id),
          dreadstep_core::ItemDefinitionId::new(id + 100),
        ),
      )
      .expect("capacity-sized inventory should be accepted");
  }
  source
    .give_item(
      ActorId::new(2),
      dreadstep_core::Item::new(
        dreadstep_core::ItemId::new(99),
        dreadstep_core::ItemDefinitionId::new(199),
      ),
    )
    .expect("enemy fixture item should be accepted");
  source
    .drop_item(ActorId::new(2), dreadstep_core::ItemId::new(99))
    .expect("enemy fixture item should drop");
  source
    .set_hit_points(ActorId::new(2), HitPoints::new(0))
    .expect("enemy fixture should become a dead retained record");
  source
    .teleport(ActorId::new(1), Position::new(2, 0))
    .expect("player can use the dead enemy tile");

  let state = PresentationState::new(7, source);
  assert!(!state.legal_commands().iter().any(|command| {
    matches!(command, Command::Pickup { actor, item } if *actor == ActorId::new(1) && *item == dreadstep_core::ItemId::new(99))
  }));
}

#[test]
fn replay_commands_expose_only_accepted_commands_in_order() {
  let mut state = PresentationState::new(7, world());
  let move_command = Command::Move {
    actor: ActorId::new(1),
    direction: Direction::East,
  };
  state
    .execute(move_command)
    .expect("scheduled player should move");
  let rejected = Command::Wait {
    actor: ActorId::new(1),
  };
  state
    .execute(rejected)
    .expect_err("enemy should be scheduled after the player moves");

  assert_eq!(state.replay_commands(), &[move_command]);
}
