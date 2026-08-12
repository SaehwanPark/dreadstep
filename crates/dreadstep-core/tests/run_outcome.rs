//! Deterministic canonical run-outcome projection tests.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, GridMap, HitPoints, Position, RunOutcome, Tile, WorldState,
};

fn world_with_enemy(player_hit_points: u16, enemy_hit_points: u16) -> WorldState {
  WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should be valid"),
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Player,
        Position::new(0, 0),
        HitPoints::new(player_hit_points),
      ),
      Actor::with_hit_points(
        ActorId::new(2),
        ActorKind::Enemy,
        Position::new(1, 0),
        HitPoints::new(enemy_hit_points),
      ),
    ],
  )
  .expect("world should be valid")
}

#[test]
fn outcome_starts_in_progress_and_turns_to_victory_after_the_last_enemy_dies() {
  let mut world = world_with_enemy(10, 1);
  assert_eq!(world.outcome(), RunOutcome::InProgress);

  world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("player attack should succeed");

  assert_eq!(world.outcome(), RunOutcome::Victory);
}

#[test]
fn player_defeat_has_precedence_when_no_living_enemies_remain() {
  let mut world = world_with_enemy(1, 1);
  world
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("player attack should succeed");
  assert_eq!(world.outcome(), RunOutcome::Victory);

  let mut defeat = world_with_enemy(1, 1);
  defeat
    .execute(Command::Attack {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("player attack should succeed");
  defeat
    .set_hit_points(ActorId::new(1), HitPoints::new(0))
    .expect("tester mutation should retain the dead player");
  assert_eq!(defeat.outcome(), RunOutcome::Defeat);
}

#[test]
fn a_world_without_enemies_is_not_implicitly_victorious() {
  let world = WorldState::new(
    GridMap::filled(1, 1, Tile::Floor).expect("map should be valid"),
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("world should be valid");

  assert_eq!(world.outcome(), RunOutcome::InProgress);
}
