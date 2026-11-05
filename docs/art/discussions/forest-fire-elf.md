# 制作讨论与提示词

以下文字记录设计讨论与制作过程；当前采用资源以角色档案为准。

---

# 森林炎爆精灵 · 人设图  提示词

人设图以讨论阶段第四套皮革下装图为身份参考，矮人  人设图只作为版式参考。使用内置 imagegen 生成；原始输出为 PNG，当前保存为无损 WebP。后续两轮按用户要求修订了树杈造型和长度；工作树只保留修订后的 ，人设历史可由 Git 追溯。

## 主生成提示词

```text
Expand reference image 1 (the earlier four-panel leather-bottom forest elf clothing comparison) into ONE polished wide landscape original game CHARACTER MODEL SHEET. In reference 1, ONLY the FAR-RIGHT FOURTH character is authoritative for design: cheerful adult female forest elf with long light-gold high side ponytail, pointed ears, bright wide confident grin, simple cream SHORT-SLEEVE tunic ending upper thigh, short deep-green shoulder cape, woven tie belt, distinct asymmetrically overlapping BARK-BROWN MATTE LEATHER WRAP-SKIRT/SKORT that forms a short modest knee-above silhouette, bare knees and calves, simple woven straw grass sandals, and ONE unadorned naturally forked tree-branch staff. Preserve the fourth design's overlapping leather silhouette, not the first three shorts. No visible seams, stitching, pockets, buckles, armor, jewelry, gems, hair ornaments or extra accessories. Ref image 2 (dwarf model sheet) is ONLY reference for clean structured high-density pixel model-sheet presentation and clear panels. Do NOT borrow its dwarf, armor, hammer, book, color scheme or motifs.
Composition: wide dark-charcoal model-sheet canvas with restrained thin panel dividers and no lettering. Upper-left: THREE matching-scale full-body turnaround views of the same elf, front, TRUE SIDE, back, showing the short sleeves, short cape, leather wrap skirt, golden ponytail, bare legs and straw sandals correctly from each angle. Upper-right: TWO matching-scale three-quarter full-body acting poses: ordinary form standing boldly with branch staff and open delighted grin; 'awakened / ready to unleash a single enormous fire-explosion spell' with SAME clothes and same staff, one small intense orange-red ember at the branch tip, hair and cape lifted slightly, excited theatrical grin; do not add new costume pieces, wing, aura, giant fireball or ornate weapon. Lower-left: FOUR matching-face expression portraits: sunny laugh, smug 'watch this' grin, spell-release ecstatic shout, deeply asleep peaceful face. Lower-right: carefully separated equipment studies on dark neutral background: short-sleeve cream tunic + plain short green cape as one simple outfit study; overlapping seam-free matte leather wrap skirt shown front and side; a pair of simple woven straw grass sandals; the SAME unadorned forked tree-branch staff. Include one SMALL horizontal acting vignette along bottom: after the gigantic explosion she collapses and sleeps immediately on the ground, naturally on her side, staff lying next to her, no flames, no injury, comedic complete exhaustion. Keep all subjects original and consistent across panels.
Art direction E: densely detailed HAND-PLACED PIXEL ILLUSTRATION, clearly visible intentionally shaped square pixel clusters, coherent pixel scale throughout, crisp staircase contours, nuanced stepped hue shifts for cream cloth, forest-green cape, matte bark leather, golden hair, natural wood and straw. Concentrate detail in face, hands, wrap silhouette and staff while large cloth fields remain quiet. Upper-left lighting, restrained controlled fire effect with clear pixel contours, no random pixel noise, no dithering, no blur or painterly antialiasing, no smooth anime line art. This is a CONCEPT REVIEW SHEET, not a production sprite sheet or calibrated 384x384 frames. No text, labels, logos, watermark, copied fantasy characters, recognizable anime character features or franchise costume elements. Character is lively and cute but unmistakably an adult.
```

## 树杈结构修订

```text
Precise edit to the finished wide character model sheet. Preserve all panel layout, character identity, clothes, asymmetrical brown leather wrap skirt, short sleeves, short cape, sandals, expressions, poses, dark background and pixel art style. Change only the staff everywhere it appears. Use the simple staff shape from the leftmost mushroom forager concept: a plain brown natural branch, thin and slightly crooked, splitting into a modest Y-shaped fork with two short blunt uneven offshoots. No carved shaft, ring, elaborate crown, leaves, mushroom, crystal, ornaments or bindings. Maintain the same staff shape across all panels, including the one lying beside the sleeping elf. One small ember may hover between tips during the fire spell; normal staff has bare wood only.
```

## 用户标注后的最终长度修订

```text
The short Y-shaped twig held high by the spell-casting figure in the upper-right area (approximately x=96.1%, y=5.3%) is the exact authoritative length and silhouette for every depiction of her staff. Replace all other staff depictions with this same short, simple Y-shaped twig of approximately one-third the elf's full body height. In standing views it is hand-held beside her torso and does not extend to the ground. In the equipment study show only one short twig. In the sleeping vignette show one identical short fallen twig. Remove the residual/ghost staff fragment at approximately x=83.9%, y=49.9% and all previous long-staff remnants. Preserve everything else, including layout, outfit, expressions and crisp pixel clusters.
```

用户所指的原参考 A 只用于短树杈形状；人物不采用 A 的草帽、发色或衣装。


---

# 森林炎爆精灵 · 动画  提示词

使用内置 imagegen，以修订后的 人设图 为身份与服装参考。动画源图为透明 RGBA；导出脚本只做裁切、统一锚点、共享 64 色量化与无损 WebP 封装。以下保留生成与修订提示词。

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


---

# 森林炎爆精灵 · 动画资源 （待用户视觉评审）

依据 人设图  制作；**尚未接入游戏**。角色仍在评审阶段，觉醒外观与规则尚未定稿。

## 文件

- `animation-source-.webp`：透明 4×4 原始动画源图，保留生成的高密度细节。
- `grid-384.webp`：透明 1536×1536 图集，每帧 384×384。行序为普通待机、觉醒待机视觉提案、炎爆施法、倒下睡觉；每行四帧。
- `normal-idle-0.webp`：普通待机首帧。
- `normal-idle.webp`、`focused-idle.webp`、`fire-explosion.webp`、`fall-asleep.webp`：四组可循环播放的动画 WebP 预览。
- `report.json`：每帧源图与图集边界、颜色数、共用锚点与导出规格。
- PROMPT.md：源图生成与修订提示词。

复现导出：使用 `.build-tools/python-path.txt` 指定的 Python 运行 `tools/prepare_forest_fire_elf_assets.py`。脚本仅写入本目录，不写 `assets/roles/`，也不修改角色注册表。

## 规格与已知限制

- 16 帧；4×4 图集；每帧 384×384；脚底／地面锚点 `(192,336)`；同一缩放因子确保睡姿不被放大；共用 64 色、无抖动、二值透明、最近邻缩放；WebP 无损。
- 四行分别表现开朗待机、树杈尖端的小火星、炎爆与立即睡觉。觉醒待机只是目前的人设图视觉提案，不代表觉醒技能已确定。
- 这是根据生成源图裁切的**美术评审资源**。源图各行并非严格等高，导出器采用记录在代码中的行边界并清理跨行小碎片。游戏内原尺寸可读性、四帧动作节奏、短树杈一致性和睡姿切换仍需用户验收。
- 未制作角色技能、状态、AI、收益、方块物件或游戏运行时注册；这些不由人设图自动决定。


---

# forest-fire-elf 十二帧动画

Issue #11 第一阶段实现稿，覆盖新五人；尚不代表用户最终美术验收。沿用既定人设，历史生产版本保留。

- 每个 `*-source-.webp` 是内置 ImageGen 生成的独立 4×3 源图，12 张不同姿势；对应 `*-prompt-.md` 保存实际提示词。
- 动作：normal-idle / awake-idle / normal-cast / sleep。运行时每角色 48 帧、384×384，6×8 图集（2304×3072），脚底锚点 (192,336)。
- `*.webp` 动作预览均由运行时同样的量化帧导出。待机一轮 720 ms；施法及入睡 1400 ms；预览为便于比较循环播放，游戏内施法／入睡只播放一次并保持末帧。
- 全角色各自共享至多 64 色、二值透明、最近邻采样、无损 WebP。不是把原四帧重复三次，也没有光流或交叉淡化。
- `report.json` 记录尺寸、色板、源动作统一缩放、帧边界和像素哈希。自动检查不能替代对武器、服装、姿势衔接的视觉审查。
- 保留原人物介绍与能力配置。矮人书本层继续使用 ；火法师两形态共用施法及入睡动作。

复现：用 `.build-tools/python-path.txt` 指定的 Python 安装 `tools/art-requirements.txt` 到 `.build-tools/art-site`，运行 `tools/prepare_character_animations.py`。游戏只打包 `assets/`，本目录源图和预览不进入游戏包。


---

Use case: identity-preserve. BattleBlocks production sprite animation sheet, EXACT same adult female forest fire elf as reference: expressive happy face, long light-golden blonde hair, pointed ears, simple cream short-sleeve blouse, dark green short cape, brown matte leather skirt-shorts above knee, simple sandals, ONE SHORT simple Y-shaped twig wand. No hat, jewelry, ornaments, complicated branches, long staff or costume changes. Match the richly detailed crisp pixel clusters, stepped material ramps, exact proportions and colors. Exactly TWELVE genuinely distinct drawn poses in FOUR columns by THREE rows, equal cells in reading order. TRUE transparent background, no text/numbers/frame lines/checkerboard/floor/shadows. 1536x1152 or larger. Fixed camera, fixed character scale, stable ground baseline in each cell, full body and wand inside margins. No duplicates, no camera zoom, no whole-image motion substitute, no blurry painting. AWAKENED IDLE 12-frame loop: ONLY small orange ember on the short twig tip and slightly lifted hair/cape; same costume and body. Gentle inhale/blink/exhale with shoulders, wrist and hair lag; final poses converge near first. Feet planted. No giant fire, extra aura, armor or ornaments.


---

Use case: identity-preserve. BattleBlocks production sprite animation sheet, EXACT same adult female forest fire elf as reference: expressive happy face, long light-golden blonde hair, pointed ears, simple cream short-sleeve blouse, dark green short cape, brown matte leather skirt-shorts above knee, simple sandals, ONE SHORT simple Y-shaped twig wand. No hat, jewelry, ornaments, complicated branches, long staff or costume changes. Match the richly detailed crisp pixel clusters, stepped material ramps, exact proportions and colors. Exactly TWELVE genuinely distinct drawn poses in FOUR columns by THREE rows, equal cells in reading order. TRUE transparent background, no text/numbers/frame lines/checkerboard/floor/shadows. 1536x1152 or larger. Fixed camera, fixed character scale, stable ground baseline in each cell, full body and wand inside margins. No duplicates, no camera zoom, no whole-image motion substitute, no blurry painting. FIRE CAST 12-frame one-shot:1-3 crouch slightly and bring short Y-twig forward;4-6 straighten and extend twig gradually as a small flame appears;7-8 compact bright orange spell burst at twig tip held forward (large explosion is a separate game effect);9-12 flame fades and she slumps drowsily with closed eyes and lowered twig, preparing to fall asleep. Feet planted. Twig always SHORT, two little prongs, same length in all poses. Effect fully contained inside cell, no large background explosion.


---

Use case: identity-preserve. BattleBlocks production sprite animation sheet, EXACT same adult female forest fire elf as reference: expressive happy face, long light-golden blonde hair, pointed ears, simple cream short-sleeve blouse, dark green short cape, brown matte leather skirt-shorts above knee, simple sandals, ONE SHORT simple Y-shaped twig wand. No hat, jewelry, ornaments, complicated branches, long staff or costume changes. Match the richly detailed crisp pixel clusters, stepped material ramps, exact proportions and colors. Exactly TWELVE genuinely distinct drawn poses in FOUR columns by THREE rows, equal cells in reading order. TRUE transparent background, no text/numbers/frame lines/checkerboard/floor/shadows. 1536x1152 or larger. Fixed camera, fixed character scale, stable ground baseline in each cell, full body and wand inside margins. No duplicates, no camera zoom, no whole-image motion substitute, no blurry painting. NORMAL IDLE subtle 12-frame loop, twig unlit. Holds short twig upright at side, other hand on hip. Frames1-3 cheerful inhale,4-6 blink and small wrist/hair/cape sway,7-9 exhale,10-12 return near pose1. Feet planted, restrained body motion, no attack or flames.


---

Use case: identity-preserve. BattleBlocks production sprite animation sheet, EXACT same adult female forest fire elf as reference: expressive happy face, long light-golden blonde hair, pointed ears, simple cream short-sleeve blouse, dark green short cape, brown matte leather skirt-shorts above knee, simple sandals, ONE SHORT simple Y-shaped twig wand. No hat, jewelry, ornaments, complicated branches, long staff or costume changes. Match the richly detailed crisp pixel clusters, stepped material ramps, exact proportions and colors. Exactly TWELVE genuinely distinct drawn poses in FOUR columns by THREE rows, equal cells in reading order. TRUE transparent background, no text/numbers/frame lines/checkerboard/floor/shadows. 1536x1152 or larger. Fixed camera, fixed character scale, stable ground baseline in each cell, full body and wand inside margins. No duplicates, no camera zoom, no whole-image motion substitute, no blurry painting. FALL ASLEEP 12-frame one-shot, same physical scale THROUGHOUT (lying figure must NOT grow to fill cell). Frame1 standing drowsy, eyes closed, wand lowered;2-3 knees bend;4-5 sits with legs folding;6-7 leans onto side;8-9 lowers head onto folded hands;10-12 fully curled on ground sleeping peacefully, hair settles, tiny breathing differences. One SHORT Y twig rests beside her hand, no second twig. Same ground baseline, no scenery or pillow or Z letters. Frames10-12 must all be fully lying asleep, not standing again. Keep entire lying body comfortably inside cell.
