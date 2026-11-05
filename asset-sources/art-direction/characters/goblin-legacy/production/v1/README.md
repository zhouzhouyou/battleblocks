# 哥布林 · 拾荒者 — 觉醒动画 production/v1

用户已批准 #28 推荐方案，范围仅觉醒。内置 image_gen 生成；实际提示词为 awake-idle-prompt-v1.md 与 awake-cast-prompt-v1.md，参考已有 assets/awakened/goblin-awakening.webp。源图无损保存，不打包进游戏。

registration.json 锁定每张源表的12个脚底中心和一次性密度校准；palette.json 为两动作共用64色。导入只做4×3切帧、alpha>=160二值化、最近邻缩放、脚部注册和量化，没有复制/平移帧扩充动作。

运行指定 Python：tools/prepare_character_animations.py --role goblin-legacy，然后 --registry-only。完整 tools/prepare_assets.py 同样可复现。runtime图集 assets/roles/goblin-legacy.webp；每个源表12独立姿势，施法预览WebP为1.4秒循环以便评审，运行时明确 Once，按技能采样时长见 src/sprites/legacy.rs；待机0.72秒循环。

最终视觉待用户验收；逐帧/实战证据见 analysis/issue28-awake-animation。普通角色文件不属于该导入器输出。

