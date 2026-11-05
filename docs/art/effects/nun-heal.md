# Nun Healing overlays

Reference: PixelFX `holy/heal`, pinned at `a84159688af3ca37a4be05a0d422b94451b2e24e` (MIT). Attribution and full license: `licenses/pixelfx/`.

Rebuild from the repository root with the Python recorded in `.build-tools/python-path.txt`, running `tools/prepare_nun_heal.py`. Pillow draws integer pixel clusters directly, without antialiasing or a font in the production atlases. Preview-only labels use Pillow's default font. No PixelFX fonts, stock actors, skeletal poses or body recolouring are used.

- Normal: gold/cream eight-petal foot bloom, rising hoop, ascending heart motes.
- Converted damage: violet/crimson bloom, descending broken hoop, split hearts.
- 160×192 per frame; anchor (80,176); 40 frames at 20fps; back/front layers, each 5 rows of 8 frames. Both ending frames are fully transparent.
- Runtime loads both lossless WebP atlases at GUI initialization. It chooses recipients and variants from uncancelled settled events whose source chain contains Healing. Combat values and character atlas files are untouched.
- Previews composite the existing nun frames for review only. Sources and previews are excluded from game packages. Effects also work around the opposing character's shared foot anchor.

Visual review: first pass checked rise/descent and layered hoop occlusion; second pass added inner petal veins and a deeper corrupted ramp; third pass enlarged heart clusters for readability. Native review covered ordinary healing and simultaneous normal/corrupted playback under Frenzy. Character health popups use compact columns below the board during playback so they do not obscure either body or the status row.
