# 森林炎爆精灵 · 动画 v1 提示词

使用内置 imagegen，以修订后的 [人设图](../../forest-fire-elf-model-sheet-e-v1.webp) 为身份与服装参考。动画源图为透明 RGBA；导出脚本只做裁切、统一锚点、共享 64 色量化与无损 WebP 封装。以下保留生成与修订提示词。

## 4×4 动画源图

```text
Create ONE NEW square TRUE TRANSPARENT RGBA 4-column × 4-row animation SOURCE SHEET, exactly sixteen equal square cells, for the SAME cheerful adult golden-haired female forest elf shown in the reference model sheet. The model sheet is authoritative for identity and clothes: high light-golden side ponytail, pointed ears, bright grin, short-sleeve ivory tunic, simple deep-green short cape, asymmetrically overlapping matte bark-brown leather wrap-skirt/skort above the knees with no visible stitching, bare calves, woven grass sandals. Her only hand object is a simple thin Y-shaped natural tree twig, with two short blunt fork tips, no leaves, ring, knobs, gemstone or bindings. Keep the staff shape consistent in every frame. All sprites are the same character, proportions, three-quarter facing direction and logical pixel scale. No text, grid lines, panel frames, scenery, floor, shadows or fake checkerboard; true transparent space between cells and around each figure.
Row 1: four distinct normal idle loop frames, cheerful breathing and slight hair/cape/twig sway, feet anchored. Row 2: four distinct focused/awakened idle visual-proposal frames, same clothes and body, a tiny ember between twig tips and slight hair lift, feet anchored. Row 3: four sequential fire-explosion cast frames: wind-up, point twig, ecstatic shout as a compact orange-red blast exits the fork, then spent wobble. Row 4: four sequential immediate post-cast sleep frames: buckle, settle, lie on side asleep with short twig fallen beside her, peaceful breathing. The blast is powerful but must stay within its cell and not hide the elf. She cannot remain standing after casting. Densely detailed handcrafted E-style pixel art with hard square clusters, consistent density, nuanced stepped hue ramps and clear material hierarchy. No blur, antialiasing, smooth painting, random noise or dithering. True RGBA transparency.
```

## 树杈长度修订

```text
Preserve the sixteen animation cells, row order, poses, elf identity, clothes and transparent background. The updated model sheet's short Y-shaped twig held high by the upper-right spell-casting figure is the exact authoritative shape and length. Replace all staffs in all frames with this same short, thin, undecorated twig, approximately one-third of body height. In idle frames it is beside the torso, not touching the ground; in casting frames it is raised; in sleeping frames it is one short fallen twig. Remove all long staff remnants. Keep the fire effect attached to the fork and everything else unchanged.
```

## 火焰越格修订

```text
Preserve the exact transparent 4×4 sheet and all sixteen figures. Change only row 3, column 3's fire effect: retract the bright blast so every orange pixel is strictly inside its own third cell, with transparent safe margin before the boundary at about 75% of sheet width. Remove stray flames or ember blobs in the neighboring fourth cell. Keep the effect powerful but compact and attached to the short twig. No other changes.
```
