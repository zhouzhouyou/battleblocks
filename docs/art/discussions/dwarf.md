# 制作讨论与提示词

以下文字记录设计讨论与制作过程；当前采用资源以角色档案为准。

---

# 矮人 E-A 黑红无盔人设图 ·

日期：2026-10-29。生成方式：内置 ImageGen。用户选择的 E-A 精修为角色基础，用户上传图为露头发型／胡须／气质参考；参考文件仅本地保存、不进入仓库，保留来源性质，不宣称其由本项目原创生成。

## 输出与用途

- `dwarf-model-sheet-black-red-.webp`：本轮人设图，**待用户评审，未自动视为最终通过**。
- 上轮 `dwarf-refined-ka-.png` 已按用户要求从工作树和 Git 历史移除；不作为当前素材。
- `references/user-bareheaded-dwarf-2026-10-29.png`：用户提供原始参考，仅本地保存，不进入仓库。
- 人设包含人物正／侧／背视图，武器正／侧／俯视图，平静／开心／生气头像，以及肩甲、黑色围裙、红色内衣与腰布、腰带、护腕、靴子、胡须束环、独立仇恨之书的装备拆解。
- 武器重新设计为双端对称精锻钢战锤，中央套筒、双端打击面、成对结构件与握柄轴线对齐。符文是横向矩形锤头＋竖直中轴锤柄的几何图记，普通暗刻／觉醒原刻痕发红光。
- 用户已要求用本图重制动画，当前  评审媒体已完成；没有导入游戏。人设图的画布、字标、透视和色数不等于生产 spritesheet 已验收；锤子的对称以正视设计结构为准，后续像素图仍需逐格校准。
- 黑红与无头盔覆盖历史造型，但被动、主动、仇恨之书和介绍维持档案已有用户设定。人物与书仍为独立素材方向，不绑定。

## 用户本轮原文

> 先回到矮人的人设，创建矮人的人设图。你现在 E-A 精修 的 锤子显然存在问题，不够对称了。人设图包括角色的所有装备，人物和武器的三视图，人物不同心情的面貌（平静，开心，生气）。从 E-A 精修出发，我希望你做以下修改：1. 角色的锤子得是个对称的设计 2. 锤子上的符文得重新设计，可以是简单的锤子样式的符文 3. 锤子需要看上去质量很好。4. 参考我给的图，角色不要带头盔了。5. 角色服装使用黑色和红色。在完成这些后给出人设图。

## 完整生成提示词

```text
Use case: stylized-concept / original game CHARACTER MODEL SHEET.
Create ONE LARGE LANDSCAPE, extremely clear professional CHARACTER DESIGN SHEET for BattleBlocks dwarf E-A. User specifically requests COMPLETE CHARACTER AND EQUIPMENT reference, not a new action illustration.

INPUT ROLES:
Image1 is existing "矮人E-A精修": use for original identity, male adult stout dwarf, overall E-style pixel refinement, orange beard and practical forge-warrior outfit. Its helmet, blue clothes, forked rune and asymmetric hammer are REJECTED and MUST CHANGE.
Image2 is USER REFERENCE: use for BARE HEAD with swept-back reddish-orange hair, rugged expressive face, large orange mustache and beard tied in chunky braids, proud dwarf presence. Do NOT copy blue clothes, helmet, moss, complex old runes or asymmetric hammer. No assumptions about image authorship. This is an original redesign.

MUST HAVE ALL FOUR SECTIONS, clearly organized, no missing views:
1) CHARACTER TURNAROUND across upper left ~60%: THREE FULL BODY ORTHOGRAPHIC views, exactly FRONT / SIDE / BACK. Labels "正面" "侧面" "背面". All same person, same BLACK AND RED clothing and identical proportions, ground line and top of head aligned, complete boots visible. Relaxed neutral A-pose arms slightly away from torso, EMPTY OPEN hands, NOT holding hammer so entire costume visible. Side means TRUE90degree profile, back TRUE180degree. No front-three-quarter substituted for side. Turnaround consistency more important than dramatic posing.
2) WEAPON TURNAROUND upper right ~40%: SAME NEW PREMIUM SYMMETRICAL WARHAMMER, THREE clear engineering-like ORTHOGRAPHIC views, labels "锤·正视" "锤·侧视" "锤·俯视". Front view large enough to inspect exact mirror symmetry. Top view shows rectangular head depth, side shows central socket, all consistent with front.
3) EXPRESSIONS lower left: THREE LARGE portrait heads of SAME bareheaded dwarf, same head angle/face/hair/beard in every portrait, "平静" relaxed closed mouth focused eyes; "开心" broad genuine grin lifted cheeks and eyebrows; "生气" deep lowered brows, narrowed eyes, clenched mouth. Different expressions, not3different people, NO helmet on any. Don't hide eyes with eyebrows.
4) EQUIPMENT BREAKDOWN lower center/right: neatly separated ALL worn items, no occlusion: pair of steel shoulder caps with simple brass edge; charcoal smith apron with broad front pocket; dark red tunic and split waist tabard; blackened leather belt with round brass buckle and simple straps; pair of black leather bracers; pair of heavy black boots with gray toe caps; small brass beard ties. Also ONE separate closed thick dark-red GRUDGE BOOK as an independent accessory, no dwarf standing on book, no permanent attachment to body. Show no helmet item. Include two small RUNE closeups labeled "暗刻" and "觉醒": exact SAME SIMPLE HAMMER-SHAPED RUNE, dark non-glowing before awakening, crisp RED light after awakening. No other new props.

DESIGN LOCK:
Middle-aged male forge dwarf, HALF normal human knight body height in-game at equal pixel scale; very short legs, broad sturdy torso, muscular bare forearms, large hands but mature adult joints and face. Not a tall human, not toddler/chibi toy. BARE HEAD, visible swept-back orange hair, no helmet, no headband, no crown, no mohawk or war plume. Original rugged orange beard identity, chunky braids with simple brass ties. Outfit MAINLY CHARCOAL BLACK and DEEP CRIMSON RED, not blue or green. Black leather smith apron and boots/bracers, red tunic and waist cloth, gray iron shoulders, limited structural bronze/brass fittings. Practical worn-but-well-kept master smith clothing, E-A ancestry. No added huge ornate plate armor or flowing royal cape. Clothing front/back construction matches: apron cross straps visible on back, red cloth visible under apron. Keep freehand unmistakably empty in every body view, nothing crossing palm, no pocket tool beside fingers.

WEAPON IS MOST IMPORTANT:
A high-quality masterwork RECTANGULAR DOUBLE-FACED WARHAMMER. Front-view hammerhead a perfectly LEFT-RIGHT MIRROR SYMMETRIC shape centered on a straight vertical shaft. Both ends identical flat striking faces, equal end-cap length, mass and bevel, no picks, blades, spikes or longer side. One clean polished forged-steel head, identical silver bevels/end faces, a precisely centered solid central socket and restrained equally placed bronze structural bands. NOT lopsided, bent, improvised, cracked, rusty, mossy, chipped or junk. Quality through true construction, machining-like precision and controlled highlights, not ornamental excess. Handle centered below head, straight, tightly wrapped dark-red leather grip, compact symmetrical pommel. Same geometry across all3views.
Rune in center of broad front face is a SIMPLE HAMMER PICTOGRAM: ONE SMALL HORIZONTAL RECTANGULAR BAR at top + ONE STRAIGHT VERTICAL HANDLE centered underneath, T-shaped hammer silhouette, maybe one tiny rectangular grip end. EXACTLY MIRROR SYMMETRIC. NOT branching trident/Y, not antlers, not complex magic writing, not the old symbol. Rune ENGRAVED recessed dark charcoal in normal views. The exact same pictogram lights RED in awakening inset. No damage cracks radiating through hammerhead, no whole-head flames or softened bloom. Same hammer shape and armor in awakening, only rune light changes.
Weapon orthographic views must make symmetry visually undeniable, equal ends and socket dead center. No oblique perspective in the hammer front view. Very faint centerline behind front view allowed.

ART E QUALITY:
Densely crafted pixel illustration using deliberate square clusters, nuanced hue-shifted3–4 stepped material ramps, crisp pixel contours, purposeful rich detail around face/beard/forged metal, quiet broad black/red cloth planes. Match reference E-A/K-A level of refinement, NOT8bit crude blocks, NOT smooth painted fantasy art or vector illustration. Enlarge consistent pixel grid across views; portraits may be enlarged crops stylistically but same person. No random pixel noise, dither grime, tiny unreadable engraving, smooth gradients, antialiasing or glossy3D. Neutral flat dark charcoal background, thin quiet section dividers, clear warm ivory readable Chinese labels. No scenery, no floor shadows that hide boots, no pseudo-dashboard, no stats or speculative skills. Title exactly "矮人 E-A · 人设图". Large landscape canvas preferably3072x2048or equivalent, enough space so ALL required sections remain legible simultaneously. No cropped figures/gear. This is an original production reference concept sheet, not final animation sprites.
```


---

> 历史记录：对应旧稿已按用户要求从工作树和 Git 历史移除。当前只使用  人设图与 当前装备制作目录，不要据本文恢复旧素材。

# 矮人战士 E 版 · 生成记录

2026-10-28 用户要求按 E 方案重新比较矮人四稿。内置 ImageGen 生成，样式参考 E 全角色概念图；本轮尚未选定角色稿，不覆盖 F 版历史素材。

输出：`dwarf-review-e-.png`。图中上排普通、下排觉醒，左至右 E-A／B／C／D。此图用于造型评审，未验证生产用网格或色数。矿道先锋的武器偏双尖矿镐，定稿时需依最终武器意象修订。

## 本轮提示词

```text
Use case: stylized-concept.
Create one ORIGINAL western fantasy dwarf warrior design review sheet for BattleBlocks. The attached image is a STYLE REFERENCE ONLY: match its E art direction, deliberate PIXEL CLUSTERS, hue-shifted stepped material ramps and expressive compact character sprites. Do not copy the whole roster or draw other species. Generate FOUR different DWARF designs each with ordinary and awakened form, EIGHT full-body sprites total.

Layout: large landscape sheet with FOUR equally wide columns, TWO character rows. Top title exactly "矮人战士 · E 版四稿". Each column top has readable code and name: "E-A 锻炉老兵", "E-B 氏族卫士", "E-C 矿道先锋", "E-D 石誓行者". First row label "普通", second row label "觉醒". Same character in each column, before/after. One tiny sublabel per column under name: "男 · 中年", "女 · 成年", "女 · 成年", "男 · 老年". Neutral flat dark charcoal backdrop with thin quiet grid lines. No scene, decorative frames, stats, portraits, plinths. Full bodies, hats, weapons, cloaks all visible with clear spacing. Each figure shown at SAME logical pixel scale. Enough room so awakened equipment isn't cropped.

E ART BIBLE: sprites purpose-designed around 32x48 logical pixels ordinary and up to48x64 awakened; visible discrete enlarged square pixels, actual carefully shaped chunky clusters, NOT soft painted art pixelated at edges. 3-head-tall dwarf style with distinctly SHORT WIDE sturdy bodies. 10–14 local colors, broad quiet garment areas, 3–4 stepped tones for focal iron/skin/wood/cloth, muted hues and limited bright accents. Upper-left light. Pixel faces about7x7, expressive brow and eye clusters. Cloth has broad folds, metal short sharp ivory highlights, hair grouped masses NOT individual strands. Plain equipment except focal feature. No microdetail, dithering, noise, antialiasing, blurry scaling, glossy gradients or cinematic soft light. Match the style reference's knight/goblin/dwarf density, not high resolution painted concepts. Awakening adds one silhouette feature and strengthens signature weapon while keeping face, palette, costume ancestry and pixel density. Effects restrained, no clouds of particles.

FOUR DESIGNS, all melee DWARF WARRIORS; gender does not make women tall/slim. Both female designs have broad dwarf shoulders, thick boots and powerful bodies, adult faces, practical armor, no sexualization.

COLUMN E-A male FORGE VETERAN: orange block-shaped beard with one braid, weathered tan face, gray iron half-helmet, bare strong forearms, heavy brown smith leather apron, blue waist sash. Low forward center of gravity, gruff reliable demeanor. Carries SHORT-HAFT SQUARE IRON WARHAMMER with very obvious rectangular hammer head, TWO FLAT STRIKING FACES. NOT an axe, not a cutting blade, no pointed edge. Awakened: forge champion, broad angular soot-gray shoulders, orange helmet crest, wider rectangular forge hammer with ONE ember-red seam, leather apron and beard identity retained. Dominant iron/orange/leather.

COLUMN E-B female CLAN GUARDIAN: determined mature woman dwarf, silver hair in TWO THICK BRAIDS, broad sturdy face and body, no beard. Dark BLUE shoulder cloth, angular gray plate torso, simple clan belt, wide planted straight posture. LONG-HAFT TWO-HANDED RECTANGULAR MALLET upright beside her, hammer head a thick block with two FLAT opposing faces, no axe curves or cutting point. Awakened: clan matriarch champion, banner-shaped blue shoulder mantle, broad plain gold headband, bigger squared mallet. Preserve braids and sturdy proportions. Dominant gray/blue/ivory with limited gold; not a slender human princess.

COLUMN E-C female TUNNEL VANGUARD: mature dwarf woman, copper SHORT BOB distinct from B, round iron MINING HELMET with small ivory lamp tile (no modern electric headlamp aesthetics), brown practical leather tunic, burgundy short cape, square expedition backpack, low crouched ready posture, lively fearless face. PICK-HAMMER with one blunt FLAT hammer striking face and ONE clear mining pick on opposite side, no crescent axe. Awakened: reinforced iron mining faceguard WITH EYES AND COPPER BOB STILL VISIBLE, stronger backpack outline, larger wedge pick-hammer, same burgundy scarf and copper hair. Emphasize scout/expedition personality, not generic gray heavy knight. No mechanical robot armor.

COLUMN E-D elderly male STONE-OATH WANDERER: broad mature square IVORY beard, moss GREEN cloth HOOD (not pointy wizard hat), large moss travel cape over simple brown heavy tunic, wide boots, calm grounded stance. Massive crude STONE BLOCK HAMMER, rectangular gray stone on plain wood shaft, heavy two flat ends, NO AXE BLADE. Awakened: ancient oath sentinel, TWO broad stone shoulder slabs and a gate-post-shaped giant stone mallet, green cape and white beard clearly retained. Large geometric stone planes with ONE old carved notch, not ornate tiny runes. Not a wizard: fighter stance, no spell staff, no floating glyphs.

Keep four ordinary designs obviously distinct through helmet versus braids versus mining pack versus green hood, and FOUR awakenings preserve those differences. DO NOT turn all four awakenings into identical golden plate armor. Clearly Hammers, not four battle axes. Match reference pixel craft and cluster density. Only E-A/E-B/E-C/E-D labeling, no F or8-bit-minimal labels. Original designs and compositions.
```


---

> 历史记录：对应旧稿已按用户要求从工作树和 Git 历史移除。当前只使用  人设图与 当前装备制作目录，不要据本文恢复旧素材。

# 矮人 E-A 按 K-A 精细度重绘 · （历史）

输出 `dwarf-refined-ka-.png`，内置 ImageGen。2026-10-29 用户要求改为无头盔、黑红服装、对称锤及锤形符文后，本图作为历史基础，不是当前最新外观。

```text
历史生成记录：对应参考图与用户要求见角色档案。
```


---

> 历史记录：对应旧稿已按用户要求从工作树和 Git 历史移除。当前只使用  人设图与 当前装备制作目录，不要据本文恢复旧素材。

# 矮人战士首轮评审：生成记录


全局提示词与制作约束：ART-DIRECTION.md。本轮使用下列资产级提示词收简第一轮过密的细节。原始最终预览保存在 `dwarf-review-.png`；`dwarf-review-.webp` 是像素一致的无损副本；`dwarf-review-ui-preview.webp` 是仅供界面预览的压缩副本。未采纳的第一轮过密图不作为最终交付素材。

## 最终资产级提示词

```text
Create a NEW image using the attached F character sheet ONLY as a style reference. Not an edit of the sheet. Need dramatically SIMPLE actual tiny 8-bit pixel sprite designs, no fine detail. The dwarf sprites must be NO MORE DETAILED than the reference goblin and tiny tree spirit. Do not make detailed large dwarf portraits.

Output landscape 1024x768 preferred. PLAIN absolutely FLAT SOLID background #171a25 with no gradient/no noise. Exact4 columns x2rows, equally spaced 8 cells. NO TEXT, no lines, no labels. ONE full-body dwarven character centered in each cell, generous padding. Top row normal, bottom awakened, same person per column. Actual design looks constructed from 16x24 giant square logical pixels. Head is 5x5 pixels and eyes each ONE pixel. Character ONLY 5 or6 colors. All bodies short wide two-head-tall DWARVES. NO shaded armor panels, hair strands, engraved hammer patterns, metallic textures, knee pads or jewelry. One flat fill per garment. No material has >2 shades. Huge noticeable square pixels, rough simple compact roguelike sprites. Use about 150-250 colored pixels per sprite. No smooth or soft anything.

Column1 top: MALE forge veteran dwarf, 3x3 orange beard, gray block helmet, brown leather square apron, bare tan arms, ONE simple darkgray T-hammer. Bottom: same male with TWO broad gray shoulder blocks, blue waist sash, larger T-hammer with ONE orange edge. No layered armor.
Column2 top: FEMALE clan guardian dwarf, silver hair TWO chunky braids, tan square face no beard, wide short blue square tunic with gray shoulders, plain long wooden haft and gray RECTANGULAR hammer head. Bottom: same silver-braided woman, wider blue cloak and ONE gold headband pixel stripe, wider rectangular hammer head. No gold filigree or detailed armor.
Column3 top: FEMALE tunnel scout dwarf, orange bob hair no beard, simple gray mining helmet with ONE ivory square lamp, brown tunic, short RED scarf, square brown backpack, simple iron pickhammer, low crouch. Bottom: same woman with gray squared faceguard but orange bob visible, larger square pack and broad wedge pickhammer. No elaborate clothing.
Column4 top: OLD MALE stone-oath dwarf, white 3x3 beard, GREEN stepped hood and big GREEN cloak, brown body, plain gray block-stone hammer. Bottom: same man with TWO simple stonegray shoulder blocks, wider green cloak and giant gray rectangular hammer head. No metal plates all over.

All 8 figures at the exact same logical pixel scale, all full bodies and tools visible. Awakened adds ONLY silhouette and one accent, not texture. Minimal 8-bit, visible huge square color blocks, heavily simplified. Do not render many strands of beard or complex fingers. Plain icon-like sprites, low resolution, game production spirit. No background effects. Uniform grid is needed for displaying cells individually in review UI. No copying actual characters from another game.
```

## 已知评审限制

生成图没有严格满足所有像素网格、色数、纯色背景与锤头形状指令。当前只评人物方向、性别、轮廓、打扮与觉醒连续性；选定后需修正武器并按制作规范产出正式资产。


---

> 历史记录：对应旧稿已按用户要求从工作树和 Git 历史移除。当前只使用  人设图与 当前装备制作目录，不要据本文恢复旧素材。

# 矮人修订图与仇恨之书动画 ·

日期：2026-10-28。人物与书原画由内置 ImageGen 生成；动画预览用 FFmpeg 进行媒体合成，不修改游戏代码，不实现技能或战斗规则。

## 独立素材与交付

- `dwarf-normal-awake-corrected-.png`：普通／觉醒两态透明修订图；空手张开、离开腰带，只有另一只手拿锤。身体与装备一致，锤上刻痕普通暗色、觉醒红色。当前待用户确认，不自动视为定稿。
- `grudge-book-thickness-.png`：独立透明书设计，三个厚度方向；每格只有一本书。既不是三个叠放的书，也不含角色。
- `grudge-book-independent-.gif`：**不含人物**的独立增厚动画评审预览；640×300、16fps、144帧、9秒。
- `dwarf-grudge-book-preview-.gif`：人物与书的效果合成评审预览；640×580、16fps、144帧、9秒。此 GIF 不用于作为绑定的正式角色素材。
- `preview-stage-one.png`、`preview-stage-three.png`：媒体导出后的检查快照，不是新增选稿。

GIF 为便于查看而循环播放。实际游戏中的增厚／变薄是血量跨阈值触发的一次性事件，不能将评审循环当成技能循环。两份 GIF 有中性展示底色；可替换的透明原画分别保留，不能以 GIF 替代生产资产。

## 当前动画设计

同一本厚书从侧面垫入→一级厚度停留→二级停留→三级停留→回血减为二级→一级→整本移出。全过程不打开、不写字、不增加第二本书。增厚主要改变纸页区域，上／下封皮保持自身尺寸；角色脚底随书面升降，身体外形、护甲和头发不变。

本次媒体合成把当前选用书图分成上部、纸页段、底部，只有纸页段随时间改变高度，顶面随之移动，人物是另外叠加的独立输入。输入图的具体裁切只服务本次预览，不是未来游戏的固定接口；替换人物时保留其脚底锚点与书面高度对齐即可，无需重做独立书动画。

当前人物裁切／书裁切均为展示用途。身体相对骑士的 1:2 比例是已记录的正式设计要求，本次不含骑士并排比较，不能声称已完成生产比例验收。正式动画仍需按像素网格、alpha边缘与相同脚底锚点校准。

## 已弃用

- `dwarf-idle-runes-.png`：非持锤手仍有不明握柄，未通过，不能继续当作当前角色贴图。
- `grudge-book-study-.png`：开合方案废弃。
- `grudge-book-closed-.png`：太薄，废弃。
- 多本叠放：已被同一本三级增厚覆盖。

## 修订人物提示词

```text
EDIT the supplied dwarf animation sheet into a clean TWO-FRAME corrected design sheet, ordinary left and awakened right, transparent background. One adult male dwarf only, two states of exactly same person. Reference E style, preserve face, orange beard, helmet WITHOUT a crest, shoulders, apron, blue waist cloth, boots, stout short body and iron rune hammer. Each cell square with generous transparent padding, image twice as wide as high. Feet aligned, all equipment within frame.

MOST IMPORTANT CORRECTION: the viewer-LEFT FREE HAND MUST BE CLEARLY EMPTY. REMOVE the small gray horizontal dagger/rod/handle passing from the palm toward the belt. REMOVE that item entirely, not just recolor. REMOVE the nearby belt-mounted tool, dagger handle, pocket hook and any gray horizontal projection from that side of his waist. Change the free arm pose: elbow slightly out, arm hangs away from his torso with a WIDE TRANSPARENT GAP between hand and waist. The empty palm faces the viewer with FOUR short visible orange/tan skin finger shapes and one thumb, fingers relaxed pointing DOWN. There is NO object in the hand, beside the fingers or crossing palm. Clearly OPEN EMPTY HAND, not a fist around invisible handle. His belt must be plain unbroken brown leather on that side, NOTHING projecting laterally. Only one weapon in entire picture: HAMMER held by other hand viewer-RIGHT.

Keep ordinary(left) and awakened(right) bodies EXACTLY IDENTICAL: no new armor, bigger pauldrons, new hair, crest, feathers, crown, cape, horns, book or muscle size change. The hammer is exactly the SAME short wooden shaft and rectangular iron block, two FLAT striking ends, not axe. Three connected angular carved strokes on its visible broad face: ordinary dark gray non-glowing, awakened SAME carving red with tiny warm red/cream core and HARD PIXEL EDGES. Local red rune ONLY, not hammer flames or body glow.

Pixel cluster art E, visibly deliberate enlarged squares, clear material steps, readable at game sprite scale, limited texture on apron/armor, warm skin, orange grouped beard, subdued blue cloth, upper-left light. NO smooth digital fantasy painting. Dwarf body is HALF normal knight height at same pixel scale, very wide and squat, not tall. Need only TWO static corrected poses for review, normal and awake, no labels, no extra characters, no opening books, genuine transparent background. Both free hands unquestionably empty and separated from torso.
```

## 当前厚书提示词

```text
ONE original pixel-art PROP ANIMATION KEYFRAME STRIP for western fantasy game BattleBlocks.
Exactly THREE equal square transparent cells in one horizontal row, no text or labels. In each cell the SAME CLOSED BOOK OF GRUDGES at THREE increasingly THICK states. This is ONE BOOK GETTING THICKER, NOT stacks of multiple books. Stage1 already a very THICK leather tome. Stage2 same book with paper/page block twice as thick. Stage3 same book with page block three times as thick. SAME dark red-brown leather top cover, SAME bottom cover, SAME simple bronze corner blocks, same tiny central lock in each. Exactly TWO covers on each book, ONLY ONE book per cell, no repeated layered covers or horizontal leather lines across middle pages. Increase the ivory page mass between covers, not the covers' thickness or number. A few broad page-edge bands, not hundreds of fine stripes.

Consistent low front-three-quarter perspective (mostly view spine with some flat TOP visible), equal book width in all3cells and same pixel scale, bottoms aligned at same baseline in all cells. Book width about64logicalpixels; depth top plane12pixels; page block height stage1:12px, stage2:24px, stage3:36px. Stage3 enormous brick-thick grudge tome, not a thin notebook. No open pages, no magic writing animation, no hands, no character, no platform, no aura, no shadow background. Flat strong top surface to stand dwarf feet on. Flat rectangular cover, not arched chest lid. Book stays CLOSED with parallel covers at all stages.
Colors dark burgundy/old brown leather, warm ivory page edges, iron-bronze corner hardware. One tiny angular embossed rune on SPINE so identity clear; not random readable lettering, no cross or skull. Minimal symbolic detail. Enough separation so no cell overlaps. Transparent background true alpha.

Art E: clearly visible square pixel clusters, 3-4 stepped material tones, hard outlines, warm upper-left light. Not a smooth high-resolution painting. Same resolution/density across3states, no gradients, antialiasing, tiny texture or noise. Pixel-art game asset, THREE thick states of the same closed book. Not3different designs. Overall output landscape3:1; each full book fully visible in square cell with generous transparent padding.
```


---

# 仇恨之书垫高动画 · （当前）

日期：2026-10-29。依据 `../../dwarf-model-sheet-black-red-.webp` 的黑红无盔矮人和**装备拆分中的仇恨之书**重制。最新用户原话：「记得移除之前的稿件，重新制作仇恨之书垫高角色动画。仇恨之书采用你刚才生成的人设图中的设计。」「人设图的装备拆分里面有仇恨之书的图案啊，按那个来」。

## 当前文件

- `dwarf-awake-source.webp`：内置 ImageGen 根据  人设图生成的独立人物源图，黑红无盔、三束橙胡须、空手、对称精良战锤、红色锤形符文。不含书。
- `dwarf-awake.webp`：用于本轮媒体合成的透明人物层，350×326。只整理透明边缘、边界与展示尺寸，未将书合入。
- `grudge-book-model-crop.webp`：**直接裁自人设图**的原始书图，180×184；参考裁切范围为人设图 `(1345,620,180,184)`。保留原设计，无重新绘制封面。
- `book-cover.webp`：从该原图封面做透视整理所得透明封皮层，280×112。金色边框、包角、铆钉、锤形标记均来自原图像素，不采用另行生成的大号 T 书封面。
- `book-pages.webp`：从同一书图的纸页区域裁出，280×40；动画只扩展这一层的高度。
- `book-bottom.webp`：同一书图底部封皮层，280×12。
- `book-growth-independent.gif`：透明背景、**无人物**的书增厚／变薄动画，550×300、16fps、160帧、10秒；可独立替换或配其他人物。
- `dwarf-book-elevation.gif`：最新人物＋上述书层的合成评审预览，720×680、16fps、160帧、10秒；不是将书绑定到人物的正式贴图。

## 动画行为

仅一本闭合的书：从侧面进入脚下，一级厚度停留→二级→三级→随回血变薄到二级→一级→收回。没有开合、写字、多本叠放、人物长高／新增护甲或头发。封面／封底本身不随纸页增厚拉伸，人物保持相同图像与尺度，脚底跟随书面高度。

本轮展示尺寸：纸页高度 40／60／80px，封面112px、底封12px。书底保持同一地面，承托面位于封面内距后边72px处。角色脚底位置从该承托面推导；这些是当前展示媒体的几何参数，不是绑定未来人物图片的游戏接口。替换人物只需要调整其脚底锚点，不需要重做书的独立动画。

Gif 循环仅便于评审；正式游戏事件设计仍为血量跨阈值触发的一次性过渡。本轮未编写游戏代码、修改技能、数值或资源包。

## 核对

- 合成与独立 GIF 均160帧、10000ms。
- 已查看一级与三级合成快照：角色站在书封面上，只有纸页与角色高度变化。
- 独立 GIF 背景透明；开始与结束 alpha 边界为空，收回无残影，透明帧 disposal=2。
- 实际身体相对骑士 1:2 的生产比例仍需统一标尺校准；当前只有矮人，不声称已经完成跨角色尺寸验收。
- 书封面源自小尺寸人设图，放大后像素与纸页纹理较粗；这是直接复用原图的表现，没有擅自重新设计其图案。

## 清理

旧蓝衣／头盔／分叉符文矮人图、旧多本／开合／其他书样式和旧 GIF 已按用户要求从当前工作树移除。此前已提交的历史原图已从 Git 历史移除，不再从生成缓存重新复制到当前资产目录。历史提示词文档仅保留用于追溯，不表示对应旧图片仍为有效资产。

## 当前人物源图生成提示词

```text
Use case: precise-object-edit / character asset extraction.
SOURCE: the supplied completed "矮人E-A人设图" black-red BAREHEADED dwarf model sheet. This is the only current design. Create ONE isolated FULL BODY awakened dwarf on transparent background, no book, floor, shadow, labels, grid or other figures.

Copy the SAME middle-aged male character from the LEFT FRONT turnaround: swept-back orange hair, no helmet or crown, rugged face, blue eyes, three orange beard braids tied with brass rings, short very broad adult dwarf body, gray steel shoulder caps edged brass, charcoal BLACK leather smith apron, dark RED tunic and tabard, dark bracers, wide belt and gray-capped heavy boots. Preserve EXACT outfit, proportions, face, hair, three braids, palette and refined E pixel-cluster style. Do not add more armor, hair, spikes, cape or new equipment. Slight front-three-quarter standing pose, legs firmly planted, boots aligned on same level. BODY height is HALF human knight in game. Leave generous transparent border (at least10% per side), all hair/feet/weapon visible. Aim centered in landscape or square image.

His viewer-left FREE hand hangs clearly OPEN and EMPTY away from belt, no tool, rod, dagger handle or ambiguous buckle touching palm. Viewer-right hand holds the EXACT warhammer from source weapon front view upright beside body: high-quality RECTANGULAR hammerhead with TWO IDENTICAL MIRRORED flat striking ends, equal mass and length on both sides of centered shaft, polished steel with matching bronze structural bands, straight dark-red-wrapped handle and compact bronze pommel. No picks, blades, lopsided head, cracks, rust, moss or random extra ornaments. Same centered simple HAMMER-SHAPED T PICTOGRAM engraved on broad hammer face (a horizontal rectangular hammerhead + central vertical handle), NOT Y or trident. It glows controlled RED because this is awakened stance. Keep carving crisp, not giant flames or whole body light.

This asset will be MOVED UPWARD by a separately layered thick book in a review animation; do NOT include any book or footstool and do not change body height. Neutral steady idle pose, NO action swing. Character's feet must have a clear flat baseline for anchoring.

Art direction E: visibly discrete squared pixel clusters, nuanced3–4stepped material ramps, source model sheet level of refinement, expressive readable face and angular hair clusters, broad quiet leather and red cloth planes, crisp contours, upper-left light. Not smooth glossy3D, not painted illustration blurred to pixels, no gradients or dither noise. TRUE transparent alpha outside figure. Single full-body dwarf only.
```

书的最终封面、纸页和底封直接来自  人设图，采用媒体裁切／透视整理与分层合成。中途另画的书图没有采用，已清理，不作为本轮素材。


---

> 历史记录：这一版动画图片已按 2026-11-01 清理移出仓库，本目录只保留提示词；当前使用 当前制作源。

# dwarf · 游戏动画

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
Character: short stocky mature MALE DWARF, bright orange swept hair and huge braided orange beard, bare head NO HELMET, black leather smith apron over deep red tunic, dark steel shoulder plates and boots, brass trim. One hand naturally EMPTY. Other hand always holds ONE finely forged double-ended perfectly symmetric rectangular STEEL WARHAMMER with red leather grip and a simple HAMMER-SHAPED rune. BOTH ends same dimensions and material, not an axe. Normal rune dark engraved; awakened rune alone glows red, same body/armor/hair/weapon. No added armor/hair/halo. Row3 swings this SAME hammer with planted then lightly jumping pose. NO BOOK in any frame; book will be a separate layer. At the shared scale the dwarf body is 32 logical pixels tall, HALF the height of a normal human (64). Keep the dwarf low centered in each cell, feet at logical y=84, body top about y=52; full hammer may extend higher. Do not fill the entire cell with a human-height dwarf.

## 动画与一致性修订

Edit this transparent 4x4 dwarf animation atlas. Preserve rows1,2,3 exactly. Replace ONLY bottom ROW4 with FOUR sequential AWAKENED ATTACK poses matching row3: anticipation, overhead windup, powerful hammer swing, recovery. Same exact body, hair, beard, armor and clothing. Only difference from row3 normal attack: engraved HAMMER-shaped rune on same symmetrical double-headed warhammer glows red. Remove broad golden slash effect; keep tight red rune light, motion conveyed by poses. No books, no helmet, no new equipment or hair. Same cell sizes, alpha transparency, feet anchors and pixel density, no text/grid/background.



---

> 历史记录：这一版动画图片已按 2026-11-01 清理移出仓库，本目录只保留提示词；当前使用 当前制作源。

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

# dwarf 十二帧动画

Issue #11 第一阶段实现稿，覆盖新五人；尚不代表用户最终美术验收。沿用既定人设，历史生产版本保留。

- 每个 `*-source-.webp` 是内置 ImageGen 生成的独立 4×3 源图，12 张不同姿势；对应 `*-prompt-.md` 保存实际提示词。
- 动作：normal-idle / awake-idle / normal-cast / awake-cast。运行时每角色 48 帧、384×384，6×8 图集（2304×3072），脚底锚点 (192,336)。
- `*.webp` 动作预览均由运行时同样的量化帧导出。待机一轮 720 ms；施法及入睡 1400 ms；预览为便于比较循环播放，游戏内施法／入睡只播放一次并保持末帧。
- 全角色各自共享至多 64 色、二值透明、最近邻采样、无损 WebP。不是把原四帧重复三次，也没有光流或交叉淡化。
- `report.json` 记录尺寸、色板、源动作统一缩放、帧边界和像素哈希。自动检查不能替代对武器、服装、姿势衔接的视觉审查。
- 保留原人物介绍与能力配置。矮人书本层继续使用 ；火法师两形态共用施法及入睡动作。

复现：用 `.build-tools/python-path.txt` 指定的 Python 安装 `tools/art-requirements.txt` 到 `.build-tools/art-site`，运行 `tools/prepare_character_animations.py`。游戏只打包 `assets/`，本目录源图和预览不进入游戏包。


---

Use case: identity-preserve. Create a production 12-frame sprite animation sheet for BattleBlocks in the EXACT densely detailed crisp pixel-cluster style of the reference. Exactly FOUR columns and THREE rows, 12 equally sized cells, read left-to-right then down. True transparent background, no text/labels/borders/checkerboard/floor/shadows. 1536x1152 or larger, rich stepped material colors, no blurry painting. Fixed camera/scale, stable ground baseline, generous empty margins, entire figure and hammer inside each cell. Same stocky red-haired red-bearded male dwarf with NO helmet, black and red clothing/armor, bare upper arms, short heavy boots, braided beard, and ONE finely made DOUBLE-ENDED SYMMETRIC rectangular warhammer: matching flat ends, gold bands, hammer-shaped engraved rune. Keep body, clothes, hair, shoulder guards and proportions IDENTICAL to reference; no new armor, no new hair, no axe/spikes, no book under feet (book is separate runtime layer). Each image must be a distinct articulated intermediate pose, not repeats or whole-image translation. AWAKENED HAMMER ACTION. ONLY form difference is glowing RED hammer-shaped rune on same rigid symmetrical hammer, NO body change. Smooth 12-frame action:1-3 crouch and ready hammer;4-6 lift hammer over shoulder;7-8 powerful strike/downward landing stance;9-12 recover upright with hammer vertical at side. Boots at consistent baseline (vertical jump is handled by game). No book or scenery. No new armor or hair.


---

Use case: identity-preserve. Create a production 12-frame sprite animation sheet for BattleBlocks in the EXACT densely detailed crisp pixel-cluster style of the reference. Exactly FOUR columns and THREE rows, 12 equally sized cells, read left-to-right then down. True transparent background, no text/labels/borders/checkerboard/floor/shadows. 1536x1152 or larger, rich stepped material colors, no blurry painting. Fixed camera/scale, stable ground baseline, generous empty margins, entire figure and hammer inside each cell. Same stocky red-haired red-bearded male dwarf with NO helmet, black and red clothing/armor, bare upper arms, short heavy boots, braided beard, and ONE finely made DOUBLE-ENDED SYMMETRIC rectangular warhammer: matching flat ends, gold bands, hammer-shaped engraved rune. Keep body, clothes, hair, shoulder guards and proportions IDENTICAL to reference; no new armor, no new hair, no axe/spikes, no book under feet (book is separate runtime layer). Each image must be a distinct articulated intermediate pose, not repeats or whole-image translation. AWAKENED IDLE, ONLY change from normal form is red light INSIDE hammer-shaped rune. No glow anywhere on body. Hold hammer upright at side. Smooth restrained 12-pose breathing loop with shoulders/chest, beard and elbow moving gently and boots planted. Frames10-12 converge toward frame1. No new armor, no attack, rune stays hammer-shaped and glows.


---

Use case: identity-preserve. Create a production 12-frame sprite animation sheet for BattleBlocks in the EXACT densely detailed crisp pixel-cluster style of the reference. Exactly FOUR columns and THREE rows, 12 equally sized cells, read left-to-right then down. True transparent background, no text/labels/borders/checkerboard/floor/shadows. 1536x1152 or larger, rich stepped material colors, no blurry painting. Fixed camera/scale, stable ground baseline, generous empty margins, entire figure and hammer inside each cell. Same stocky red-haired red-bearded male dwarf with NO helmet, black and red clothing/armor, bare upper arms, short heavy boots, braided beard, and ONE finely made DOUBLE-ENDED SYMMETRIC rectangular warhammer: matching flat ends, gold bands, hammer-shaped engraved rune. Keep body, clothes, hair, shoulder guards and proportions IDENTICAL to reference; no new armor, no new hair, no axe/spikes, no book under feet (book is separate runtime layer). Each image must be a distinct articulated intermediate pose, not repeats or whole-image translation. NORMAL HAMMER ACTION, rune stays dark engraving. Smooth 12-frame one-shot:1-3 prepare and bend knees with hammer at side;4-6 lift hammer over shoulder;7-8 strike across/down with small sharp warm arc;9-12 recover and return hammer upright at side. BOTH matching flat hammer ends and one shaft remain rigid, held by same hand. Show anticipation, force and recovery, no added scenery or particles crossing cells. No feet cut off.


---

Use case: identity-preserve. Create a production 12-frame sprite animation sheet for BattleBlocks in the EXACT densely detailed crisp pixel-cluster style of the reference. Exactly FOUR columns and THREE rows, 12 equally sized cells, read left-to-right then down. True transparent background, no text/labels/borders/checkerboard/floor/shadows. 1536x1152 or larger, rich stepped material colors, no blurry painting. Fixed camera/scale, stable ground baseline, generous empty margins, entire figure and hammer inside each cell. Same stocky red-haired red-bearded male dwarf with NO helmet, black and red clothing/armor, bare upper arms, short heavy boots, braided beard, and ONE finely made DOUBLE-ENDED SYMMETRIC rectangular warhammer: matching flat ends, gold bands, hammer-shaped engraved rune. Keep body, clothes, hair, shoulder guards and proportions IDENTICAL to reference; no new armor, no new hair, no axe/spikes, no book under feet (book is separate runtime layer). Each image must be a distinct articulated intermediate pose, not repeats or whole-image translation. NORMAL IDLE, rune is dark UNLIT engraving in ALL cells. Hold hammer upright at side, fist at other side like top reference row. Smooth restrained breathing loop:1-3 breathe in shoulders/chest,4-6 beard and elbow lag,7-9 relax,10-12 return near pose1. Boots planted, no full body scaling, no attack. Preserve broad silhouette.
