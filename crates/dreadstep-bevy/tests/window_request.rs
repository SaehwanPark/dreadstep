//! Deterministic headless window request behavior.

use dreadstep_bevy::PresentationWindow;

#[test]
fn valid_request_exposes_logical_and_physical_dimensions() {
  let request = PresentationWindow::new(320, 180, 3).expect("request should validate");

  assert_eq!(request.logical_width(), 320);
  assert_eq!(request.logical_height(), 180);
  assert_eq!(request.pixel_scale(), 3);
  assert_eq!(request.physical_width(), 960);
  assert_eq!(request.physical_height(), 540);
}

#[test]
fn zero_dimensions_and_scale_are_rejected() {
  assert!(PresentationWindow::new(0, 180, 3).is_none());
  assert!(PresentationWindow::new(320, 0, 3).is_none());
  assert!(PresentationWindow::new(320, 180, 0).is_none());
}

#[test]
fn physical_size_overflow_is_rejected() {
  assert!(PresentationWindow::new(u32::MAX, 2, 2).is_none());
  assert!(PresentationWindow::new(2, u32::MAX, 2).is_none());
}

#[test]
fn equal_requests_have_equal_typed_configuration() {
  let first = PresentationWindow::new(640, 360, 2).expect("request should validate");
  let second = PresentationWindow::new(640, 360, 2).expect("request should validate");

  assert_eq!(first, second);
}
