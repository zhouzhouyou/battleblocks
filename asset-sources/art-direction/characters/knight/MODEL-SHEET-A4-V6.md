# 骑士 K-A4 人设稿 v6 · 独立头盔与石中剑比例修订

日期：2026-10-29。内置 imagegen 编辑，输入 `knight-a4-model-sheet-v5.png`。输出 `knight-a4-model-sheet-v6.png`，待评审，非最终生产素材。

用户原文：
> 装备-头盔 里面不要带上头发。现在石中剑的三视图和你右上角出现的剑不符合，后者的剑露出的部分明显更多。

修订：装备头盔只保留空心金属件；以右上角人物持剑图为比例依据，为三视图增加外露剑刃。三视图使用相同外露长度与石头插入点，石头仍近球形。概念图尚不具备精确机械／正投影验收精度，之前面甲一致性待修事项保留。

## 完整提示词

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.


Use case: precise-object-edit. Edit attached BattleBlocks K-A4 character model sheet. ONLY TWO necessary local corrections; preserve character identity, armor, blue eyes, expressions, entire layout, captions, pixel illustration material style, plain deep crimson cloth, all fullbody figures and helmet mechanism panel.
1. EQUIPMENT helmet (装备分解, leftmost separate object, near x100,y745): remove ALL BLONDE HAIR, ALL BRAIDS, ALL SCALP, ANY HEAD/NECK from this equipment item. It must be a standalone EMPTY METAL HELMET with dark hollow interior, circular pivot and gold wing ornament only. Clean ivory background behind it, no orange/yellow hair shapes sticking above/behind/below helmet. Gold METAL edging is allowed. Do NOT remove the character's blonde hair anywhere else. Only the detached equipment helmet is empty/hairless.
2. BOTTOM WEAPON TURNAROUND: all three small swords currently have only a tiny blade stub outside rock, unlike sword held in upper-right awakened panel. CORRECT bottom three weapon drawings to match the upper-right held stone-sword proportions. Keep upper-right ordinary and awakened weapons unchanged as authority.
Define same scale-independent shape ratios for all three bottom views:
- spherical rock diameter D.
- VISIBLE STRAIGHT STEEL BLADE from guard to top of rock = approximately 0.70 D (long exposed blade shaft), NOT tiny stub and NOT just handle sticking out.
- hilt/grip/pommel above guard = approximately 0.40 D.
- blade width approximately 0.13 D.
- continuous sword visibly embeds into rock at top center; blade does not end before rock; no blade tip protrudes below rock.
All front, SIDE and back use SAME sword hilt design, SAME visible blade length, SAME insertion depth, SAME nearly spherical boulder (front height/width and side depth comparable).
Existing bottom panel too shallow, so SHRINK the whole three weapon drawings uniformly to fit bottom panel while preserving these relative proportions: rock diameters roughly 65-75 display pixels and total assembly roughly 140-160 display pixels tall. Each sword then clearly shows a long straight steel blade between hilt and rock. Do not shrink the blade alone. Keep bottom row labels legible and each complete weapon inside its own space without crossing panel borders.
Rock must remain a SINGLE solid nearly spherical faceted gray stone in every view. No detached pebbles/rubble/chips. Three views are rotations of SAME complete sword-in-stone object. Do not pull sword free. Preserve existing fullbody posing, blue eyes, smile plus musical notes and anger symbol. Keep crisp dense pixel clusters, not smooth painting. Do not alter armor, cape, pure crimson lower cloth or faceplate. Deliver revised complete model sheet.


## 去除残留头发定向修订

Edit ONLY the small detached helmet at left of 装备分解 equipment row, bounding area approximately x30-175 y675-810 on the 1536x1024 sheet. It STILL HAS GOLDEN BLONDE HAIR on top and behind it. Remove ALL these hair pixels: the orange curved scalp-like patch above the black dome behind visor (roughly x90-155 y680-726), AND the orange braided texture hanging behind right ear ornament (roughly x146-163 y754-788). Replace the top orange scalp/hair patch with a smooth DARK CHARCOAL STEEL helmet dome, hard neutral cool-gray highlight, NO warm golden/orange organic texture. Replace rear braid with clean ivory background. The only gold on this EMPTY helmet should be thin METAL perimeter, hinge and wing ornament. Interior opening solid dark empty, no head/hair/skin. Make this look like an empty steel helmet on a table, not helmet worn by a cropped head. Every other panel unchanged including character hair, bottom three long exposed sword blades, figures and faces. Do not add plumes to detached helmet. Preserve detailed crisp pixel art and whole sheet composition.

