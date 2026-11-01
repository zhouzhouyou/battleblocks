# BattleBlocks 全局美术方向：F / 8-bit-inspired pixel art

状态：用户于 2026-10-28 选择 F 方案。适用于全项目所有新美术；以下英文提示词为用户提供的原文，后续资产请求以此为全局前缀，再追加具体资产说明。

## 全局提示词（原文）

```text
Act as the art director for an original game. Establish 8-bit-inspired pixel art as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Choose and document a small logical pixel resolution, tile grid, sprite scale and limited color ramps appropriate to the game. Use deliberate pixel clusters, crisp contours and reusable tile shapes. Keep characters, props, UI icons and effects aligned to the same visual pixel scale, scaling by integer factors wherever practical. Favor clear silhouettes and economical animation frames over fine texture. Use limited stepped shading and a small, controlled accent palette. This is an original visual direction, not a claim to emulate specific hardware exactly. Avoid accidental antialiasing, mixed pixel densities, blurred scaling and modern glossy effects that overwhelm the pixel structure.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.
```

## 项目约束

- 西式奇幻、2D 三消回合战斗、共享 8×8 棋盘；macOS 原生与 Web。美术评审不改变战斗规则。
- 保留现有正面／四分之三角色展示；角色、物件与 UI 统一美术语言。
- 参考图：`../../asset-sources/art-direction/f-reference.webp`。参考图是概念预览，不保证符合生产用尺寸、色数、网格或透明度。

## 制作规范

| 项目 | 规范 |
| --- | --- |
| 全局像素 | 1 个逻辑像素为最小绘制单位；最近邻采样，优先整数倍缩放；禁止不同资产随意改变像素密度 |
| 角色 | 普通形态以 16×24 逻辑像素画布为目标；觉醒最多 24×28，保持相同像素尺度和脚底锚点；超尺寸需先评审 |
| 比例 | 人形约 2.5 头身；矮人、哥布林、狗头人约 2 头身；性别不强制对应身高、装备或职业能力 |
| 方块物件 | 16×16 透明画布，主体建议不超过 14×14；在棋盘原尺寸验证，不能从角色大图缩小得到 |
| 场景与边框 | 8×8 基础拼接单元，可组合为 16×16；避免细纹与棋盘主体竞争 |
| 色数 | 全局 16 色；单角色每帧目标不超过 6 个不透明颜色，透明不计；方块目标不超过 5 色 |
| 明暗 | 左上主光；每种材质通常主色＋阴影两级，亮点仅用于焦点；不以渐变或柔光掩盖轮廓 |
| 动画 | 待机目标 4 帧、动作目标 4–6 帧；整数像素位移，保留道具和头部轮廓；生产时逐帧验证而非把概念稿当动画 |
| 文字与操作 | 文字独立渲染，优先可读字体；常规正文目标至少 16 逻辑像素；交互热区目标至少 44×44，可大于可见像素图标 |
| 导出 | 生产资产 PNG 透明底或现有管线支持的无损 WebP；正式导入前检查尺寸、色数、alpha 与实际显示效果 |

### 锁定的 16 色板

`#171A25 #303348 #666B80 #A5ABB3 #EEE6CC #C89962 #79523B #D99A37 #913D47 #D05B50 #304B39 #659447 #B6C877 #344F87 #759BBC #8868A5`

背景／轮廓用前两色；金属用冷灰与象牙亮点；皮革与木材用棕色；布料用一个主色和一个暗色；植物用成组绿色色块。色板约束作用于游戏资产，正常可读文字和评审页面不受单角色色数限制。

### 轮廓、觉醒与可读性

- 每人拥有独立的头部形状、身体重心、服装外形和标志物；同类角色也不能只换颜色。
- 觉醒保留脸部、主色与身份特征；优先改变一个主要轮廓、装备或姿态，避免全员加光环、披风或粒子。
- 性别、年龄与体型是候选维度，不用性别刻板区分职业能力；女性矮人可以穿重甲、使用重锤并保持宽厚体型。
- 正常尺寸先看得清；放大预览只辅助检查像素，不能代替游戏内可读性。
- 避免：随机碎点、抖动纹理、混合像素密度、抗锯齿、模糊缩放、精细插画缩小、现代亮面效果、觉醒后细节暴增、不同角色共用同一轮廓。

## 方块与角色关系（设计阶段）

每种收益对应两位角色；每局从两位角色的物件中随机选一个外观，本局同类方块保持一致。此项为用户选定的美术设计，尚未实现；不据此推定技能或数值已经改变。物件最终造型随角色评审确定。

| 收益 | 角色与当前物件意象 |
| --- | --- |
| 伤害 | 矮人战士／战锤；骑士／长剑 |
| 治疗 | 牧师／圣徽；修女／祈祷珠 |
| 魔力 | 巫师／木法杖；女巫／榛木魔棒 |
| 治疗＋伤害 | 吸血鬼伯爵／吸血匕首；血族女爵／血酒杯 |
| 治疗＋魔力 | 精灵德鲁伊／新芽枝杖；树灵／生命种荚（不添加蓝色魔力晶体） |
| 伤害＋魔力 | 契约术士／恶魔契约；暗影祭司／黑烛 |
| 三种收益 | 哥布林盗贼／战利品袋；狗头人工匠／冒险工具箱 |
