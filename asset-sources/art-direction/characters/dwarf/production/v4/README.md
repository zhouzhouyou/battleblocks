# dwarf 十二帧动画 v4

Issue #11 第一阶段实现稿，覆盖新五人；尚不代表用户最终美术验收。沿用既定人设，历史生产版本保留。

- 每个 `*-source-v1.webp` 是内置 ImageGen 生成的独立 4×3 源图，12 张不同姿势；对应 `*-prompt-v1.md` 保存实际提示词。
- 动作：normal-idle / awake-idle / normal-cast / awake-cast。运行时每角色 48 帧、384×384，6×8 图集（2304×3072），脚底锚点 (192,336)。
- `*.webp` 动作预览均由运行时同样的量化帧导出。待机一轮 720 ms；施法及入睡 1400 ms；预览为便于比较循环播放，游戏内施法／入睡只播放一次并保持末帧。
- 全角色各自共享至多 64 色、二值透明、最近邻采样、无损 WebP。不是把原四帧重复三次，也没有光流或交叉淡化。
- `report.json` 记录尺寸、色板、源动作统一缩放、帧边界和像素哈希。自动检查不能替代对武器、服装、姿势衔接的视觉审查。
- 保留原人物介绍与能力配置。矮人书本层继续使用 v6；火法师两形态共用施法及入睡动作。

复现：用 `.build-tools/python-path.txt` 指定的 Python 安装 `tools/art-requirements.txt` 到 `.build-tools/art-site`，运行 `tools/prepare_character_animations.py`。游戏只打包 `assets/`，本目录源图和预览不进入游戏包。
