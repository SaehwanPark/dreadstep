# Milestone 3 presentation asset evaluation

Status: verified technical evaluation gate. The files referenced below are local working copies
under ignored `art/` and `audio/` directories; none are production assets or GitHub-tracked files.

Evaluated 2026-08-09 on the merged headless scene-placement boundary (`de26800`). The experiment
keeps source, creator, license, attribution, and modification metadata here rather than beside
ignored binaries.

## Technical gate

- Candidate logical tile sizes: 24×24 and 32×32, both caller-configurable through
  `PresentationTileSize`. The generated PNGs below remain unconstrained contact sheets; exact
  normalized evidence and the provisional decision are recorded in [`tile-samples.md`](tile-samples.md).
- Map coordinates must map through checked integer multiplication and nearest-neighbor sampling;
  no fractional placement or implicit resize may enter the renderer.
- Candidate families: terrain/structures, player and enemy silhouettes, item/UI glyphs, animation
  frames, and event audio cues.
- Local retention: raster art belongs under ignored `art/` or `assets/`; audio belongs under ignored
  `audio/`; tracked records stay in this document and `CREDITS.md` when a candidate is adopted.
- Production integration is intentionally local-only: the selected CC0 fallback is prepared by
  `scripts/prepare-local-assets.sh`, while the archive and generated PNGs remain ignored.

## Candidate evidence

| Candidate | Family | Source / creator / license | Evidence | Evaluation | Decision |
| --- | --- | --- | --- | --- | --- |
| `generated-gothic-24-candidate.png` | Terrain, actor, item, and torch contact sheet | OpenAI image-generation tool; generated for this project; no distribution license asserted | Unmodified generated PNG, 1254×1254; SHA-256 `57febb06dd591ac0b020f54824234f67f195ffff1739fd951c00f92ecaac3ecb` | Broad gothic coverage for unconstrained visual direction; it is not a native 24×24 sample and no cell-size readability claim is made; no animation frames; source is not production-editable | Evaluation-only contact sheet; do not integrate |
| `generated-gothic-32-candidate.png` | Terrain, actor, item, and torch contact sheet | OpenAI image-generation tool; generated for this project; no distribution license asserted | Unmodified generated PNG, 1254×1254; SHA-256 `67efbdde417fb63ad09e982b3db4482e0264aac42c45677770e4f7ad28f1af29` | Broad gothic coverage for unconstrained visual direction; it is not a native 32×32 sample and no cell-size readability claim is made; no animation frames; source is not production-editable | Evaluation-only contact sheet; do not integrate |
| `kenney-tiny-dungeon-preview.png` | Terrain, structures, actors, and items | [Kenney Tiny Dungeon](https://kenney.nl/assets/tiny-dungeon); creator Kenney; Creative Commons CC0 | Unmodified official preview download, 918×515; SHA-256 `b6023d3a9504f84bb128d21680d0899c7a150bce3443d0861590f3f3de1c24b`; source page reports 16×16 tiles and CC0 | Cohesive reusable set with strong coverage and clear grid; exact 24×24/32×32 nearest-neighbor samples from its 16×16 tiles are recorded separately; palette is brighter than the gothic target and needs normalization | Selected local prototype fallback; the preparation mapping below records the six adopted source tiles, while final palette/art direction remains open |
| `generated-cue-click.wav` | Original/generated UI feedback cue | Project-generated PCM candidate; no third-party source or license asserted | Unmodified local WAV; 44.1 kHz mono, 0.16 s; SHA-256 `a05f59c3e9eca9baf18a90a0bc68a8aa8a8f37656d7be29761475fba2cde7c67`; the generation recipe was not retained | Immediate, short, and easy to normalize; suitable for rejected-command/UI feedback but not a dungeon material cue; no variation set | Evaluation-only timing/level reference; do not integrate |
| `kenney-ui-audio.zip` | Reusable UI audio cue set | [Kenney UI Audio](https://kenney.nl/assets/ui-audio); creator Kenney; Creative Commons CC0 | Unmodified official archive download; SHA-256 `946fc23a63d535d693eb31b2eabb80c8c28d6351e2186b344ceb71b2cb1d5eb6`; observed 52 OGG files (51 under `Audio/` plus `Preview.ogg`) and license text; official page advertises 50 files | Strong click/rollover/switch coverage and easy OGG integration; not a combat, movement, dungeon, or item cue library | Reusable UI-prototyping fallback only; final audio source remains open |

## Decisions and open gate

1. **Working tile size: 32×32 provisional.** The generated sheets remain visual direction only;
   the exact nearest-neighbor samples in [`tile-samples.md`](tile-samples.md) preserve clearer
   actor/item silhouettes at 32×32 and provide a 2× integer scale from Kenney's 16×16 source. The
   typed 24×24 option remains available for future comparison.
2. **Pixel-art sourcing: mixed prototype path remains provisional.** Treat the generated sheets as
   visual direction and the CC0 Kenney pack as the selected local integration fallback. The six
   native 16×16 source tiles are scaled by the desktop showcase to the provisional 32×32 logical
   size; final palette/art direction remains open and any changed binary requires a refreshed
   provenance record.
3. **Audio sourcing: UI fallback only.** The generated click establishes timing/level constraints
   and Kenney UI Audio supplies CC0 UI coverage, but neither covers the dungeon cue families in the
   proposal. Combat, movement, pickup, detection, and environmental audio require a later targeted
   source or original set.

The asset gate is verified for the next renderer boundary: native/normalized tile evidence supports
the provisional 32×32 decision, and the UI-only audio fallback plus deferred dungeon-audio decision
are explicit. The local CC0 art fallback is now reproducibly prepared by the checked archive hash and
source mapping below; clean checkouts still retain deterministic colored placeholders when local
media is absent.

## Local prototype adoption

Run `scripts/prepare-local-assets.sh --check` to validate the recorded archive and source members,
then `scripts/prepare-local-assets.sh --install` to populate the ignored desktop paths. The mapping
uses native 16×16 source PNGs and lets the showcase's nearest-neighbor 32×32 sizing preserve the
source pixels:

| Showcase family | Source tile |
| --- | --- |
| `terrain` | `Tiles/tile_0040.png` |
| `player` | `Tiles/tile_0100.png` |
| `enemy` | `Tiles/tile_0112.png` |
| `dead` | `Tiles/tile_0124.png` |
| `ground-item` | `Tiles/tile_0064.png` |
| `inventory-item` | `Tiles/tile_0065.png` |

The script rejects a missing or unexpected archive, validates PNG signatures, requires the exact
media-root grammar, proves each output is Git-ignored, rejects traversal, symlink components and
output leaves, rejects existing non-regular output paths, and never writes outside the caller-selected
ignored destination. It does not alter the simulation, journal schema, or canonical asset manifest.
