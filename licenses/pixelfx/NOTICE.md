# PixelFX hammer-smash

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

PixelFX's stock character, skeletal acting system, targets and fonts are not used.
