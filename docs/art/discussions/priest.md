# 制作讨论与提示词

以下文字记录设计讨论与制作过程；当前采用资源以角色档案为准。

---

# 狗头人牧师人设图

日期：2026-11-01。使用内置 imagegen，以用户选定的狗头人蓝金牧师稿为视觉来源，生成人设图。这是待评审概念图，不是游戏图集；未替换当前 `assets/roles/priest/runtime/atlas.webp`。

## 用户决定

> 还是用狗头人。手上可以拿个纯金的小法杖，暗示这个牧师并不干净。配色可以采用蓝色和金色。

> 法杖前面得是个球。另外服装方面可以参考我给的安度因，但是不要太复杂。

> 法杖太短了显得很奇怪。法杖可以加一点简单的装饰，主要是表现财富。

> 方块上不需要有任何图案。好了，基于这个版本生成人设图

狗头人、简洁蓝金牧师服、长纯金球头法杖、腰带正面两个口袋和无图案方块为本版锁定方向。附图仅作配色与服装简洁结构参考，不复制其人物或复杂装饰。

## 生成提示词

Use case: identity-preserve / stylized-concept. Create ONE wide landscape CHARACTER MODEL SHEET for the selected BattleBlocks dog-headed kobold priest, using the selected full-body image as exact character-design source. Original Western fantasy E-style densely detailed pixel illustration: carefully shaped crisp pixel clusters, consistent pixel scale, nuanced stepped hue ramps, no smooth painting, no random noise. Warm ivory neutral board, clean separated panels, no text or labels.

Keep the same dog-headed kobold identity: short coarse brown fur, broad canine muzzle, uneven triangular ears, amber sly eyes, compact wiry build. Royal-blue high-collar priest robe with modest gold edging; one simple cream prayer stole and small original gold sun emblem. Exactly two practical brown leather pockets attached to the front belt, left and right of center. One long entirely solid-gold staff, nearly character height, straight shaft with a few raised gold rings and a smooth round gold sphere on top. No gems, spikes, flame, or excessive ornament.

Left area: same-scale full-body front, side and back turnarounds, aligned feet. Right upper area: polite, delighted and annoyed expressions. Right lower area: full staff and ball-tip detail, front-view two-pocket belt, open pocket, and pocketing action. Every square game tile must be a completely plain single-color square with no pattern, emblem, sun, writing, icon, engraving or decoration. No extra accessories, book, halo, pouches or armor. Do not crop figures or props.

## 局部修订

首图误把两只口袋画在背面。最终图删除背视图腰带上的口袋，只保留正面腰带两只口袋；方块仍为无图案纯色方块。其余人设与版面保持不变。


---

> 历史记录：这一版人设图已按 2026-11-01 清理移出仓库，本文件只保留提示词与文字记录。

# 牧师 P-C 人设稿

日期：2026-10-29。内置 imagegen，参考 `priest-drinking-candidates-e-.webp` 第三列。用户原文：「按 p-c 来」。已选人物方向，新人设图待评审。仍按用户要求不提交 Git。

## 完整提示词

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.


Use case: identity-preserve / stylized-concept.
Create ONE large landscape detailed CHARACTER MODEL SHEET for selected P-C priest, original BattleBlocks western fantasy 2D desktop/web game. Attached image is selection source: use ONLY THIRD COLUMN P-C adult FEMALE cleric, ignore other columns. Same project E detailed crisp pixel-cluster illustration, stepped hue-shifted material ramps, NOT smooth anime painting, NOT coarse 8bit.
Preserve P-C: adult woman around thirties, short tousled wavy AUBURN BOB, AMBER eyes both sides, expressive warm confident smile, normal human proportions, practical cream high-neck long-sleeve robe with full ivory sleeves, forest-green split cape/mantle with restrained gold sun embroidery and gold tassels, leather belt and boots. One sun brooch/clasp, one consistent golden sun emblem design everywhere. Green/gold wide-bowl footed WINE CHALICE with dark burgundy wine; forest-green hardback holy BOOK with same gold sun emblem. No priest staff, shield, extra weapons or nun veil. Original lighthearted wine-loving priest, not vampire/wizard. Consistent ADULT head/body ratio and identity in all views.
Layout on warm ivory board titled "牧师 P-C · 人设稿":
LEFT HALF three large same-scale fullbody turnarounds labeled 正面 / 侧面 / 背面, aligned feet, arms relaxed hands EMPTY for garment construction. Same length and cut across views. ONE ivory robe front hem and side split, dark trousers and brown boots under it. Cape green outside, slightly darker green inside, same gold border and sun motif placement in all views; no color flip or multiplying skirt layers. Braid absent, short bob consistent.
UPPER RIGHT ordinary and awakened fullbody pair labeled 普通 / 觉醒. RIGHT HAND holds chalice, LEFT HAND holds same open book in BOTH states (viewer left/right depends view); SAME book size and cup size relative to hand across panels. Ordinary relaxed toast/sip pose, warm confident. Awakened purposeful half-step and extended toast, open book near torso, a few controlled warm-gold pixel blessings around SAME chalice rim and book emblem only. No added armor, hair, crowns, wings or huge halos. Awake pose/effects are visual proposal, not a skill definition. Outfit completely identical to ordinary. Don't transform cup or book.
MIDDLE RIGHT exactly three face closeups labeled 平静 / 开心 / 生气. Same short auburn bob and both AMBER eyes. Calm relaxed half-smile, happy visible open smile/joyful cheeks plus small external musical notes, angry furrowed brows with tense mouth plus red anger mark. No eye-color drift, no forehead accessories appearing.
BOTTOM equipment breakdown: EMPTY cream robe (no head/skin), EMPTY green mantle (no hair), belt and boots, detached sun brooch, wine chalice, holy book. Do not duplicate new accessories. Equipment exactly matches character.
Below/along bottom, focused prop studies with labels 圣杯 正面 / 侧面 / 俯视 and 圣典 闭合 / 展开: SAME green enamel cup with warm gold sun/leaf trim and sturdy short stem, bowl has REAL depth, top view shows liquid surface. Same diameter/depth and gold motif positioning across views. Book same green cover, gold SUN emblem, cream paper, same rectangular thickness in closed/open. Equipment not attached to severed hands. Cup not bowl-shaped one panel and narrow glass another. Staff absent. No separate generic crosses replacing chosen sun.
Clean readable panel arrangement, ample margins, all full figures and props fit, large faces/hands and material emphasis, quiet cloth surfaces, no random noise or fake watercolor grain. Only title and listed section labels; no invented lore, skills or stats. This is concept model sheet for review, not a claim of production pixel-grid compliance.


---

# 牧师 P-C 人设稿  · 圣杯俯视修订

日期：2026-10-30。使用内置 imagegen 编辑 ；仅修订底部圣杯「俯视」示意。原  保留，不提交 Git。角色、技能及其余设计的评审状态不变。

## 修订目标

 的俯视仍呈倾斜视角，可见外杯身与杯脚。本次改为正上方的正交俯视：圆形杯口、同心金色杯沿、窄内壁和酒红色液面；杯梗、杯脚及正面外壁徽记被杯碗遮挡。其他图与版面保持原样。

## 完整提示词

Use case: precise-object-edit.
Edit target: attached BattleBlocks priest P-C model sheet . Make ONE tiny localized correction ONLY to the bottom-left prop studies panel, the RIGHTMOST chalice directly above the Chinese label 俯视 (approximately x=500..635 y=845..975 in the 1536x1024 reference).
Problem: this currently depicts a tilted perspective view with the outer bowl and foot visible. Replace that single chalice illustration with a geometrically correct orthographic TOP-DOWN view, camera directly vertically above the chalice. The round cup opening must be a CIRCLE, not a tilted ellipse. Show a thin concentric golden rim, the inner wall as a narrow concentric band, and a circular horizontal dark burgundy wine surface with restrained pixel highlights. Stem, foot, front exterior sun badge, and side silhouette must be fully hidden beneath the bowl in this view. Same cup diameter, gold/green materials and design as the neighboring front and side studies. Center this circular view in the existing allotted space above 俯视 with enough margin.
Preserve absolutely all other pixels/composition/content: characters, faces, expressions, all costumes, equipment, other cup views, book diagrams, panel borders, every Chinese label, title, parchment background and exact image dimensions/aspect ratio.
Style: existing densely detailed pixel illustration, carefully shaped crisp pixel clusters, nuanced hue shifts and stepped material ramps, consistent pixel scale; no blur or smooth vector redraw. Maintain existing original Western fantasy design. No additional objects or effects. This is a small correction to concept art, not a redesign.


---

# 牧师候选 · E 方案

日期：2026-10-29。工具：内置 imagegen。状态：四稿待选择，非最终素材；按用户最新要求，候选阶段只保存不提交 Git。

参考：本地 `references/user-drinking-priest-2026-10-29.png`（未入库）仅为概念／服装参考，其内文和比例说明不是用户已选设定；骑士  与全局 E 图用于像素材质风格参照。

## 完整提示词

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.


Use case: stylized-concept.
Create ONE large landscape FOUR-CANDIDATE CHARACTER COMPARISON BOARD for original BattleBlocks western fantasy 2D turn-based match3 game, desktop/web. Title "牧师 · 酒与祝祷". Four columns labeled P-A, P-B, P-C, P-D. Quiet warm ivory background, thin separators. Each column top large fullbody ordinary labeled 普通, middle large fullbody awakened labeled 觉醒, footer single clear head portrait and two small separated identifying props (drinking vessel and holy emblem). NOT a final model sheet or UI interface. Big enough to compare personalities and coherent art as a whole.
Input images:
1 user's drinking priest model sheet = concept/costume mood reference ONLY, adult wine-drinking cleric, goblet, sacred staff, layered vestments. Do NOT use its smooth anime painting style or its written biography/abilities/proportion specifications.
2 current knight model sheet = shared project rendering quality and adult human proportions, carefully shaped visible pixel clusters, stepped material colors, crisp contours, no airbrush. Match this project's E detailed pixel illustration, not smooth anime or coarse 8bit.
3 E global style guide = supporting pixel-cluster language only, ignore its roster and old awakening armor transformations.
FOUR meaningfully distinct adult priest candidates, all unmistakably clerics who enjoy drinking, each has readable sacred emblem and liturgical stole / vestments so not mistaken for wizard, tavern patron or pirate. Original fictional geometric sun/cross emblem consistent within each candidate. No copying franchise characters.
P-A: young ADULT male, messy ash-brown side-part medium hair, clean shaven, lean adult proportions, kindly mischievous half-smile. Ivory ankle-length robe, deep charcoal short mantle, wine-red stole, restrained warm gold edging. One hand holds tall sacred circle-cross STAFF; other holds elegant FOOTED WINE GOBLET with dark red wine. Relaxed dignified toast ordinary. Awakened same face/hair/clothing/objects, raises goblet in expansive blessing; tiny warm gold pixel light catches cup rim and staff emblem. Closest concept to input1 but original, cleaner vestments.
P-B: middle-aged adult male, very BROAD sturdy human torso (NORMAL human height, NOT dwarf), cropped dark hair, full short beard, rolled-up cream sleeves, layered rustic brown/ivory cleric robe and broad dark ochre stole, leather belt. Large wooden ALE TANKARD with metal rim in one hand, simple short holy-symbol staff in other. Laughing warm tavern-chaplain energy. Awakened plants feet wide and offers robust toast, staff emblem warm gold and a small neat blessing arc from tankard. Same gear, no new barrel armor or hair.
P-C: adult FEMALE priest in her thirties, short auburn wavy bob, confident amber-eyed friendly grin, tall straight silhouette. Practical high-neck cream robe with forest-green split mantle, warm brass sun brooch, trousers/boots visible below robe openings, no nun veil, no pointed wizard hat, no revealing outfit. Short broad metal WINE CHALICE in one hand, compact sacred BOOK carried open in other; clear stole and holy brooch. Awakened same identity and outfit, purposeful step forward, holds chalice outward and open book near chest, restrained warm gold pixel blessing marks. No new hair/armor.
P-D: elderly adult male, slender stooped human silhouette, receding gray hair, short pointed gray beard, arched brows and dry amused smile. Weathered charcoal/cream travel cassock, dusty burgundy scarf-like stole with simple holy mark, walking-staff holy symbol and WICKER-WRAPPED WINE FLASK held in hand. No dwarf proportions, no wizard beard to floor. Awakened straightens slightly and tilts flask in solemn humorous blessing, tiny gold highlights on flask holy emblem and staff. Same elderly face and robe, no age reversal.
All candidates ordinary versus awakened STRICT identity/equipment continuity, no added wings, armor, crowns, extra arms, oversized halos or big light effects. Awakened differences are VISUAL PROPOSALS ONLY, do not write skills/stat numbers. Different silhouettes/ages/gender/clothing cuts and drinking objects, NOT recolors of same person. Everyday vessels clearly rendered with quality material clusters; objects don't fuse to fingers. Every figure has exactly two arms, clear hands and no extra hidden weapon.
Adult human anatomy and sensible head/body ratio consistent with knight; not children or tiny chibi. Each column room for full staff and toes, no cropping, no overflow into next column. Pixel clusters deliberate, shading 3-4 hard steps around face/material focus, quiet large cloth planes, no random speckled noise, no blurred glow. 24-color family overall with controlled local ramps; previews not production sprites.
Footer per column includes face closeup showing personality and isolated EMPTY VESSEL shape + holy emblem, labeled only "酒器" and "圣徽". Props are candidate proposals, not finalized tile icons. Empty equipment vessel no floating hand attached.
Text limited to title, P-A/P-B/P-C/P-D and 普通/觉醒/酒器/圣徽, no invented lore or translated reference biography.


---

> 历史记录：这一版动画图片已按 2026-11-01 清理移出仓库，本目录只保留提示词；当前使用 production/。

# priest · 游戏动画

2026-10-30。内置 imagegen，根据当前人设制作透明帧图；生成源图需经网格、透明度、色数与锚点校准后接入。

## 初始完整提示词

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.
Use case: identity-preserve. Asset type: transparent game animation SPRITE SHEET based on the attached reference model sheet.
Create an exactly square 4-column by 4-row grid of 16 separate full-body frames on a genuinely TRANSPARENT background. NO lettering, NO titles, NO grids, NO frame borders, NO floor, NO cast shadow, NO UI. Each identical cell has ample transparent gutters. Every frame uses the same frontal three-quarter facing slightly right. Consistent anatomy, head size, outfit, equipment, pixel grid, scale and feet anchor. Crisp densely detailed PIXEL ILLUSTRATION with deliberately stepped 3-4-tone material ramps, coherent pixel clusters, hard pixel edges, no smooth digital painting, no blurry downscaled art. High detail readability rather than coarse 8bit. Think each frame drawn intentionally on a 96x96 logical canvas and enlarged 4x by nearest-neighbor. Uniform cell size and fixed camera. Each full subject fits its cell completely with at least 8% margins.
Row1: four subtle normal IDLE cycle frames (breathing, hair and cloth settling), feet remain grounded.
Row2: four subtle AWAKENED IDLE cycle frames, same identity and all clothing.
Row3: four sequential CAST / ATTACK frames: anticipation, windup, release, recovery. This must be actual articulated movement, not four duplicate poses.
Row4: four sequential HIT/RECOVER frames: slight recoil, stronger recoil, regaining stance, return toward idle. No blood, no dying/disappearing, no new accessories.
Read the reference as identity and design authority. Do NOT include portraits or equipment breakdowns from the reference.
Character: adult FEMALE PRIEST P-C, short wavy AUBURN bob, AMBER eyes, cream full-sleeve robe, dark forest-green mantle with gold sun embroidery, brown belt with EXACTLY TWO small pocket pouches, brown boots. One gold SUN brooch. RIGHT hand holds a wide green-and-gold CHALICE with burgundy wine. LEFT hand holds a green HOLY BOOK with matching gold sun emblem. Identity and costume invariant. No staff, no wings, no crown, no extra armor or hair. Normal idle relaxed confident toast. Awakened idle same character raises the SAME cup a little higher, only restrained warm-gold blessing pixels around cup and book. Row3 casts by extending chalice and opening book, one coherent gesture. Row4 protects book and cup while recoiling, never swaps hands, never spills into a background. Human body top logical y=20 and feet y=84, 64-pixel body scale. Ample transparent space.

## 动画与一致性修订

Edit this transparent 4x4 female priest sprite atlas. Keep exact identity, short auburn bob, amber eyes, green/cream/gold costume, TWO belt pouches, right-hand chalice and left-hand holy book. Preserve row1. Row2 awakened idle: REMOVE gigantic glowing aura, halos and large stars; keep only a few small warm-gold blessing PIXELS immediately near cup rim/book emblem, same costume and body, subtle stable breathing loop. Row3 normal CAST: remove large glowing halo/ribbon backdrop, keep articulated book/chalice gesture and very restrained local gold pixels. Replace bottom ROW4 with four sequential AWAKENED CAST poses: anticipation opening book, extending chalice, blessing release, recovery. Same appearance as row2, no new armor/hair/wings/crown, just small controlled golden pixel marks by cup/book. Same square 4x4 grid, consistent scale/gutters/anchors, full subject in every cell, genuine transparency, crisp dense pixel clusters, no labels/grid/floor/background. No huge light effects obscuring silhouette.



---

> 历史记录：这一版动画图片已按 2026-11-01 清理移出仓库，本目录只保留提示词；当前使用 production/。

# 高分辨率游戏帧

2026-10-31。生成源图与完整提示词沿用 ..//PROMPT.md；按照用户「一半身高但像素不应模糊」的纠正，从原始高分辨率生成图重新导入。每帧384×384，正常人形身体256像素、矮人128像素，游戏中仍为1:2身高；三名角色采用相同纹理像素密度。固定每角色64色共享色板，无抖色，二值透明；4×4图集1536×1536，最近邻采样。没有将的96像素结果放大。


---

# 游戏动画与统一展示

2026-10-31。生成提示词沿用 ..//PROMPT.md；高清导入依据 ..//PROMPT.md。本版遵从用户新决定：「矮人只要说明矮就行了，不需要真的只展示一半的大小。另外，现在牧师明显比骑士矮一些，没必要，保持一样的像素。」

统一384×384帧，正常／觉醒待机的可见全身目标高度320像素，共用脚底锚点(192,336)。矮保留在人物轮廓、介绍和玩法中，UI不再按1:2缩放；三人使用同级纹理精度和展示高度。每角色64色共享色板，二值透明，最近邻采样；从原始高分辨率生成图重新导入。骑士额外清理双辫错误，保持单侧一根粗辫；面罩开启／闭合与一剑一石约束不变。


---

# 当前生产动画

依据已选定人设生成；四行依次为普通待机、觉醒待机、普通施法、觉醒施法，每行四帧。源图 animation-source.webp；提示词 PROMPT.md。导入结果每帧 384×384，共 16 帧，脚底 (192,336)，可见高度标尺 320；共享 64 色、二值透明、最近邻采样。人物显示大小三人一致。/ 为历史迭代，图片已移除、仅保留提示词，不作为运行时来源；仇恨之书复用  拆分。

normal-idle.webp、awake-idle.webp、normal-cast.webp、awake-cast.webp 用于预览；运行时为 assets/roles/ 下无损 WebP。导入像素检查记录于 config/role-assets.json。


---

# 狗头人牧师动画源

使用内置 imagegen，以 狗头人人设图  为身份参考，生成透明背景 4×4 源图 `animation-source.webp`。

## 生成提示词

Create a perfectly aligned 4 columns × 4 rows grid of sixteen distinct full-body frames of the same BattleBlocks dog-headed kobold priest from the selected model sheet. Square overall canvas, genuinely transparent background, no panel borders, no labels, no floor. One character in each equal-sized cell; aligned feet, same pixel scale and 3/4 front camera. Do not crop staff, ears, feet, tail or pockets.

Lock design: adult brown dog-headed kobold, amber eyes, uneven triangular ears, broad muzzle, compact wiry build; royal-blue simple high-collar priest robe with modest gold edging, cream prayer stole and small original gold sun emblem; exactly two brown leather square-tile pockets on the front belt; long solid-gold staff nearly full character height, simple smooth round gold-ball tip and a few raised bands. Staff stays in the same hand. No book, cup, jewelry, armor, halo or extra bags.

Row 1: normal idle, subtle breathing and blink. Row 2: awakened idle, same clothes, only small controlled warm-gold light at the staff sphere. Row 3: normal cast, glance toward pocket, draw blank square tile, present it, return. Row 4: awakened cast, same action with restrained sphere light. Every tile is completely plain and single-color, with no pattern, emblem, sun, icon, writing or ornament. Crisp E-style pixel clusters and stepped hue ramps, no blur or random noise.

## 导入

`tools/prepare_role_assets.py` 从此目录读取源图，只做确定性的裁切、锚点对齐、最近邻缩放、64 色共享色板和二值透明导入。帧序：普通待机 0–3、觉醒待机 4–7、普通施法 8–11、觉醒施法 12–15。觉醒视觉光点是动画表现，不改变技能规则。源图保留供追溯；`assets/roles/priest/runtime/atlas.webp` 是运行时资源。


---

# 狗头人牧师游戏素材

新狗头人牧师以蓝金简洁牧师袍、纯金球头长杖和腰带正面两只口袋替换旧 P-C 图像。`animation-source.webp` 是透明源图；`grid-384.webp` 是导入后的 1536×1536 图集预览；四个动画 WebP 分别展示普通／觉醒待机及施法；`normal-idle-0.webp` 与 `awake-idle-0.webp` 是静帧预览。提示词见 PROMPT.md。

运行时只打包 `assets/roles/priest/runtime/atlas.webp`。每帧 384×384，16 帧，脚底锚点 (192,336)，每行最大可见高度 320，整个角色共享至多 64 色、二值透明。空白方块无图案；觉醒只在金杖球头增加少量光。检查结果见 `config/role-assets.json`。


---

# priest 十二帧动画

Issue #11 第一阶段实现稿，覆盖新五人；尚不代表用户最终美术验收。沿用既定人设，历史生产版本保留。

- 每个 `*-source-.webp` 是内置 ImageGen 生成的独立 4×3 源图，12 张不同姿势；对应 `*-prompt-.md` 保存实际提示词。
- 动作：normal-idle / awake-idle / normal-cast / awake-cast。运行时每角色 48 帧、384×384，6×8 图集（2304×3072），脚底锚点 (192,336)。
- `*.webp` 动作预览均由运行时同样的量化帧导出。待机一轮 720 ms；施法及入睡 1400 ms；预览为便于比较循环播放，游戏内施法／入睡只播放一次并保持末帧。
- 全角色各自共享至多 64 色、二值透明、最近邻采样、无损 WebP。不是把原四帧重复三次，也没有光流或交叉淡化。
- `report.json` 记录尺寸、色板、源动作统一缩放、帧边界和像素哈希。自动检查不能替代对武器、服装、姿势衔接的视觉审查。
- 保留原人物介绍与能力配置。矮人书本层继续使用 ；火法师两形态共用施法及入睡动作。

复现：用 `.build-tools/python-path.txt` 指定的 Python 安装 `tools/art-requirements.txt` 到 `.build-tools/art-site`，运行 `tools/prepare_character_animations.py`。游戏只打包 `assets/`，本目录源图和预览不进入游戏包。


---

Use case: identity-preserve. Production sprite animation for BattleBlocks. Match the reference's EXACT brown kobold dog-headed cleric, not a human or different animal: long canine muzzle, pointed ears, amber eyes, bushy brown tail, simple blue robe with gold trim and cream stole, EXACTLY TWO brown belt pockets on front. Holds ONE long solid-gold staff with round GOLD BALL top and simple metal rings, never cross/crystal/short wand. No costume redesign. Densely detailed crisp pixel clusters, rich stepped cloth/fur/gold ramps. Exactly 12 genuinely different articulated poses arranged FOUR columns by THREE rows of equal cells, reading left to right then down. 1536x1152 or larger, fixed camera and scale, stable foot baseline, whole character AND staff fit comfortably in each cell; clear transparent gutters. TRUE transparent background with no text/numbers/borders/floor/checkerboard. No duplicate frames, camera moves, whole-image shifts or blur. AWAKENED CAST 12-frame one-shot sleight-of-hand gesture. Same body and clothes, only staff gold ball has tiny warm light. Frames1-3 free hand reaches into front belt pocket;4-6 draws ONE plain cream square block;7-8 presents it forward at chest height;9-10 puts block back into pocket;11-12 relaxes to idle. Long staff stays upright in other hand. Blank cream block NO text, symbols, pips, image or card frame. Both front pockets remain visibly intact.


---

Use case: identity-preserve. Production sprite animation for BattleBlocks. Match the reference's EXACT brown kobold dog-headed cleric, not a human or different animal: long canine muzzle, pointed ears, amber eyes, bushy brown tail, simple blue robe with gold trim and cream stole, EXACTLY TWO brown belt pockets on front. Holds ONE long solid-gold staff with round GOLD BALL top and simple metal rings, never cross/crystal/short wand. No costume redesign. Densely detailed crisp pixel clusters, rich stepped cloth/fur/gold ramps. Exactly 12 genuinely different articulated poses arranged FOUR columns by THREE rows of equal cells, reading left to right then down. 1536x1152 or larger, fixed camera and scale, stable foot baseline, whole character AND staff fit comfortably in each cell; clear transparent gutters. TRUE transparent background with no text/numbers/borders/floor/checkerboard. No duplicate frames, camera moves, whole-image shifts or blur. AWAKENED IDLE 12-pose loop. Only form change is small warm light around staff's gold ball; no body/costume changes, no halo around head. Staff planted upright. Smooth subtle inhale/blink/exhale with ears, free elbow and tail; last frames approach first. Both pockets remain visible.


---

Use case: identity-preserve. Production sprite animation for BattleBlocks. Match the reference's EXACT brown kobold dog-headed cleric, not a human or different animal: long canine muzzle, pointed ears, amber eyes, bushy brown tail, simple blue robe with gold trim and cream stole, EXACTLY TWO brown belt pockets on front. Holds ONE long solid-gold staff with round GOLD BALL top and simple metal rings, never cross/crystal/short wand. No costume redesign. Densely detailed crisp pixel clusters, rich stepped cloth/fur/gold ramps. Exactly 12 genuinely different articulated poses arranged FOUR columns by THREE rows of equal cells, reading left to right then down. 1536x1152 or larger, fixed camera and scale, stable foot baseline, whole character AND staff fit comfortably in each cell; clear transparent gutters. TRUE transparent background with no text/numbers/borders/floor/checkerboard. No duplicate frames, camera moves, whole-image shifts or blur. NORMAL CAST 12-frame one-shot sleight-of-hand gesture, staff unlit solid metal throughout. Frames1-3 free hand reaches into front belt pocket;4-6 smoothly takes out ONE plain cream square block;7-8 holds block forward in palm at chest height;9-10 draws block back into pocket;11-12 returns relaxed to idle. Staff stays in other hand, long and upright. Block MUST be blank solid color: no writing/symbols/pips/picture/card border. Keep both pockets and gold ball shape consistent.


---

Use case: identity-preserve. Production sprite animation for BattleBlocks. Match the reference's EXACT brown kobold dog-headed cleric, not a human or different animal: long canine muzzle, pointed ears, amber eyes, bushy brown tail, simple blue robe with gold trim and cream stole, EXACTLY TWO brown belt pockets on front. Holds ONE long solid-gold staff with round GOLD BALL top and simple metal rings, never cross/crystal/short wand. No costume redesign. Densely detailed crisp pixel clusters, rich stepped cloth/fur/gold ramps. Exactly 12 genuinely different articulated poses arranged FOUR columns by THREE rows of equal cells, reading left to right then down. 1536x1152 or larger, fixed camera and scale, stable foot baseline, whole character AND staff fit comfortably in each cell; clear transparent gutters. TRUE transparent background with no text/numbers/borders/floor/checkerboard. No duplicate frames, camera moves, whole-image shifts or blur. NORMAL IDLE 12-pose loop. Gold ball is unlit solid metal. Staff stays upright planted left; free hand relaxed by front pocket. Frames1-3 subtle inhale,4-6 blink and slight ear/tail/elbow movement,7-9 exhale,10-12 ease back near frame1. Feet stay planted, robe folds and tail lag smoothly. Preserve both pockets visibly on front in EVERY pose.
