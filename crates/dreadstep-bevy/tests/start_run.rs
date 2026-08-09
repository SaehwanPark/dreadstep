//! Presentation startup over the authored content boundary.

use dreadstep_bevy::PresentationState;
use dreadstep_content::starter_floor;
use dreadstep_core::ReplayTrace;

#[test]
fn start_run_delegates_to_shared_content_and_preserves_seed() {
  let seed = 41;
  let state = PresentationState::start_run(seed).expect("starter content should validate");
  let content = starter_floor().expect("same starter content should validate");

  assert_eq!(state.seed(), seed);
  assert_eq!(state.snapshot().digest().value(), content.digest().value());
  assert_eq!(state.snapshot().width(), 7);
  assert_eq!(state.snapshot().height(), 5);
  assert_eq!(state.snapshot().actors().len(), 4);
  assert_eq!(state.replay_digest(), ReplayTrace::new(seed).digest());
}
