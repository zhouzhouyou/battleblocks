# 修女 N-A 游戏美术资源 v1（待用户评审）

根据用户选择的 N-A 人设图制作。`assets/roles/nun.webp` 已被游戏加载；两张立绘 WebP 已导出，但尚未被渲染代码引用。

## 文件

- `animation-source-v2.png`：修去十字形光点与大面积光晕后的 4×4 透明动画源图，当前导出来源。初次生成的 v1 源图已按 2026-11-01 清理移出仓库，不用于导出。
- `grid-384.png`：16 帧、每帧 384×384 的完整透明图集。第 1 行普通待机，第 2 行暗影待机，第 3 行普通治疗施法，第 4 行暗影攻击施法；每行 4 帧。
- `normal-idle-0.png`、`awake-idle-0.png`：两种形态的待机首帧。
- `normal-idle.gif`、`awake-idle.gif`、`normal-heal.gif`、`awake-attack.gif`：四组动画预览。
- `standee-normal-source-v1.png`、`standee-awake-source-v1.png`：独立全身立绘原图。
- `standee-normal.png`、`standee-awake.png`：共享色表、二值透明的独立立绘导出。
- `report.json`：逐帧边界、色数、锚点与输出尺寸检查。
- [PROMPT.md](PROMPT.md)：完整生成及修订提示词。

游戏资源：动画图集为 `assets/roles/nun.webp`（已加载）；`assets/roles/nun-standee-normal.webp`、`nun-standee-awake.webp` 已导出但尚未被渲染代码引用。复现导出运行 `tools/prepare_nun_assets.py`；它不覆盖其他角色图集或角色注册表。

## 规格与评审重点

- 动画：4×4 图集 1536×1536；各帧 384×384；可见高度约 320；脚底锚点 (192,336)；角色资源共享 64 色、无抖动、二值透明、最近邻缩放。
- 独立立绘：普通与暗影各 1024×1536，二值透明，并使用与动画相同的 64 色表。
- 已检查 16 帧的边界、透明度、色数，四组 GIF 各有 4 张不同画面；无额外手持道具。透明图集与无损 WebP 解码后像素一致。
- 此批图仍待用户视觉评审。立绘源图的灯光范围、动作节奏和暗影表现尚未视为用户定稿；导出前已将动画初稿的十字形光点与大面积光晕修掉。
