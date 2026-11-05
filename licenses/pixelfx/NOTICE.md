# PixelFX effects

BattleBlocks' hammer-smash visual is a Rust adaptation of `melee/hammer-smash`
and its swing, stone, dust, crack and force-shell techniques from
[PixelFX](https://github.com/patch-gg/pixelfx), revision
`a84159688af3ca37a4be05a0d422b94451b2e24e`.

Copyright (c) 2026 Patch. Copyright (c) 2026 Hugo Duprez. MIT License; see LICENSE.
PixelFX credits Hugo Duprez's MIT Sprite Fusion effects for the rocks, grit,
rolling dust and floor ring (explosion), the dome (ice nova), and the swing smear
(slash): https://www.spritefusion.com/pixel-art-vfx.

Adaptations include Rust rendering, a fixed visual seed, a 0.4-second board impact,
shorter timing, the BattleBlocks palette, bounded particle paths, fading pixel
clusters, and an independently placed 2x2 board impact. The generated WebP atlas
is included in both native and web builds. Design sources remain outside packages.

BattleBlocks' character landing dust is also a Rust adaptation of
`impacts/heavy-landing` at the same revision. Its staggered dust rings, low puffs,
pebbles and contact streak are adapted from Patch / Hugo Duprez's MIT work;
PixelFX credits Sprite Fusion's explosion for rocks, grit and dust. BattleBlocks
adds separate back/front atlas layers, its own palette and timing, and a shared
Rust hop trajectory for the existing character sprites. No skeletal acting code
is used.

The full-board Ember Blast's expanding billows, ring and cooling ember motifs
reference `fire/fire-ring` and `explosions/big-explosion` at the same revision,
which credit Hugo Duprez's MIT fire/explosion effects. Its board-sized composition,
Rust rasterizer and timing integration are adapted for BattleBlocks. The sleep
loop references `status/sleep`'s staggered rising Z motif; the small bitmap glyphs
are drawn by our renderer, without loading a font or using the stock body/eyes.

Legacy wizard effects reference/adapt `lightning/lightning-strike` and
`fire/meteor` at the same revision: stepped leaders, forked strokes, a cooling
channel, burning stone, a comet tail, fragments and a fading impact. PixelFX
credits Hugo Duprez's MIT lightning/fire/explosion work for these techniques.
BattleBlocks adds its own X-shaped board composition, single central meteor,
diamond wave, mana-return paths and shared clear-triggered visual timing in Rust.

The  wizard revision expands the Lightning Strike motif with a charging sigil,
a thick primary stroke, forks and a cooling ground channel; the meteor's
aftermath is a circular pressure wave with delayed, per-tile visual disappearance.
The original board target sets and resource rules remain those of BattleBlocks.

PixelFX's stock character, skeletal acting system, targets and fonts are not used.

Nun healing overlays adapt `holy/heal` at the same pinned revision: eight foot
petals, a body-height travelling hoop and spiralling heart motes. BattleBlocks
uses an offline pixel rasterizer, two depth layers, gold/cream restoration and
a violet/crimson descending broken-hoop variant, driven by settled reward
events. No stock actor, body recolouring, acting system or fonts are used.
