# 大僵尸：觉醒动作

## 角色介绍

沿用 `config/role-lore.txt`：血肉施法者，用血肉支付缺失魔力的大僵尸。每次施法都像一次反向验尸。普通版保持现有资源、播放、尺寸和施法表现。

## 当前造型与动作

觉醒保留灰褐色厚重身体、方形下颌、头顶绿芽、棕色衣物和粗制短柄武器。待机是缓慢的肩胸起伏，施法通过含胸、后蓄、沉重下挥和迟滞收势表现体重。下挥武器可以低于鞋底，配准始终使用鞋底而非武器包围盒。

待机和施法各十二个独立姿势。生命汲取第八姿势对齐本次目标实际清除；伤害转生沿用即时应用。生命代付、觉醒和收益保持模型现有规则，不为播放动画延迟结算。

正式制作输入位于 `assets-source/roles/zombie/animation/`；运行图集为 `assets/roles/zombie/runtime/atlas.webp`。通用规格见 [全局美术方向](../ART-DIRECTION.md)，播放契约见 `src/sprites/legacy.rs`。

## 制作提示词

使用内置 imagegen，以原觉醒造型为身份参照；二十四格按上下两半拆分，固定配准与调色板用于重建。源图中最明显的低点是第八施法姿势，因此播放配置按实际源图对齐；不据源图改变角色设定。

```text
Create a transparent production animation sprite sheet preserving this exact awakened big zombie character: massive stocky gray-brown flesh body, square slack jaw with few teeth, small green sprout on bald scalp, brown shorts and straps, brown boots, short crude dark wood/stone club in right hand. No armor, hair, new weapons or changed identity. Densely detailed pixel illustration with crisp clusters and stepped ramps. EXACTLY 24 full-body poses in 4 columns x 6 rows equal square cells, 1024x1536 image. Fixed camera and scale, feet baseline, generous transparent margins in each cell. Rows 1-3: 12 independent consecutive idle poses, slow heavy breathing and swaying shoulders, changing elbows/knees and sprout tip, seamless cyclic return. Rows 4-6: 12 cast poses, ready, hunch, pull left arm inward, raise club, lean back, hold anticipation, thrust free hand forward and drive club down (pose 7), heavy followthrough, stagger recovery, lower arms, regain balance, settle close to idle. Convey ponderous mass rather than agile demon. All 24 poses independently articulated, not translations or duplicated images. GENUINELY TRANSPARENT alpha outside sprites, no colored background, no floor, no glow, no shadows, no text or grid.
```
