# 旧骑士 · 旧誓骑士 — 觉醒动画 production/v1

用户已批准 #28 推荐方案，范围仅觉醒。内置 image_gen 生成；实际提示词为 awake-idle-prompt-v1.md 与 awake-cast-prompt-v3.md，参考已有 assets/awakened/knight-awakening.webp。源图无损保存，不打包进游戏。施法 v1 因横向伸剑安全边界与体型偏差被替换，v3进一步修复脚距，为当前源图；v1仅保留过程记录。

registration.json 锁定每张源表的12个脚底中心和一次性密度校准；palette.json 为两动作共用64色。导入只做4×3切帧、alpha>=160二值化、最近邻缩放、脚部注册和量化，没有复制/平移帧扩充动作。

运行指定 Python：tools/prepare_character_animations.py --role knight-legacy，然后 --registry-only。完整 tools/prepare_assets.py 同样可复现。runtime图集 assets/roles/knight-legacy.webp；每个源表12独立姿势，施法预览WebP为1.4秒循环以便评审，运行时明确 Once，按技能采样时长见 src/sprites/legacy.rs；待机0.72秒循环。

最终视觉待用户验收；逐帧/实战证据见 analysis/issue28-awake-animation。普通角色文件不属于该导入器输出。

