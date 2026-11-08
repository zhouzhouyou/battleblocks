# 角魔：觉醒动作

## 角色介绍

沿用 `config/role-lore.txt`：血怒角魔，血越少越危险。他倒得越低，棋盘越该担心。普通版保持现有资源、播放、尺寸和施法表现。

## 当前造型与动作

觉醒保留红色头部、象牙双角、露齿大嘴、黑红装备和宽刃斧。待机以肩部呼吸、握斧和膝部变化表现重量；施法通过扭身蓄力、举斧、下劈和沉重收势区分小恶魔。斧刃低于脚底时以鞋底配准，不能把武器下沿误当脚底。

待机和施法各十二个独立姿势。战意转化沿用即时应用；同类收割第七姿势对齐本次目标实际清除。单次施法结束后返回待机；不改变技能数值或结算。

正式制作输入位于 `assets-source/roles/chort/animation/`；运行图集为 `assets/roles/chort/runtime/atlas.webp`。通用规格见 [全局美术方向](../ART-DIRECTION.md)，播放契约见 `src/sprites/legacy.rs`。

## 制作提示词

使用内置 imagegen，以原觉醒造型为身份参照；二十四格按上下两半拆分，固定配准与调色板用于重建。

```text
Production transparent PNG sprite sheet. Exact reference awakened horned demon identity: broad red head, two ivory horns, toothy mouth, black armor with red straps, heavy silver curved axe, stocky legs. Densely detailed pixel illustration, crisp stepped clusters, no blur or gradients. EXACTLY 24 full body unique poses arranged 4 columns x 6 rows square equal cells, 1024x1536 sheet. No background: genuine alpha transparency everywhere outside characters, no colored backdrop or glow. Each character well inside its cell, fixed scale and same foot baseline, generous blank margins. Rows 1-3 are 12 consecutive seamless idle breathing poses with changing shoulders, knees, grip and jaw, distinct poses not translations. Rows 4-6 are 12 consecutive cast poses: ready, brace, pull axe back, twist body, raise axe, anticipation, powerful forward axe release (pose 7), heavy followthrough, rebound, lower axe, regain stance, near idle. Feet grounded, distinct inertia and recovery. Preserve horns, axe, body proportions, clothing in every frame. No labels, grid lines, particles, floor shadows or new equipment.
```
