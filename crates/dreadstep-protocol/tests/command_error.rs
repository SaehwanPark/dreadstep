//! Contract tests for ranged command rejection conversion.

use dreadstep_core::{ActorId as CoreActorId, CommandError as CoreCommandError};
use dreadstep_protocol::{ActorId, CommandError};

#[test]
fn ranged_out_of_range_error_maps_to_the_typed_protocol_variant() {
  assert_eq!(
    CommandError::from(CoreCommandError::RangedAttackOutOfRange {
      attacker: CoreActorId::new(1),
      target: CoreActorId::new(2),
    }),
    CommandError::RangedAttackOutOfRange {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    }
  );
}
