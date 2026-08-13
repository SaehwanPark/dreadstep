//! Protocol projection of authored enemy behavior policies.

use dreadstep_core::EnemyBehavior as CoreEnemyBehavior;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The closed behavior policy authored for an enemy actor.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EnemyBehavior {
  /// Pursue targets using the existing combat policy.
  Pursuer,
  /// Retreat from an adjacent living target when an escape tile exists.
  Kiter,
  /// Break a directly blocking breakable before pursuing a living target.
  Brute,
  /// Cast Chilled along a clear bounded cardinal ray before other combat fallbacks.
  Frostcaster,
  /// Hold position, attacking only when a target is already within melee reach.
  Blocker,
}

impl From<CoreEnemyBehavior> for EnemyBehavior {
  fn from(behavior: CoreEnemyBehavior) -> Self {
    match behavior {
      CoreEnemyBehavior::Pursuer => Self::Pursuer,
      CoreEnemyBehavior::Kiter => Self::Kiter,
      CoreEnemyBehavior::Brute => Self::Brute,
      CoreEnemyBehavior::Frostcaster => Self::Frostcaster,
      CoreEnemyBehavior::Blocker => Self::Blocker,
    }
  }
}

impl From<EnemyBehavior> for CoreEnemyBehavior {
  fn from(behavior: EnemyBehavior) -> Self {
    match behavior {
      EnemyBehavior::Pursuer => Self::Pursuer,
      EnemyBehavior::Kiter => Self::Kiter,
      EnemyBehavior::Brute => Self::Brute,
      EnemyBehavior::Frostcaster => Self::Frostcaster,
      EnemyBehavior::Blocker => Self::Blocker,
    }
  }
}
