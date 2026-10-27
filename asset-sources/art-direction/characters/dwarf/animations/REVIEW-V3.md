# 矮人修订图与仇恨之书动画 · v3

日期：2026-10-28。人物与书原画由内置 ImageGen 生成；动画预览用 FFmpeg 进行媒体合成，不修改游戏代码，不实现技能或战斗规则。

## 独立素材与交付

- `dwarf-normal-awake-corrected-v2.png`：普通／觉醒两态透明修订图；空手张开、离开腰带，只有另一只手拿锤。身体与装备一致，锤上刻痕普通暗色、觉醒红色。当前待用户确认，不自动视为定稿。
- `grudge-book-thickness-v3.png`：独立透明书设计，三个厚度方向；每格只有一本书。既不是三个叠放的书，也不含角色。
- `grudge-book-independent-v3.gif`：**不含人物**的独立增厚动画评审预览；640×300、16fps、144帧、9秒。
- `dwarf-grudge-book-preview-v3.gif`：人物与书的效果合成评审预览；640×580、16fps、144帧、9秒。此 GIF 不用于作为绑定的正式角色素材。
- `preview-stage-one.png`、`preview-stage-three.png`：媒体导出后的检查快照，不是新增选稿。

GIF 为便于查看而循环播放。实际游戏中的增厚／变薄是血量跨阈值触发的一次性事件，不能将评审循环当成技能循环。两份 GIF 有中性展示底色；可替换的透明原画分别保留，不能以 GIF 替代生产资产。

## 当前动画设计

同一本厚书从侧面垫入→一级厚度停留→二级停留→三级停留→回血减为二级→一级→整本移出。全过程不打开、不写字、不增加第二本书。增厚主要改变纸页区域，上／下封皮保持自身尺寸；角色脚底随书面升降，身体外形、护甲和头发不变。

本次媒体合成把当前选用书图分成上部、纸页段、底部，只有纸页段随时间改变高度，顶面随之移动，人物是另外叠加的独立输入。输入图的具体裁切只服务本次预览，不是未来游戏的固定接口；替换人物时保留其脚底锚点与书面高度对齐即可，无需重做独立书动画。

当前人物裁切／书裁切均为展示用途。身体相对骑士的 1:2 比例是已记录的正式设计要求，本次不含骑士并排比较，不能声称已完成生产比例验收。正式动画仍需按像素网格、alpha边缘与相同脚底锚点校准。

## 已弃用

- `dwarf-idle-runes-v1.png`：非持锤手仍有不明握柄，未通过，不能继续当作当前角色贴图。
- `grudge-book-study-v1.png`：开合方案废弃。
- `grudge-book-closed-v2.png`：太薄，废弃。
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
