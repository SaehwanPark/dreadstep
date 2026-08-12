//! Contract tests for local-only audio cue asset metadata.

use bevy::app::App;
use dreadstep_bevy::{
  PresentationAssetReference, PresentationAudioAssetManifest, PresentationAudioAssetProjection,
  PresentationAudioCue, PresentationAudioCueKind, PresentationAudioCues, PresentationPlugin,
  PresentationRuntime,
};
use dreadstep_core::{ActorId, BlockReason, Command, ItemId};

fn reference(path: &str) -> PresentationAssetReference {
  PresentationAssetReference::new(path).expect("fixture path should validate")
}

fn manifest(suffix: &str) -> PresentationAudioAssetManifest {
  PresentationAudioAssetManifest::new(vec![
    (
      PresentationAudioCueKind::Moved,
      reference(&format!("audio/move-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::MovementBlocked,
      reference(&format!("audio/blocked-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::Waited,
      reference(&format!("audio/wait-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::Attacked,
      reference(&format!("audio/attack-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::Died,
      reference(&format!("audio/death-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::ItemEquipped,
      reference(&format!("audio/equip-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::ItemUnequipped,
      reference(&format!("audio/unequip-{suffix}.ogg")),
    ),
    (
      PresentationAudioCueKind::ItemConsumed,
      reference(&format!("audio/consume-{suffix}.ogg")),
    ),
  ])
  .expect("manifest should contain each cue family once")
}

fn all_cues() -> Vec<PresentationAudioCue> {
  vec![
    PresentationAudioCue::Moved {
      actor: ActorId::new(1),
    },
    PresentationAudioCue::MovementBlocked {
      actor: ActorId::new(1),
      reason: BlockReason::Terrain,
    },
    PresentationAudioCue::Waited {
      actor: ActorId::new(1),
    },
    PresentationAudioCue::Attacked {
      attacker: ActorId::new(1),
      target: ActorId::new(2),
    },
    PresentationAudioCue::Died {
      actor: ActorId::new(2),
    },
    PresentationAudioCue::ItemEquipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    },
    PresentationAudioCue::ItemUnequipped {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    },
    PresentationAudioCue::ItemConsumed {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    },
    PresentationAudioCue::ItemPickedUp {
      actor: ActorId::new(1),
      item: ItemId::new(1),
    },
  ]
}

fn audio_app() -> App {
  let mut app = App::new();
  app.insert_resource(PresentationRuntime::start_run(7).expect("starter run should validate"));
  app.insert_resource(PresentationAudioCues::new());
  app.insert_resource(manifest("one"));
  app.insert_resource(PresentationAudioAssetProjection::new());
  app.add_plugins(PresentationPlugin);
  app
}

fn wait_app() -> App {
  let mut app = audio_app();
  app
    .world_mut()
    .resource_mut::<PresentationRuntime>()
    .execute(Command::Wait {
      actor: ActorId::new(1),
    })
    .expect("starter actor should be scheduled");
  app.update();
  app
}

#[test]
fn projection_preserves_all_typed_cues_in_order_without_loading_files() {
  let cues = all_cues();
  let projection = PresentationAudioAssetProjection::from_cues(&cues, &manifest("one"));
  assert_eq!(projection.entries().len(), cues.len());
  for (entry, cue) in projection.entries().iter().zip(cues.iter().copied()) {
    assert_eq!(entry.cue(), cue);
    assert!(entry.reference().path().starts_with("audio/"));
  }
  let expected = [
    "audio/move-one.ogg",
    "audio/blocked-one.ogg",
    "audio/wait-one.ogg",
    "audio/attack-one.ogg",
    "audio/death-one.ogg",
    "audio/equip-one.ogg",
    "audio/unequip-one.ogg",
    "audio/consume-one.ogg",
    "audio/consume-one.ogg",
  ];
  assert_eq!(
    projection
      .entries()
      .iter()
      .map(|entry| entry.reference().path())
      .collect::<Vec<_>>(),
    expected
  );
}

#[test]
fn manifest_requires_exact_audio_families_and_rejects_non_audio_paths() {
  let valid = manifest("one");
  assert_eq!(valid.bindings().len(), 8);
  assert!(PresentationAudioAssetManifest::new(vec![]).is_none());
  let mut crate_local_audio = valid.bindings().to_vec();
  crate_local_audio[0] = (
    PresentationAudioCueKind::Moved,
    reference("crates/dreadstep-bevy/audio/move.ogg"),
  );
  assert!(PresentationAudioAssetManifest::new(crate_local_audio).is_some());
  let mut assets_audio = valid.bindings().to_vec();
  assets_audio[0] = (
    PresentationAudioCueKind::Moved,
    reference("assets/audio/move.ogg"),
  );
  assert!(PresentationAudioAssetManifest::new(assets_audio).is_some());
  for path in [
    "assets/not-audio.wav",
    "art/not-audio.png",
    "crates/dreadstep-bevy/assets/not-audio.wav",
    "crates/dreadstep-bevy/art/not-audio.png",
  ] {
    let mut bindings = valid.bindings().to_vec();
    bindings[0] = (PresentationAudioCueKind::Moved, reference(path));
    assert!(
      PresentationAudioAssetManifest::new(bindings).is_none(),
      "{path} should not be accepted as an audio reference"
    );
  }
  let mut duplicate = valid.bindings().to_vec();
  duplicate[1].0 = PresentationAudioCueKind::Moved;
  assert!(PresentationAudioAssetManifest::new(duplicate).is_none());
}

#[test]
fn runtime_projection_refresh_preserves_cues_and_authority() {
  let mut app = wait_app();
  let before = app
    .world_mut()
    .resource::<PresentationAudioAssetProjection>()
    .entries()
    .to_vec();
  assert_eq!(before.len(), 1);
  let before_cue = app
    .world()
    .resource::<PresentationAudioCues>()
    .cues()
    .to_vec();
  let before_snapshot = app.world().resource::<PresentationRuntime>().snapshot();
  let before_digest = app
    .world()
    .resource::<PresentationRuntime>()
    .replay_digest();
  app.world_mut().insert_resource(manifest("two"));
  app.update();
  let after = app
    .world()
    .resource::<PresentationAudioAssetProjection>()
    .entries()
    .to_vec();
  assert_eq!(after.len(), before.len());
  assert_eq!(after[0].cue(), before[0].cue());
  assert_ne!(after[0].reference(), before[0].reference());
  assert!(after[0].reference().path().ends_with("-two.ogg"));
  assert_eq!(
    app.world().resource::<PresentationAudioCues>().cues(),
    before_cue.as_slice()
  );
  let runtime = app.world().resource::<PresentationRuntime>();
  assert_eq!(runtime.snapshot(), before_snapshot);
  assert_eq!(runtime.replay_digest(), before_digest);
}

#[test]
fn missing_runtime_cue_source_manifest_and_destination_preserve_safely() {
  let mut missing_runtime = wait_app();
  let before_runtime = missing_runtime
    .world()
    .resource::<PresentationAudioAssetProjection>()
    .entries()
    .to_vec();
  missing_runtime
    .world_mut()
    .remove_resource::<PresentationRuntime>();
  missing_runtime.update();
  assert_eq!(
    missing_runtime
      .world()
      .resource::<PresentationAudioAssetProjection>()
      .entries(),
    before_runtime.as_slice()
  );

  let mut missing_cues = wait_app();
  let before_cues = missing_cues
    .world()
    .resource::<PresentationAudioAssetProjection>()
    .entries()
    .to_vec();
  missing_cues
    .world_mut()
    .remove_resource::<PresentationAudioCues>();
  missing_cues.update();
  assert_eq!(
    missing_cues
      .world()
      .resource::<PresentationAudioAssetProjection>()
      .entries(),
    before_cues.as_slice()
  );

  let mut missing_manifest = wait_app();
  let before_manifest = missing_manifest
    .world()
    .resource::<PresentationAudioAssetProjection>()
    .entries()
    .to_vec();
  missing_manifest
    .world_mut()
    .remove_resource::<PresentationAudioAssetManifest>();
  missing_manifest.update();
  assert_eq!(
    missing_manifest
      .world()
      .resource::<PresentationAudioAssetProjection>()
      .entries(),
    before_manifest.as_slice()
  );

  let mut missing_destination = wait_app();
  missing_destination
    .world_mut()
    .remove_resource::<PresentationAudioAssetProjection>();
  missing_destination.update();
  assert!(
    missing_destination
      .world()
      .get_resource::<PresentationAudioAssetProjection>()
      .is_none()
  );
}
