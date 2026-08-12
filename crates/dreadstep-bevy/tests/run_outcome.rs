//! Presentation projection tests for canonical run outcomes.

use dreadstep_bevy::{PresentationState, RunOutcome};
use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Position, Tile, WorldState,
};

fn state() -> PresentationState {
  let world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(1),
      ),
    ],
  )
  .expect("world should be valid");
  PresentationState::new(7, world)
}

#[test]
fn presentation_snapshot_tracks_core_victory_projection() {
  let mut state = state();
  assert_eq!(state.snapshot().outcome(), RunOutcome::InProgress);
  state
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("attack should succeed");
  assert_eq!(state.snapshot().outcome(), RunOutcome::Victory);
}
