//! Contract tests for deterministic Zombie (slow pursuer) enemy behavior.

use dreadstep_core::{
  ActionTime, Actor, ActorId, ActorKind, Command, Damage, EnemyBehavior, Event, GridMap, HitPoints,
  Position, Tile, WorldState,
};

fn zombie_combat_world(zombie_pos: Position) -> WorldState {
  let map = GridMap::filled(5, 5, Tile::Floor).expect("map should validate");
  WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_enemy_behavior(ActorId::new(2), zombie_pos, EnemyBehavior::Zombie),
    ],
  )
  .expect("world should validate")
}

#[test]
fn adjacent_zombie_attacks_and_costs_slow_time() {
  let mut world = zombie_combat_world(Position::new(1, 0));
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));

  // Player waits 1 tick at t=0 -> ready at t=1.
  let player_wait = world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait should succeed");
  assert_eq!(player_wait.current_time().value(), 0);
  assert_eq!(world.next_actor(), Some(ActorId::new(2)));

  let preferred = world.preferred_enemy_command(ActorId::new(2), ActorId::new(1));
  assert_eq!(
    preferred,
    Some(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );

  // Zombie attacks adjacent player at t=0 -> takes 2 ticks (ActionCost::SLOW) -> ready at t=2.
  let zombie_result = world
    .execute(preferred.unwrap())
    .expect("zombie attack should succeed");
  assert_eq!(
    zombie_result.events(),
    &[Event::Attacked {
      attacker: ActorId::new(2),
      target: ActorId::new(1),
      damage: Damage::new(1),
      remaining_hit_points: HitPoints::new(9),
    }]
  );
  // Next scheduled actor should be the Player who is ready at t=1!
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));
  assert_eq!(world.current_time().value(), 1);
  assert_eq!(
    world.actor(ActorId::new(2)).unwrap().ready_at(),
    ActionTime::new(2)
  );
}

#[test]
fn distant_zombie_chases_and_costs_slow_time() {
  let mut world = zombie_combat_world(Position::new(4, 0));

  // Player waits at t=0 -> ready at t=1.
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait should succeed");
  assert_eq!(world.next_actor(), Some(ActorId::new(2)));

  let preferred = world.preferred_enemy_command(ActorId::new(2), ActorId::new(1));
  assert_eq!(
    preferred,
    Some(Command::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );

  let zombie_result = world
    .execute(preferred.unwrap())
    .expect("zombie chase should succeed");
  assert_eq!(
    zombie_result.events(),
    &[Event::Moved {
      actor: ActorId::new(2),
      from: Position::new(4, 0),
      to: Position::new(3, 0),
    }]
  );
  // Zombie takes 2 ticks -> ready at t=2.
  assert_eq!(
    world.actor(ActorId::new(2)).unwrap().ready_at(),
    ActionTime::new(2)
  );
  assert_eq!(world.next_actor(), Some(ActorId::new(1)));
  assert_eq!(world.current_time().value(), 1);
}

#[test]
fn chilled_zombie_takes_three_ticks() {
  let map = GridMap::from_tiles(3, 1, vec![Tile::Floor, Tile::Floor, Tile::ChillTrap])
    .expect("map should validate");
  let mut world = WorldState::new(
    map,
    vec![
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
      Actor::with_enemy_behavior(ActorId::new(2), Position::new(1, 0), EnemyBehavior::Zombie),
    ],
  )
  .expect("world should validate");

  // Player waits at t=0 -> ready at t=1.
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait should succeed");

  // Zombie moves East onto ChillTrap at t=0 -> takes 2 ticks (Zombie) -> ready at t=2.
  world
    .execute(Command::Move {
      actor: ActorId::new(2),
      direction: dreadstep_core::Direction::East,
    })
    .expect("zombie move onto chill trap");
  assert_eq!(
    world.actor(ActorId::new(2)).unwrap().ready_at(),
    ActionTime::new(2)
  );
  assert!(world.actor(ActorId::new(2)).unwrap().status().is_some());

  // Player waits at t=1 -> ready at t=2.
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait at t=1");

  // Player waits at t=2 -> ready at t=3.
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player wait at t=2");

  // Now at t=2, Zombie acts while Chilled: 2 (zombie) + 1 (chilled) = 3 ticks -> next ready at 2 + 3 = 5!
  assert_eq!(world.next_actor(), Some(ActorId::new(2)));
  world
    .execute(Command::Wait {
      actor: ActorId::new(2),
    })
    .expect("zombie wait while chilled");
  assert_eq!(
    world.actor(ActorId::new(2)).unwrap().ready_at(),
    ActionTime::new(5)
  );
}

#[test]
fn zombie_behavior_participates_in_state_digest() {
  let pursuer_world = {
    let map = GridMap::filled(3, 3, Tile::Floor).expect("map");
    WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::with_enemy_behavior(ActorId::new(2), Position::new(1, 0), EnemyBehavior::Pursuer),
      ],
    )
    .expect("world")
  };
  let zombie_world = {
    let map = GridMap::filled(3, 3, Tile::Floor).expect("map");
    WorldState::new(
      map,
      vec![
        Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 0)),
        Actor::with_enemy_behavior(ActorId::new(2), Position::new(1, 0), EnemyBehavior::Zombie),
      ],
    )
    .expect("world")
  };

  assert_ne!(pursuer_world.digest(), zombie_world.digest());
  assert_eq!(
    zombie_world
      .actor(ActorId::new(2))
      .unwrap()
      .enemy_behavior(),
    EnemyBehavior::Zombie
  );
}
