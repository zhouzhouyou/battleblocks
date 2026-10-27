# BattleBlocks 全局美术方向：E / 像素簇与材质

状态：用户于 2026-10-28 最终改选 E 方案，并选择矮人 E-A 进行修订。适用于全项目所有新美术；以下英文提示词为用户提供的 E 原文，后续资产请求以此为全局前缀，再追加具体资产说明。F 已存入 `ART-DIRECTION-F-ARCHIVE.md`，不再作为当前默认。各角色设定与明确的例外优先于通用造型建议。

## 全局提示词（原文）

```text
Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.
```

## 项目约束

- 西式奇幻、2D 三消回合战斗、共享 8×8 棋盘；macOS 原生与 Web。美术评审不改变战斗规则。
- 保留现有正面／四分之三角色展示；角色、物件与 UI 统一美术语言。
- 参考图：`../../asset-sources/art-direction/e-reference.png`。参考图是概念预览，不保证符合生产用尺寸、色数、网格或透明度。

## 制作规范

| 项目 | 规范 |
| --- | --- |
| 全局像素 | 1 个逻辑像素为最小绘制单位；最近邻采样，优先整数倍缩放；禁止不同资产随意改变像素密度 |
| 角色 | 生产目标统一 96×96 逻辑像素画布与脚底锚点；普通人形身体高度以 64 像素为标尺，矮人身体 32 像素；装备与脚下书另计，超界扩画布而非改变像素尺度 |
| 比例 | 以同地平线、同像素尺度比较身体；矮人高度明确为正常角色（例如骑士）的一半；性别不强制对应体型或职业能力 |
| 方块物件 | 独立 24×24 透明画布，主体建议不超过 20×20；在棋盘原尺寸验证，不能从角色大图缩小得到 |
| 场景与边框 | 16×16 基础拼接单元，可组合为 32×32；避免细纹与棋盘主体竞争 |
| 色数 | 全局基准 24 色；单角色每帧通常选择 10–14 色，统一管理新增色，透明不计 |
| 明暗 | 左上主光；焦点材质 3–4 级有色相变化的阶梯色阶，安静表面 2 级；不以模糊光晕掩盖像素轮廓 |
| 动画 | 待机目标 4 帧、动作目标 4–6 帧；整数像素位移，保留道具和头部轮廓；生产时逐帧验证而非把概念稿当动画 |
| 文字与操作 | 文字独立渲染，优先可读字体；常规正文目标至少 16 逻辑像素；交互热区目标至少 44×44，可大于可见像素图标 |
| 导出 | 生产资产 PNG 透明底或现有管线支持的无损 WebP；正式导入前检查尺寸、色数、alpha 与实际显示效果 |

### 锁定的 24 色基准

`#161820 #252633 #373847 #535064 #787080 #ACA0A1 #E2D4BC #FFF0CD #352736 #653341 #9F4350 #D87066 #583C31 #885739 #BC8552 #DAB573 #293D32 #426644 #73945B #B1C879 #293B58 #45658A #7699B7 #8866A5`

背景／轮廓用前两色；金属用短而硬的冷灰阶梯亮边；皮革与木材用宽暖色块；布料用成片折面与色相变化；叶片成簇、石材用大平面。细节集中脸部、手部与标志物，不堆满通身碎点。色板约束作用于游戏资产，正常可读文字和评审页面不受单角色色数限制。

### 轮廓、觉醒与可读性

- 每人拥有独立的头部形状、身体重心、服装外形和标志物；同类角色也不能只换颜色。
- 觉醒以各角色档案为准，保留脸部、主色与身份特征；不默认追加护甲、头发、披风或光环。矮人只改变锤子红色符文发光，仇恨之书独立垫脚，身体不变。
- 性别、年龄与体型是候选维度，不用性别刻板区分职业能力；女性矮人可以穿重甲、使用重锤并保持宽厚体型。
- 正常尺寸先看得清；放大预览只辅助检查像素，不能代替游戏内可读性。
- 避免：随机碎点、抖动纹理、混合像素密度、抗锯齿、模糊缩放、精细插画缩小、现代亮面效果、觉醒后细节暴增、不同角色共用同一轮廓。

## 方块与角色关系（设计阶段）

每种收益对应两位角色；每局从两位角色的物件中随机选一个外观，本局同类方块保持一致。此项为用户选定的美术设计，尚未实现；不据此推定技能或数值已经改变。物件最终造型随角色评审确定。

| 收益 | 角色与当前物件意象 |
| --- | --- |
| 伤害 | 矮人战士／符文战锤；骑士／长剑 |
| 治疗 | 牧师／圣徽；修女／祈祷珠 |
| 魔力 | 巫师／木法杖；女巫／榛木魔棒 |
| 治疗＋伤害 | 吸血鬼伯爵／吸血匕首；血族女爵／血酒杯 |
| 治疗＋魔力 | 精灵德鲁伊／新芽枝杖；树灵／生命种荚（不添加蓝色魔力晶体） |
| 伤害＋魔力 | 契约术士／恶魔契约；暗影祭司／黑烛 |
| 三种收益 | 哥布林盗贼／战利品袋；狗头人工匠／冒险工具箱 |
