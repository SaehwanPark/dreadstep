//! Closed authored enemy behavior identities.

/// The deterministic behavior policy assigned to an enemy actor.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EnemyBehavior {
  /// Move toward and attack the controlled target using the existing policy.
  #[default]
  Pursuer,
  /// Retreat one tile when adjacent to the controlled target before other combat fallbacks.
  Kiter,
  /// Break a directly blocking breakable on the deterministic chase step before pursuing.
  Brute,
  /// Apply the existing Chilled status along a clear cardinal ranged ray before other fallbacks.
  Frostcaster,
  /// Hold position, attacking only when the controlled target enters melee reach.
  Blocker,
  /// Pursue and attack at full hit points, but retreat when wounded.
  Scavenger,
}
