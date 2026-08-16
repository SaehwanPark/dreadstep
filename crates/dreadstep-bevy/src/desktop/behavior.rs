//! Authored enemy behavior labels shared by HUD and journal projections.

use dreadstep_core::EnemyBehavior;

pub(super) const fn enemy_behavior_name(behavior: EnemyBehavior) -> &'static str {
  match behavior {
    EnemyBehavior::Pursuer => "Pursuer",
    EnemyBehavior::Kiter => "Kiter",
    EnemyBehavior::Brute => "Brute",
    EnemyBehavior::Frostcaster => "Frostcaster",
    EnemyBehavior::Blocker => "Blocker",
    EnemyBehavior::Scavenger => "Scavenger",
    EnemyBehavior::Zombie => "Zombie",
  }
}

pub(super) const fn enemy_behavior_value(behavior: EnemyBehavior) -> &'static str {
  match behavior {
    EnemyBehavior::Pursuer => "pursuer",
    EnemyBehavior::Kiter => "kiter",
    EnemyBehavior::Brute => "brute",
    EnemyBehavior::Frostcaster => "frostcaster",
    EnemyBehavior::Blocker => "blocker",
    EnemyBehavior::Scavenger => "scavenger",
    EnemyBehavior::Zombie => "zombie",
  }
}
