> 历史记录：对应旧稿已按用户要求从工作树和 Git 历史移除。当前只使用 v5 人设图与 `animations/v6/`，不要据本文恢复旧素材。

# 矮人战士首轮评审：生成记录

工具：内置 ImageGen。用途：概念评审，不是生产用动画或正式导入素材。样式参考：`../../f-reference.webp`。

全局提示词与制作约束：[ART-DIRECTION.md](../../../../docs/art/ART-DIRECTION.md)。本轮使用下列资产级提示词收简第一轮过密的细节。原始最终预览保存在 `dwarf-review-v1.png`；`dwarf-review-v1.webp` 是像素一致的无损副本；`dwarf-review-ui-preview.webp` 是仅供界面预览的压缩副本。未采纳的第一轮过密图不作为最终交付素材。

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
