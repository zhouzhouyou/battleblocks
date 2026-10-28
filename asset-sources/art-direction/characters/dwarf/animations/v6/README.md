# 仇恨之书垫高动画 · v6（当前）

日期：2026-10-29。依据 `../../dwarf-model-sheet-black-red-v5.png` 的黑红无盔矮人和**装备拆分中的仇恨之书**重制。最新用户原话：「记得移除之前的稿件，重新制作仇恨之书垫高角色动画。仇恨之书采用你刚才生成的人设图中的设计。」「人设图的装备拆分里面有仇恨之书的图案啊，按那个来」。

## 当前文件

- `dwarf-awake-source.png`：内置 ImageGen 根据 v5 人设图生成的独立人物源图，黑红无盔、三束橙胡须、空手、对称精良战锤、红色锤形符文。不含书。
- `dwarf-awake.png`：用于本轮媒体合成的透明人物层，350×326。只整理透明边缘、边界与展示尺寸，未将书合入。
- `grudge-book-model-crop.png`：**直接裁自人设图**的原始书图，180×184；参考裁切范围为人设图 `(1345,620,180,184)`。保留原设计，无重新绘制封面。
- `book-cover.png`：从该原图封面做透视整理所得透明封皮层，280×112。金色边框、包角、铆钉、锤形标记均来自原图像素，不采用另行生成的大号 T 书封面。
- `book-pages.png`：从同一书图的纸页区域裁出，280×40；动画只扩展这一层的高度。
- `book-bottom.png`：同一书图底部封皮层，280×12。
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

旧蓝衣／头盔／分叉符文矮人图、旧多本／开合／其他书样式和旧 GIF 已按用户要求从当前工作树移除。此前已提交的历史原图可通过 Git 提交 `2aa2b99` 查看，不再从生成缓存重新复制到当前资产目录。历史提示词文档仅保留用于追溯，不表示对应旧图片仍为有效资产。

## 当前人物源图生成提示词

```text
Use case: precise-object-edit / character asset extraction.
SOURCE: the supplied completed "矮人E-A人设图" black-red BAREHEADED dwarf model sheet. This is the only current design. Create ONE isolated FULL BODY awakened dwarf on transparent background, no book, floor, shadow, labels, grid or other figures.

Copy the SAME middle-aged male character from the LEFT FRONT turnaround: swept-back orange hair, no helmet or crown, rugged face, blue eyes, three orange beard braids tied with brass rings, short very broad adult dwarf body, gray steel shoulder caps edged brass, charcoal BLACK leather smith apron, dark RED tunic and tabard, dark bracers, wide belt and gray-capped heavy boots. Preserve EXACT outfit, proportions, face, hair, three braids, palette and refined E pixel-cluster style. Do not add more armor, hair, spikes, cape or new equipment. Slight front-three-quarter standing pose, legs firmly planted, boots aligned on same level. BODY height is HALF human knight in game. Leave generous transparent border (at least10% per side), all hair/feet/weapon visible. Aim centered in landscape or square image.

His viewer-left FREE hand hangs clearly OPEN and EMPTY away from belt, no tool, rod, dagger handle or ambiguous buckle touching palm. Viewer-right hand holds the EXACT warhammer from source weapon front view upright beside body: high-quality RECTANGULAR hammerhead with TWO IDENTICAL MIRRORED flat striking ends, equal mass and length on both sides of centered shaft, polished steel with matching bronze structural bands, straight dark-red-wrapped handle and compact bronze pommel. No picks, blades, lopsided head, cracks, rust, moss or random extra ornaments. Same centered simple HAMMER-SHAPED T PICTOGRAM engraved on broad hammer face (a horizontal rectangular hammerhead + central vertical handle), NOT Y or trident. It glows controlled RED because this is awakened stance. Keep carving crisp, not giant flames or whole body light.

This asset will be MOVED UPWARD by a separately layered thick book in a review animation; do NOT include any book or footstool and do not change body height. Neutral steady idle pose, NO action swing. Character's feet must have a clear flat baseline for anchoring.

Art direction E: visibly discrete squared pixel clusters, nuanced3–4stepped material ramps, source model sheet level of refinement, expressive readable face and angular hair clusters, broad quiet leather and red cloth planes, crisp contours, upper-left light. Not smooth glossy3D, not painted illustration blurred to pixels, no gradients or dither noise. TRUE transparent alpha outside figure. Single full-body dwarf only.
```

书的最终封面、纸页和底封直接来自 v5 人设图，采用媒体裁切／透视整理与分层合成。中途另画的书图没有采用，已清理，不作为本轮素材。
