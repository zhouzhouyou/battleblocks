# 制作讨论与提示词

以下文字记录设计讨论与制作过程；当前采用资源以角色档案为准。

---

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.

Use case: identity-preserve / stylized-concept.
Asset type: refined character model sheet for BattleBlocks, PC-A, , selected design development.
Input image: selected variant A is the IDENTITY reference and design to refine. Preserve its exact fundamental character: a substantial warm-ivory ceramic cube with dark segmented frame and chamfered corner bumpers, circular face panels with pink hearts, suspended independently next to a separate portal gun. NO face, eyes, hands, arms, legs, wheels, clothes or humanoid body. No new head, helper creature or structural appendages. The character remains a true cube with equal physical width, depth and height. Retain the appealing quiet companion identity.
Create ONE spacious polished landscape character reference sheet, approximately 3:2 aspect ratio. Slate charcoal plain background, restrained technical divider lines and small crisp readable labels. Strong clear pixel illustration, discrete square clusters, stepped material ramps, sharp pixel contours; not glossy smooth 3D, not vector art, no painterly soft edging, no random pixel noise. Both cube and gun share one consistent pixel scale and upper-left lighting. Ivory shell has broad calm planes, selected tiny worn chips and seams; remove scattered noisy scratches. Corners and heart have the most detail.
Layout three clearly separated tiers:
TOP 50%: two large equal hero panels, heading NORMAL left, AWAKENED right. Each shows the entire cube in the SAME three-quarter view, SAME scale, SAME hovering height and same gun aim/position so the phase differences are easy to compare. Cube floats without feet above a compact soft-edged stepped-pixel shadow. Gun hovers on viewer's right, pointing right. Keep a visible gap of about 12% cube width from cube to gun, and gun total length about 80% cube width. Handle remains visibly empty. Portal gun keeps the selected A silhouette: ivory upper carapace, charcoal pistol grip, dark barrel with exposed cyan energy chamber and asymmetric open fork muzzle, tiny orange tips. It is a separate physical prop, never attached to the shell as a cannon.
Refine telekinetic holding: replace the reference's cable-like thick continuous tether with a very faint, interrupted arc of a few short pink-to-cyan energy segments between core and the gun's rear housing. It has NO solid cable, NO plug, NO mechanical attachment, NO tentacle; the gap remains visibly open. In NORMAL, dim pink heart and low cyan chamber glow. In AWAKENED, exactly same geometry, ivory color, heart pattern, gun and framing: stronger pink core and seam light, slightly brighter cyan gun chamber and at most 3 small pixel motes. Do not turn the gun orange to represent awakening. Blue and orange are portal shot colors, independent of awakening. No new armor or accessories.
MIDDLE 25%: three smaller isolated engineering turnarounds of the CUBE ONLY, each same physical proportion, with headings FRONT, SIDE, BACK. These are three views of the SAME cube, not variants. Match corner joints, frame crossings, circular heart panel sizes and ceramic seams from the hero panels; no gun obscuring the turnarounds. Rear face continues the same ceramic/dark-frame design and the same heart emblem treatment as the visible side; no new jet engine or thruster. Use minimal orthographic-style perspective and keep side/back labels correct. Include a small unobtrusive square proportions guide, no fictional numeric claims.
BOTTOM 25%: heading PORTAL GUN, two clean isolated consistent views of the SAME gun (SIDE and THREE-QUARTER), full silhouette and empty grip visible. Add two small muzzle-state chips labelled BLUE SHOT and ORANGE SHOT, showing only the chamber/muzzle luminous color change with identical physical weapon, no projectile explosions. A compact material swatch strip: IVORY CERAMIC / DARK FRAME / PINK CORE / CYAN / ORANGE. No giant paragraph text.
Constraints: visually cohesive, generous gutters, NO clipping, NO overlap between panels. The top hero views are the main focus, turnarounds and equipment are useful readable secondary details. Keep the chosen model recognizable rather than inventing a new design. Normal and awakening are visual specifications only and do not imply new combat rules. This is a character reference for review, not a production-ready animation atlas; do not add a game UI or watermark.


---

# 友谊方块动画  · 当前唯一保留版本

2026-11-06。按用户「只保留最终版本」要求，旧候选与问题动画已清理。保留本图集、六组预览／检查图、两份本体源图层、最终生成提示词及检查记录。用户已授权替换占位角色，当前已接入游戏，本轮未自动提交。

## 内容与动作

- 已选 A 人设：完善人设图。无脸、无四肢，传送枪独立悬浮。
- 六组各 16 帧，共 96 帧：`normal-idle / awake-idle / normal-portal-cast / awake-portal-cast / normal-gravity-cast / awake-gravity-cast`。
- 待机 720 ms；开门／重力各 1400 ms，设计为一次播放后归位。预览为方便评审循环播放。
- 384×384 帧、8×12 图集 `portal-cube-review-atlas-.webp`（3072×4608），共同登记基准 (192,336)，共享 64 色、二值透明、最近邻采样、无损 WebP。
- `*-contact.webp` 为逐帧检查图；两份 `*-rigid-body-template.webp` 是量化前的固定本体源图层。`prompts/*-prompt-.md` 保留最终生成阶段实际使用的提示词文本；已删除的中间图不属于当前包的依赖。

## 刚体修复与验证

源图通过内置 ImageGen 修复；最终采用固定本体／独立枪械的分层配准。每形态三组动作共 48 帧，只平移同一份身体图层，不缩放、不挤压；对齐后身体 RGBA 哈希一致。枪械保留独立的瞄准、后坐和蓝／橙射击变化，完整组合每组 16 个不同帧。

`report.json` 保存图集布局、逐帧哈希、身体区域与时序。已验证所有动画预览的帧数、逐帧时间和无损像素；枪械纹理、衔接与最终视觉效果仍待用户评审。

## 复现

用 `.build-tools/python-path.txt` 指定的 Python 执行：

```text
tools/prepare_portal_cube_animations.py
```

默认只验证最终图集和已有预览。添加 `--export` 从最终图集重建六组动画和检查图，不需要旧源图、不重新量化，也不会改动画素。

添加 `--install` 验证后导出 `assets/roles/portal-cube/runtime/atlas.webp`，将 96 帧无损重排为 10×10 格、3840×3840 的网页兼容纹理，并更新独立缩略图和资源报告；所有其他角色缩略图像素保持不变。


---

Use case: precise-object-edit / identity-preserve. REPAIR the supplied animation sheet.
Input image 1 is the authoritative PC-A MODEL. Input image 2 is the DEFECTIVE 4x4 animation sheet to repair, NOT an acceptable geometric reference.
The human explicitly rejected the cube because it is squashed, stretched and changing shape between frames. This is a rigid ROBOT made of ceramic and metal, never a soft character. Correct every cell of image 2. Do not rationalize bad geometry as squash-and-stretch, perspective variation or acting.
OUTPUT: one square sprite sheet, exactly 4 columns x 4 rows = SIXTEEN chronological frames, equally sized and spaced on truly transparent background. No text, numbers, labels, grid, shadows, backdrop or visual noise. At least 8% margin within each cell. Keep both cube and gun fully inside each cell.
MOST IMPORTANT REPAIR: Choose ONE exact 3/4 view of the cube in image 1. REPEAT THAT EXACT SAME RIGID CUBE RENDER in all 16 cells. Its projected outer silhouette, front-face quadrilateral, side-face quadrilateral, top-face quadrilateral, heart circle, corner blocks, seams and panel landmarks MUST keep exactly the same dimensions and angles in every frame. Camera and object yaw/pitch/roll are LOCKED. Do not rotate or tilt the body in this repair. The cube may TRANSLATE vertically by a few pixels, but do not scale or redraw it with a new shape. Match cube width and height in all cells. No narrow/tall cubes, flattened cubes, rounded blobs, eyes, arms, legs, limbs or added attachments.
Do the same for the portal gun: ONE exact weapon silhouette matching image 1, kept rigid and the same size. It floats with visibly empty handle beside the cube, never a mechanical arm or attached cannon. Its pose can translate and pitch slightly as a whole rigid object, without warping its barrel, handle or fork. Pink-to-cyan link is faint broken energy, never solid cable.
Use pixel illustration with crisp square clusters, stepped colors, upper-left lighting. Match selected A ceramic ivory, charcoal frame, heart-pink, cyan chamber and orange fork tips. Remove stray nontransparent red/gray background noise. No blur, no smooth painted 3D. No new details or changing scratches between cells.
Frame registration: cube width exactly 40% of cell width, visible height about 48% in this one fixed 3/4 perspective; same in every cell. Cube center x=31% of cell; gun center x=75%. Both together fit inside 8%-92% of cell width. Camera zoom never changes. Body and gun stay clearly separate.

AWAKENED: heart is bright pink in EVERY frame, including the first and last. Same shell and gun as normal. No new geometry. Do not let most awakened frames revert to unlit normal heart.
16-frame gravity calibration ACTION with NO BODY ROTATION in this repaired version. Use vertical translation and gun orbit to show the action: frames 1-4 cube slowly rises 2% cell height and gun moves upward separately; 5-8 cube holds the SAME rigid pose near high point while gun traces a small arc at its upper-right and pitches upward by 10 degrees; 9-12 gun descends on a curved arc, cube lowers; 13-16 recover base hover position and horizontal gun. Heart brightens at middle. At most three tiny square spatial motes. No muzzle shot, no huge ring, no body deformation.


---

Use case: precise-object-edit / identity-preserve. REPAIR the supplied animation sheet.
Input image 1 is the authoritative PC-A MODEL. Input image 2 is the DEFECTIVE 4x4 animation sheet to repair, NOT an acceptable geometric reference.
The human explicitly rejected the cube because it is squashed, stretched and changing shape between frames. This is a rigid ROBOT made of ceramic and metal, never a soft character. Correct every cell of image 2. Do not rationalize bad geometry as squash-and-stretch, perspective variation or acting.
OUTPUT: one square sprite sheet, exactly 4 columns x 4 rows = SIXTEEN chronological frames, equally sized and spaced on truly transparent background. No text, numbers, labels, grid, shadows, backdrop or visual noise. At least 8% margin within each cell. Keep both cube and gun fully inside each cell.
MOST IMPORTANT REPAIR: Choose ONE exact 3/4 view of the cube in image 1. REPEAT THAT EXACT SAME RIGID CUBE RENDER in all 16 cells. Its projected outer silhouette, front-face quadrilateral, side-face quadrilateral, top-face quadrilateral, heart circle, corner blocks, seams and panel landmarks MUST keep exactly the same dimensions and angles in every frame. Camera and object yaw/pitch/roll are LOCKED. Do not rotate or tilt the body in this repair. The cube may TRANSLATE vertically by a few pixels, but do not scale or redraw it with a new shape. Match cube width and height in all cells. No narrow/tall cubes, flattened cubes, rounded blobs, eyes, arms, legs, limbs or added attachments.
Do the same for the portal gun: ONE exact weapon silhouette matching image 1, kept rigid and the same size. It floats with visibly empty handle beside the cube, never a mechanical arm or attached cannon. Its pose can translate and pitch slightly as a whole rigid object, without warping its barrel, handle or fork. Pink-to-cyan link is faint broken energy, never solid cable.
Use pixel illustration with crisp square clusters, stepped colors, upper-left lighting. Match selected A ceramic ivory, charcoal frame, heart-pink, cyan chamber and orange fork tips. Remove stray nontransparent red/gray background noise. No blur, no smooth painted 3D. No new details or changing scratches between cells.
Frame registration: cube width exactly 40% of cell width, visible height about 48% in this one fixed 3/4 perspective; same in every cell. Cube center x=31% of cell; gun center x=75%. Both together fit inside 8%-92% of cell width. Camera zoom never changes. Body and gun stay clearly separate.

AWAKENED: heart is bright pink in EVERY frame, including the first and last. Same shell and gun as normal. No new geometry. Do not let most awakened frames revert to unlit normal heart.
16-frame idle LOOP. The EXACT SAME unrotated cube translates vertically on a gentle small sine-like hover cycle, amplitude 2% of cell height. Gun follows one quarter-beat later, translates independently and changes rigid barrel pitch only +/-3 degrees. No firing. Final frame almost returns to first without duplicating it. Visible distinct frame compositions from gun positions, hover heights and tiny heart pulse, not from changing body shape.


---

Use case: precise-object-edit / identity-preserve. REPAIR the supplied animation sheet.
Input image 1 is the authoritative PC-A MODEL. Input image 2 is the DEFECTIVE 4x4 animation sheet to repair, NOT an acceptable geometric reference.
The human explicitly rejected the cube because it is squashed, stretched and changing shape between frames. This is a rigid ROBOT made of ceramic and metal, never a soft character. Correct every cell of image 2. Do not rationalize bad geometry as squash-and-stretch, perspective variation or acting.
OUTPUT: one square sprite sheet, exactly 4 columns x 4 rows = SIXTEEN chronological frames, equally sized and spaced on truly transparent background. No text, numbers, labels, grid, shadows, backdrop or visual noise. At least 8% margin within each cell. Keep both cube and gun fully inside each cell.
MOST IMPORTANT REPAIR: Choose ONE exact 3/4 view of the cube in image 1. REPEAT THAT EXACT SAME RIGID CUBE RENDER in all 16 cells. Its projected outer silhouette, front-face quadrilateral, side-face quadrilateral, top-face quadrilateral, heart circle, corner blocks, seams and panel landmarks MUST keep exactly the same dimensions and angles in every frame. Camera and object yaw/pitch/roll are LOCKED. Do not rotate or tilt the body in this repair. The cube may TRANSLATE vertically by a few pixels, but do not scale or redraw it with a new shape. Match cube width and height in all cells. No narrow/tall cubes, flattened cubes, rounded blobs, eyes, arms, legs, limbs or added attachments.
Do the same for the portal gun: ONE exact weapon silhouette matching image 1, kept rigid and the same size. It floats with visibly empty handle beside the cube, never a mechanical arm or attached cannon. Its pose can translate and pitch slightly as a whole rigid object, without warping its barrel, handle or fork. Pink-to-cyan link is faint broken energy, never solid cable.
Use pixel illustration with crisp square clusters, stepped colors, upper-left lighting. Match selected A ceramic ivory, charcoal frame, heart-pink, cyan chamber and orange fork tips. Remove stray nontransparent red/gray background noise. No blur, no smooth painted 3D. No new details or changing scratches between cells.
Frame registration: cube width exactly 40% of cell width, visible height about 48% in this one fixed 3/4 perspective; same in every cell. Cube center x=31% of cell; gun center x=75%. Both together fit inside 8%-92% of cell width. Camera zoom never changes. Body and gun stay clearly separate.

AWAKENED: heart is bright pink in EVERY frame, including the first and last. Same shell and gun as normal. No new geometry. Do not let most awakened frames revert to unlit normal heart.
16-frame paired portal shooting ACTION. Body stays the EXACT SAME shape/camera/angle; its center translates back left 1% for anticipation (1-3), right to neutral during shot (4-6), left for second anticipation (7-9), right to neutral (10-12), settles (13-16). Gun independently aims, recoils to left with small CYAN muzzle pulse during 4-6, recovers and aims slightly upward 7-9, recoils with ORANGE muzzle pulse 10-12, returns to cyan-neutral 13-16. No giant effects. Gun silhouette rigid. Body no tilting, squashing or stretching.


---

Use case: precise-object-edit / identity-preserve. REPAIR the supplied animation sheet.
Input image 1 is the authoritative PC-A MODEL. Input image 2 is the DEFECTIVE 4x4 animation sheet to repair, NOT an acceptable geometric reference.
The human explicitly rejected the cube because it is squashed, stretched and changing shape between frames. This is a rigid ROBOT made of ceramic and metal, never a soft character. Correct every cell of image 2. Do not rationalize bad geometry as squash-and-stretch, perspective variation or acting.
OUTPUT: one square sprite sheet, exactly 4 columns x 4 rows = SIXTEEN chronological frames, equally sized and spaced on truly transparent background. No text, numbers, labels, grid, shadows, backdrop or visual noise. At least 8% margin within each cell. Keep both cube and gun fully inside each cell.
MOST IMPORTANT REPAIR: Choose ONE exact 3/4 view of the cube in image 1. REPEAT THAT EXACT SAME RIGID CUBE RENDER in all 16 cells. Its projected outer silhouette, front-face quadrilateral, side-face quadrilateral, top-face quadrilateral, heart circle, corner blocks, seams and panel landmarks MUST keep exactly the same dimensions and angles in every frame. Camera and object yaw/pitch/roll are LOCKED. Do not rotate or tilt the body in this repair. The cube may TRANSLATE vertically by a few pixels, but do not scale or redraw it with a new shape. Match cube width and height in all cells. No narrow/tall cubes, flattened cubes, rounded blobs, eyes, arms, legs, limbs or added attachments.
Do the same for the portal gun: ONE exact weapon silhouette matching image 1, kept rigid and the same size. It floats with visibly empty handle beside the cube, never a mechanical arm or attached cannon. Its pose can translate and pitch slightly as a whole rigid object, without warping its barrel, handle or fork. Pink-to-cyan link is faint broken energy, never solid cable.
Use pixel illustration with crisp square clusters, stepped colors, upper-left lighting. Match selected A ceramic ivory, charcoal frame, heart-pink, cyan chamber and orange fork tips. Remove stray nontransparent red/gray background noise. No blur, no smooth painted 3D. No new details or changing scratches between cells.
Frame registration: cube width exactly 40% of cell width, visible height about 48% in this one fixed 3/4 perspective; same in every cell. Cube center x=31% of cell; gun center x=75%. Both together fit inside 8%-92% of cell width. Camera zoom never changes. Body and gun stay clearly separate.

NORMAL: heart dim pink throughout except a restrained pulse while casting. Do not make normal identity change into another model.
16-frame gravity calibration ACTION with NO BODY ROTATION in this repaired version. Use vertical translation and gun orbit to show the action: frames 1-4 cube slowly rises 2% cell height and gun moves upward separately; 5-8 cube holds the SAME rigid pose near high point while gun traces a small arc at its upper-right and pitches upward by 10 degrees; 9-12 gun descends on a curved arc, cube lowers; 13-16 recover base hover position and horizontal gun. Heart brightens at middle. At most three tiny square spatial motes. No muzzle shot, no huge ring, no body deformation.


---

Use case: precise-object-edit / identity-preserve. REPAIR the supplied animation sheet.
Input image 1 is the authoritative PC-A MODEL. Input image 2 is the DEFECTIVE 4x4 animation sheet to repair, NOT an acceptable geometric reference.
The human explicitly rejected the cube because it is squashed, stretched and changing shape between frames. This is a rigid ROBOT made of ceramic and metal, never a soft character. Correct every cell of image 2. Do not rationalize bad geometry as squash-and-stretch, perspective variation or acting.
OUTPUT: one square sprite sheet, exactly 4 columns x 4 rows = SIXTEEN chronological frames, equally sized and spaced on truly transparent background. No text, numbers, labels, grid, shadows, backdrop or visual noise. At least 8% margin within each cell. Keep both cube and gun fully inside each cell.
MOST IMPORTANT REPAIR: Choose ONE exact 3/4 view of the cube in image 1. REPEAT THAT EXACT SAME RIGID CUBE RENDER in all 16 cells. Its projected outer silhouette, front-face quadrilateral, side-face quadrilateral, top-face quadrilateral, heart circle, corner blocks, seams and panel landmarks MUST keep exactly the same dimensions and angles in every frame. Camera and object yaw/pitch/roll are LOCKED. Do not rotate or tilt the body in this repair. The cube may TRANSLATE vertically by a few pixels, but do not scale or redraw it with a new shape. Match cube width and height in all cells. No narrow/tall cubes, flattened cubes, rounded blobs, eyes, arms, legs, limbs or added attachments.
Do the same for the portal gun: ONE exact weapon silhouette matching image 1, kept rigid and the same size. It floats with visibly empty handle beside the cube, never a mechanical arm or attached cannon. Its pose can translate and pitch slightly as a whole rigid object, without warping its barrel, handle or fork. Pink-to-cyan link is faint broken energy, never solid cable.
Use pixel illustration with crisp square clusters, stepped colors, upper-left lighting. Match selected A ceramic ivory, charcoal frame, heart-pink, cyan chamber and orange fork tips. Remove stray nontransparent red/gray background noise. No blur, no smooth painted 3D. No new details or changing scratches between cells.
Frame registration: cube width exactly 40% of cell width, visible height about 48% in this one fixed 3/4 perspective; same in every cell. Cube center x=31% of cell; gun center x=75%. Both together fit inside 8%-92% of cell width. Camera zoom never changes. Body and gun stay clearly separate.

NORMAL: heart dim pink throughout except a restrained pulse while casting. Do not make normal identity change into another model.
16-frame idle LOOP. The EXACT SAME unrotated cube translates vertically on a gentle small sine-like hover cycle, amplitude 2% of cell height. Gun follows one quarter-beat later, translates independently and changes rigid barrel pitch only +/-3 degrees. No firing. Final frame almost returns to first without duplicating it. Visible distinct frame compositions from gun positions, hover heights and tiny heart pulse, not from changing body shape.


---

Use case: precise-object-edit / identity-preserve. REPAIR the supplied animation sheet.
Input image 1 is the authoritative PC-A MODEL. Input image 2 is the DEFECTIVE 4x4 animation sheet to repair, NOT an acceptable geometric reference.
The human explicitly rejected the cube because it is squashed, stretched and changing shape between frames. This is a rigid ROBOT made of ceramic and metal, never a soft character. Correct every cell of image 2. Do not rationalize bad geometry as squash-and-stretch, perspective variation or acting.
OUTPUT: one square sprite sheet, exactly 4 columns x 4 rows = SIXTEEN chronological frames, equally sized and spaced on truly transparent background. No text, numbers, labels, grid, shadows, backdrop or visual noise. At least 8% margin within each cell. Keep both cube and gun fully inside each cell.
MOST IMPORTANT REPAIR: Choose ONE exact 3/4 view of the cube in image 1. REPEAT THAT EXACT SAME RIGID CUBE RENDER in all 16 cells. Its projected outer silhouette, front-face quadrilateral, side-face quadrilateral, top-face quadrilateral, heart circle, corner blocks, seams and panel landmarks MUST keep exactly the same dimensions and angles in every frame. Camera and object yaw/pitch/roll are LOCKED. Do not rotate or tilt the body in this repair. The cube may TRANSLATE vertically by a few pixels, but do not scale or redraw it with a new shape. Match cube width and height in all cells. No narrow/tall cubes, flattened cubes, rounded blobs, eyes, arms, legs, limbs or added attachments.
Do the same for the portal gun: ONE exact weapon silhouette matching image 1, kept rigid and the same size. It floats with visibly empty handle beside the cube, never a mechanical arm or attached cannon. Its pose can translate and pitch slightly as a whole rigid object, without warping its barrel, handle or fork. Pink-to-cyan link is faint broken energy, never solid cable.
Use pixel illustration with crisp square clusters, stepped colors, upper-left lighting. Match selected A ceramic ivory, charcoal frame, heart-pink, cyan chamber and orange fork tips. Remove stray nontransparent red/gray background noise. No blur, no smooth painted 3D. No new details or changing scratches between cells.
Frame registration: cube width exactly 40% of cell width, visible height about 48% in this one fixed 3/4 perspective; same in every cell. Cube center x=31% of cell; gun center x=75%. Both together fit inside 8%-92% of cell width. Camera zoom never changes. Body and gun stay clearly separate.

NORMAL: heart dim pink throughout except a restrained pulse while casting. Do not make normal identity change into another model.
16-frame paired portal shooting ACTION. Body stays the EXACT SAME shape/camera/angle; its center translates back left 1% for anticipation (1-3), right to neutral during shot (4-6), left for second anticipation (7-9), right to neutral (10-12), settles (13-16). Gun independently aims, recoils to left with small CYAN muzzle pulse during 4-6, recovers and aims slightly upward 7-9, recoils with ORANGE muzzle pulse 10-12, returns to cyan-neutral 13-16. No giant effects. Gun silhouette rigid. Body no tilting, squashing or stretching.
