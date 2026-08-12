//! Deterministic one-shot floor trap contract tests.

use dreadstep_core::{
  ActionCost, ActionTime, Actor, ActorId, ActorKind, Command, Damage, Direction, Event, GridMap,
  HitPoints, Position, RunOutcome, Tile, WorldState,
};

fn trap_world() -> WorldState {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Trap, Tile::Floor])
    .expect("trap map should validate");
  WorldState::new(
    map,
    vec![Actor::new(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
    )],
  )
  .expect("trap world should validate")
}

#[test]
fn entering_a_trap_moves_then_emits_damage_and_consumes_the_tile() {
  let mut world = trap_world();
  let before_digest = world.digest();
  let result = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("entering a trap should remain an accepted movement action");

  assert_eq!(world.map().tile_at(Position::new(1, 0)), Some(Tile::Floor));
  assert_ne!(world.digest(), before_digest);
  assert_eq!(
    result.events(),
    &[
      Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      },
      Event::TrapTriggered {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        damage: Damage::new(1),
        remaining_hit_points: HitPoints::new(9),
      },
    ]
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().hit_points(),
    HitPoints::new(9)
  );
  assert_eq!(
    result.current_time(),
    ActionTime::new(ActionCost::STANDARD.value())
  );
}

#[test]
fn a_trap_is_one_shot_and_can_kill_the_entering_actor() {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Trap, Tile::Floor])
    .expect("trap map should validate");
  let mut world = WorldState::new(
    map,
    vec![Actor::with_hit_points(
      ActorId::new(1),
      ActorKind::Player,
      Position::new(0, 0),
      HitPoints::new(1),
    )],
  )
  .expect("trap world should validate");

  let lethal = world
    .execute(Command::Move {
      actor: ActorId::new(1),
      direction: Direction::East,
    })
    .expect("lethal trap entry should be accepted");
  assert_eq!(
    lethal.events(),
    &[
      Event::Moved {
        actor: ActorId::new(1),
        from: Position::new(0, 0),
        to: Position::new(1, 0),
      },
      Event::TrapTriggered {
        actor: ActorId::new(1),
        position: Position::new(1, 0),
        damage: Damage::new(1),
        remaining_hit_points: HitPoints::new(0),
      },
      Event::Died {
        actor: ActorId::new(1),
      },
    ]
  );
  assert_eq!(world.outcome(), RunOutcome::Defeat);
  assert_eq!(world.map().tile_at(Position::new(1, 0)), Some(Tile::Floor));
  assert!(world.next_actor().is_none());
}

#[test]
fn enemy_chase_reuses_trap_trigger_and_traps_are_walkable_and_visible() {
  assert!(Tile::Trap.is_walkable());
  assert!(!Tile::Trap.blocks_ranged_line_of_sight());

  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Trap, Tile::Floor])
    .expect("trap map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::with_hit_points(
        ActorId::new(1),
        ActorKind::Enemy,
        Position::new(2, 0),
        HitPoints::new(3),
      ),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(0, 0)),
    ],
  )
  .expect("trap world should validate");

  let result = world
    .execute(Command::Chase {
      actor: ActorId::new(1),
      target: ActorId::new(2),
    })
    .expect("enemy chase should enter the trap");
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().position(),
    Position::new(1, 0)
  );
  assert_eq!(
    world.actor(ActorId::new(1)).unwrap().hit_points(),
    HitPoints::new(2)
  );
  assert!(
    result
      .events()
      .iter()
      .any(|event| matches!(event, Event::TrapTriggered { .. }))
  );
}
