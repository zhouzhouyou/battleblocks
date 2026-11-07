# 小恶魔：觉醒动作

## 角色介绍

沿用 `config/role-lore.txt`：契约小恶魔，擅长平衡引爆，也非常尊重契约。普通版保持现有资源、播放、尺寸和施法表现。

## 当前造型与动作

觉醒保留红色方脸、紫色双角、蝠翼、靴子与三叉戟。待机通过膝部、肩部、手腕和翅膀形成紧凑循环；施法先压低重心、举戟、前指，再回收。身体只有两条手臂，释放时仍握住武器。

待机和施法各十二个独立姿势。均衡置换沿用即时应用，施法单次播放；平衡引爆第七姿势对齐本次目标实际清除，无目标时使用有界单次播放。模型收益和结算时机不由动画改变。

正式源图、固定调色板和配准位于 `assets-source/roles/imp/animation/`；运行图集为 `assets/roles/imp/runtime/atlas.webp`。通用规格见 [全局美术方向](../ART-DIRECTION.md)，播放契约见 `src/sprites/legacy.rs`。

## 制作提示词

使用内置 imagegen，以本角色原觉醒造型为身份参照，先生成二十四格，再修正释放姿势的手臂。源图按上下两半拆为两组，保留透明通道并按固定色板和脚底配准导入。

```text
Create a production sprite sheet for this exact awakened imp character, preserving its identity: squat red rectangular face, purple horns and bat wings, purple boots, small purple trident in right hand. Densely detailed pixel illustration, crisp pixel clusters and stepped material ramps, no smooth painting, upper left light. Transparent background. EXACTLY 24 full-body independent animation poses in a strict 4 columns by 6 rows equal cell grid, read left to right top to bottom, no grid lines or labels. Each cell has a generous transparent margin, no overlap. Fixed camera, consistent size, feet on same baseline. First 12 cells: seamless idle cycle, breathing, knees, wrist and wings articulate subtly but every pose distinct, ends approaching first. Last 12 cells: one cast sequence, 1 ready, 2 crouch, 3 wind up, 4 raise trident, 5 aim, 6 hold anticipatory tension, 7 release with forward trident and open left hand, 8 follow through, 9 recoil, 10 lower weapon, 11 settle, 12 near idle. Do not duplicate or merely translate frames. Preserve equipment and facial identity throughout. No added armor or hair, no effects outside silhouette, no shadows on ground, no text. Large landscape-independent sheet with square cells; 4:6 overall aspect ratio. These are awakened art only.
```

当前源图采用的局部修正指令：

```text
Correct only the anatomy of the imp's casting poses in this sprite sheet, preserving the 4-column 6-row layout and all other poses, size, crisp pixel style, palette and equipment. Specifically row 5 columns 3 and 4 must have EXACTLY TWO ARMS AND TWO HANDS: one hand on viewer-left attached to its existing arm and one hand on viewer-right gripping the trident shaft. Remove the spurious extra open hand below the trident in row5 col3; row5 col4 must visibly GRIP the trident instead of floating open fingers beside it. The purple trident is always held by the hand on viewer-right. Do not add arms, hands, text, effects or background. Preserve real alpha transparency, no backdrop. Keep all 24 poses and existing grid unchanged.
```
