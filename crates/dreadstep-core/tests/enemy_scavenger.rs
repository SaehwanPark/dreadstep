//! Deterministic Scavenger enemy behavior.
//!
//! A Scavenger behaves aggressively (pursuing and attacking) at full hit points,
//! but prioritizes retreat when wounded (hit points below maximum).

use dreadstep_core::{
  Actor, ActorId, ActorKind, Command, EnemyBehavior, GridMap, HitPoints, Position, Tile, WorldState,
};

fn world(enemy: Position, player: Position) -> WorldState {
  let width = enemy.x().max(player.x()).cast_unsigned() + 2;
  WorldState::new(
    GridMap::filled(width, 3, Tile::Floor).expect("scavenger map should validate"),
    vec![
      Actor::with_enemy_behavior(ActorId::new(2), enemy, EnemyBehavior::Scavenger),
      Actor::new(ActorId::new(1), ActorKind::Player, player),
    ],
  )
  .expect("scavenger world should validate")
}

fn schedule_enemy(world: &mut WorldState) {
  world
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("player should yield to scavenger");
}

#[test]
fn healthy_scavenger_attacks_adjacent_target() {
  let mut world = world(Position::new(1, 1), Position::new(0, 1));
  schedule_enemy(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  world
    .execute(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("healthy scavenger should execute melee attack");
}

#[test]
fn healthy_scavenger_chases_distant_target() {
  let mut world = world(Position::new(4, 1), Position::new(0, 1));
  schedule_enemy(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Chase {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
}

#[test]
fn wounded_scavenger_retreats_when_adjacent() {
  let mut world = world(Position::new(1, 1), Position::new(0, 1));
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(1))
    .expect("scavenger hit points should be mutable in fixture");
  schedule_enemy(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
  let before_position = world
    .actor(ActorId::new(2))
    .expect("scavenger exists")
    .position();
  world
    .execute(Command::Retreat {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
    .expect("wounded scavenger should execute retreat");
  assert_ne!(
    world
      .actor(ActorId::new(2))
      .expect("scavenger exists")
      .position(),
    before_position
  );
}

#[test]
fn wounded_cornered_scavenger_attacks_if_no_escape_exists() {
  // 2x1 narrow corridor where enemy at (0,0) and player at (1,0) blocks retreat
  let mut world = WorldState::new(
    GridMap::filled(2, 1, Tile::Floor).expect("map should validate"),
    vec![
      Actor::with_enemy_behavior(
        ActorId::new(2),
        Position::new(0, 0),
        EnemyBehavior::Scavenger,
      ),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(1, 0)),
    ],
  )
  .expect("cornered world should validate");
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(1))
    .expect("scavenger hit points should be mutable");
  schedule_enemy(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Attack {
      actor: ActorId::new(2),
      target: ActorId::new(1),
    })
  );
}

#[test]
fn wounded_distant_scavenger_waits_instead_of_chasing() {
  let mut world = world(Position::new(2, 1), Position::new(0, 1));
  world
    .set_hit_points(ActorId::new(2), HitPoints::new(1))
    .expect("scavenger hit points should be mutable");
  schedule_enemy(&mut world);
  assert_eq!(
    world.preferred_enemy_command(ActorId::new(2), ActorId::new(1)),
    Some(Command::Wait {
      actor: ActorId::new(2),
    })
  );
}

#[test]
fn scavenger_identity_participates_in_state_digest() {
  let scavenger = world(Position::new(2, 1), Position::new(0, 1));
  let pursuer = WorldState::new(
    GridMap::filled(4, 3, Tile::Floor).expect("pursuer map should validate"),
    vec![
      Actor::with_enemy_behavior(ActorId::new(2), Position::new(2, 1), EnemyBehavior::Pursuer),
      Actor::new(ActorId::new(1), ActorKind::Player, Position::new(0, 1)),
    ],
  )
  .expect("pursuer world should validate");
  assert_ne!(scavenger.digest(), pursuer.digest());
}
