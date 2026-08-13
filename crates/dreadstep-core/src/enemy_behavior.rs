//! Closed authored enemy behavior identities.

/// The deterministic behavior policy assigned to an enemy actor.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EnemyBehavior {
  /// Move toward and attack the controlled target using the existing policy.
  #[default]
  Pursuer,
  /// Retreat one tile when adjacent to the controlled target before other combat fallbacks.
  Kiter,
}
