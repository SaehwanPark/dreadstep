# Native tile-sample evidence

Status: verified technical sample. These files remain local-only under ignored `art/`; the tracked
record preserves the source and reproducible normalization facts without committing binaries.

## Source

- Official source: [Kenney Tiny Dungeon](https://kenney.nl/assets/tiny-dungeon), creator Kenney,
  Creative Commons CC0.
- Retained archive: `art/kenney-tiny-dungeon.zip`, SHA-256
  `c109438ab06f65fd80f9b2686a4cf9c7c11dc64444b47333ec71d602f8bb5fc7`.
- `Tilesheet.txt` reports 132 source tiles, each 16×16 pixels with 1px sheet spacing. The selected
  source IDs are `0000, 0004, 0012, 0020, 0040, 0052, 0064, 0076, 0088, 0100, 0112, 0124`.
- The downloaded archive is unmodified; source metadata and license text remain inside the local
  archive and the official page is recorded above.

## Normalization method

Each selected 16×16 PNG is decoded without cropping, scaled into a square cell using Core Graphics
with `interpolationQuality = .none`, and placed in row-major order in a 4×3 contact sheet. The
normalization therefore preserves source pixels and uses exact nearest-neighbor mapping; it adds no
palette, alpha, or content edits.

| Output | Sheet dimensions | Cell dimensions | SHA-256 | Git policy |
| --- | ---: | ---: | --- | --- |
| `kenney-tiny-dungeon-samples-24.png` | 96×72 | 24×24 | `53de7c6ab51e48c3c98b6850ae3e27b0c76b250ee227161f862f41dbbaa821a9` | ignored/local-only |
| `kenney-tiny-dungeon-samples-32.png` | 128×96 | 32×32 | `0054ee8f402f168d566ca90e3097402ca2581cefd43aa213ef007c654f6381f8` | ignored/local-only |

## Decision

The 24×24 sample is usable but compresses the representative actor/item silhouettes more strongly
within the same logical viewport. The 32×32 sample preserves clearer silhouettes and provides an
exact 2× integer scale from the 16×16 source. Select 32×32 as the provisional logical working size
for the reversible renderer spike; this does not approve production art, textures, transforms,
asset loading, or audio playback.
