//! Contract tests for core-owned run and floor history state.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, FloorAdvanceError, FloorRecord, GridMap, HitPoints,
  RunOutcome, RunState, Tile, WorldState,
};

fn world_with_enemy(enemy_hit_points: u16) -> WorldState {
  WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("test map should be valid"),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        dreadstep_core::Position::new(0, 0),
        HitPoints::new(10),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        dreadstep_core::Position::new(1, 0),
        HitPoints::new(enemy_hit_points),
      ),
    ],
  )
  .expect("test world should be valid")
}

fn victorious_world() -> WorldState {
  let mut world = world_with_enemy(1);
  world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("player attack should succeed");
  assert_eq!(world.outcome(), RunOutcome::Victory);
  world
}

#[test]
fn new_run_records_seed_depth_and_initial_floor() {
  let world = world_with_enemy(2);
  let digest = world.digest();
  let run = RunState::new(41, 3, world);

  assert_eq!(run.seed(), 41);
  assert_eq!(run.depth(), 3);
  assert_eq!(run.world().digest(), digest);
  assert_eq!(
    run.history(),
    &[FloorRecord::new(3, digest, RunOutcome::InProgress)]
  );
}

#[test]
fn execute_updates_the_current_floor_record_without_changing_run_depth() {
  let mut run = RunState::new(41, 3, world_with_enemy(1));

  run
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("player attack should succeed");

  let record = run.history().first().expect("initial floor is recorded");
  assert_eq!(run.depth(), 3);
  assert_eq!(record.depth(), 3);
  assert_eq!(record.state_digest(), run.world().digest());
  assert_eq!(record.outcome(), RunOutcome::Victory);
}

#[test]
fn victory_transition_records_the_new_floor_and_replaces_the_world() {
  let source = victorious_world();
  let source_digest = source.digest();
  let next = world_with_enemy(2);
  let next_digest = next.digest();
  let mut run = RunState::new(41, 3, source);

  let transition = run
    .advance(4, next.clone())
    .expect("victory should permit the next contiguous floor");

  assert_eq!(transition.from_depth(), 3);
  assert_eq!(transition.to_depth(), 4);
  assert_eq!(transition.from_digest(), source_digest);
  assert_eq!(transition.to_digest(), next_digest);
  assert_eq!(run.depth(), 4);
  assert_eq!(run.world(), &next);
  assert_eq!(
    run.history(),
    &[
      FloorRecord::new(3, source_digest, RunOutcome::Victory),
      FloorRecord::new(4, next_digest, RunOutcome::InProgress),
    ]
  );
}

#[test]
fn non_victory_transition_is_typed_and_atomic() {
  let mut run = RunState::new(41, 3, world_with_enemy(2));
  let before = run.clone();

  assert_eq!(
    run.advance(4, world_with_enemy(2)),
    Err(FloorAdvanceError::NotVictorious {
      depth: 3,
      outcome: RunOutcome::InProgress,
    })
  );
  assert_eq!(run, before);
}

#[test]
fn non_contiguous_transition_is_typed_and_atomic() {
  let mut run = RunState::new(41, 3, victorious_world());
  let before = run.clone();

  assert_eq!(
    run.advance(5, world_with_enemy(2)),
    Err(FloorAdvanceError::NonContiguousDepth {
      current: 3,
      requested: 5,
    })
  );
  assert_eq!(run, before);
}

#[test]
fn depth_overflow_is_typed_and_atomic() {
  let mut run = RunState::new(41, u32::MAX, victorious_world());
  let before = run.clone();

  assert_eq!(
    run.advance(u32::MAX, world_with_enemy(2)),
    Err(FloorAdvanceError::DepthOverflow { depth: u32::MAX })
  );
  assert_eq!(run, before);
}

#[test]
fn equivalent_runs_have_equal_history_and_run_digests() {
  let source = victorious_world();
  let next = world_with_enemy(2);
  let mut first = RunState::new(41, 3, source.clone());
  let mut second = RunState::new(41, 3, source);

  let first_transition = first
    .advance(4, next.clone())
    .expect("first transition should succeed");
  let second_transition = second
    .advance(4, next)
    .expect("second transition should succeed");

  assert_eq!(first.history(), second.history());
  assert_eq!(first.digest(), second.digest());
  assert_eq!(first_transition, second_transition);
  assert_ne!(
    first.digest(),
    RunState::new(42, 3, first.world().clone()).digest()
  );
}
