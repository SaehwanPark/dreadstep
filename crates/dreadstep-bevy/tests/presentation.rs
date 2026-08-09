//! Deterministic presentation bridge behavior.

use bevy::input::keyboard::KeyCode;
use dreadstep_bevy::{KeyboardIntent, PresentationState};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, Direction, GridMap, HitPoints, Position, Tile, WorldState,
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
  let state = PresentationState::new(7, world());
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
  assert_eq!(snapshot.next_actor(), Some(ActorId::new(1)));
  assert_eq!(snapshot.digest(), state.snapshot().digest());
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
  assert_eq!(accepted.events().len(), 1);
  assert_eq!(
    accepted.snapshot().actors()[0].position(),
    Position::new(1, 0)
  );
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
