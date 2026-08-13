//! Contract tests for one-use kick-noise enemy investigation.

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, CommandError, Event, GridMap, Position, Tile, WorldState,
};

fn world_with_enemy(enemy_position: Position) -> WorldState {
  WorldState::new(
    GridMap::from_tiles(
      8,
      3,
      vec![
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Floor,
        Tile::Door,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
      ],
    )
    .expect("noise map should validate"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, enemy_position),
    ],
  )
  .expect("noise world should validate")
}

fn kick(world: &mut WorldState) {
  kick_at(world, Position::new(2, 1));
}

fn kick_at(world: &mut WorldState, position: Position) {
  world
    .execute(Command::Kick {
      actor: ActorId::new(1),
      position,
    })
    .expect("adjacent door should be kickable");
}

fn open_noise_world(enemy_position: Position) -> WorldState {
  let mut map = GridMap::filled(8, 5, Tile::Floor).expect("noise map should validate");
  for x in 0..8 {
    map.set_tile(Position::new(x, 0), Tile::Wall);
    map.set_tile(Position::new(x, 4), Tile::Wall);
  }
  for y in 0..5 {
    map.set_tile(Position::new(0, y), Tile::Wall);
    map.set_tile(Position::new(7, y), Tile::Wall);
  }
  map.set_tile(Position::new(2, 2), Tile::Door);
  WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 2)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, enemy_position),
    ],
  )
  .expect("noise world should validate")
}

fn investigates(world: &WorldState) -> bool {
  world
    .legal_commands()
    .iter()
    .any(|command| matches!(command, Command::Investigate { .. }))
}

#[test]
fn kick_arms_nearby_enemy_and_investigation_moves_toward_noise() {
  let mut world = world_with_enemy(Position::new(5, 1));
  kick(&mut world);

  let legal = world.legal_commands();
  let investigation = Command::Investigate {
    actor: ActorId::new(2),
    position: Position::new(2, 1),
  };
  assert!(legal.contains(&investigation));
  assert!(
    legal.iter().position(|command| *command == investigation)
      < legal
        .iter()
        .position(|command| matches!(command, Command::Chase { .. }))
  );
  let result = world
    .execute(Command::Investigate {
      actor: ActorId::new(2),
      position: Position::new(2, 1),
    })
    .expect("heard noise should be investigable");
  assert_eq!(
    result.events(),
    &[Event::Moved {
      actor: ActorId::new(2),
      from: Position::new(5, 1),
      to: Position::new(4, 1),
    }]
  );
  assert_eq!(world.actor(ActorId::new(2)).unwrap().heard_noise(), None);
}

#[test]
fn investigation_is_one_use_and_blocked_steps_still_clear_hearing() {
  let mut world = world_with_enemy(Position::new(5, 1));
  kick(&mut world);
  world.set_tile(Position::new(4, 1), Tile::Wall);
  let result = world
    .execute(Command::Investigate {
      actor: ActorId::new(2),
      position: Position::new(2, 1),
    })
    .expect("blocked investigation still spends its action");
  assert!(matches!(result.events()[0], Event::MovementBlocked { .. }));
  assert_eq!(world.actor(ActorId::new(2)).unwrap().heard_noise(), None);
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield the retry turn");
  assert_eq!(
    world.execute(Command::Investigate {
      actor: ActorId::new(2),
      position: Position::new(2, 1),
    }),
    Err(CommandError::NoNoiseToInvestigate(ActorId::new(2)))
  );
}

#[test]
fn kick_does_not_arm_distant_or_dead_enemies_and_rejection_is_atomic() {
  let mut world = world_with_enemy(Position::new(6, 1));
  kick(&mut world);
  assert!(
    !world
      .legal_commands()
      .iter()
      .any(|command| matches!(command, Command::Investigate { .. }))
  );
  assert_eq!(
    world.execute(Command::Investigate {
      actor: ActorId::new(2),
      position: Position::new(2, 1),
    }),
    Err(CommandError::NoNoiseToInvestigate(ActorId::new(2)))
  );
  let mut dead = world_with_enemy(Position::new(5, 1));
  dead
    .set_hit_points(ActorId::new(2), dreadstep_core::HitPoints::new(0))
    .expect("tester death should be valid");
  kick(&mut dead);
  assert_eq!(dead.actor(ActorId::new(2)).unwrap().heard_noise(), None);
}

#[test]
fn ranged_attack_stays_ahead_of_investigation_in_enemy_legal_order() {
  let mut world = WorldState::new(
    GridMap::from_tiles(
      8,
      3,
      vec![
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Floor,
        Tile::Door,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
      ],
    )
    .expect("noise map should validate"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(2), ActorKind::Enemy, Position::new(5, 1)),
      Actor::new(ActorId::new(3), ActorKind::Player, Position::new(3, 1)),
    ],
  )
  .expect("noise world should validate");
  kick(&mut world);
  let legal = world.legal_commands();
  let ranged = legal
    .iter()
    .position(|command| matches!(command, Command::RangedAttack { target, .. } if *target == ActorId::new(3)))
    .expect("clear ranged target should be legal");
  let investigate = legal
    .iter()
    .position(|command| matches!(command, Command::Investigate { .. }))
    .expect("noise investigation should be legal");
  assert!(ranged < investigate);
}

#[test]
fn propagation_arms_exactly_three_steps_but_not_four() {
  let mut three = open_noise_world(Position::new(5, 2));
  kick_at(&mut three, Position::new(2, 2));
  assert!(investigates(&three));

  let mut four = open_noise_world(Position::new(6, 2));
  kick_at(&mut four, Position::new(2, 2));
  assert!(!investigates(&four));
}

#[test]
fn propagation_blockers_occlude_noise_but_short_detours_and_actors_do_not() {
  for blocker in [Tile::Wall, Tile::Door, Tile::Breakable] {
    let mut world = open_noise_world(Position::new(4, 2));
    world.set_tile(Position::new(3, 2), blocker);
    kick_at(&mut world, Position::new(2, 2));
    assert!(!investigates(&world), "{blocker:?} should block noise");
  }

  let mut detour = open_noise_world(Position::new(3, 3));
  detour.set_tile(Position::new(3, 2), Tile::Wall);
  kick_at(&mut detour, Position::new(2, 2));
  assert!(
    investigates(&detour),
    "a two-step walkable detour should be audible"
  );

  let mut occupied = WorldState::new(
    GridMap::from_tiles(
      7,
      5,
      vec![
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Floor,
        Tile::Door,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Wall,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Wall,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Floor,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
        Tile::Wall,
      ],
    )
    .expect("occupied noise map should validate"),
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 1)),
      Actor::new(ActorId::new(2), ActorKind::Player, Position::new(3, 1)),
      Actor::new(ActorId::new(3), ActorKind::Enemy, Position::new(4, 1)),
    ],
  )
  .expect("occupied noise world should validate");
  kick_at(&mut occupied, Position::new(2, 1));
  occupied
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("the occupying actor should yield to the heard enemy");
  assert!(
    occupied.legal_commands().iter().any(
      |command| matches!(command, Command::Investigate { actor, .. } if *actor == ActorId::new(3))
    ),
    "actor occupancy should not block propagation"
  );
}
