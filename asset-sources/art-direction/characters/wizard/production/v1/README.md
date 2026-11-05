# 老巫师四组十二姿势 · v1

**已撤回的历史造型。** 用户试玩后明确要求保留原来的粗像素小人，当前生产入口为 [v2](../v2/README.md)。本目录不再进入运行时，不代表当前认可的画风。

Issue #27 / Refs #11。用户于2026-11-05批准全身蓝袍、弯尖帽、白胡须、棕鞋、蓝晶木杖；觉醒沿用身体与装备，以站姿和杖端亮度区分。概念选择与独立介绍状态见[角色档案](../../../../../../docs/art/characters/WIZARD-LEGACY.md)。最终视觉验收仍由用户完成。

四份 `*-prompt-v1.md` 保存完整 E 方案前缀与逐姿势说明，四份 `*-source-v1.webp` 为内置 imagegen 生成的透明原图无损转换，1448×1086、4×3网格。每组12张独立姿势；没有以位移或插值补帧。普通待机参考旧普通／觉醒素材，其余组以前组为造型参考。

运行 `.build-tools/python-path.txt` 指定的 Python：

```powershell
& (Get-Content .build-tools/python-path.txt) tools/prepare_character_animations.py --role wizard
& (Get-Content .build-tools/python-path.txt) tools/prepare_character_animations.py --registry-only
```

完整 `tools/prepare_assets.py` 也包含该入口。导入依赖 Pillow 13；只做网格切分、alpha阈值160、脚底注册、共同比例最近邻缩放、共享64色量化（56基础色＋8蓝晶强调色，无抖动）及无损导出，不合成新姿势。`registration.json` 逐帧保存人工检查的鞋底原点，整组横向原点固定，不能按伸出的杖或手重新居中。比例0.972644应用于全部48帧；普通最大可见高度316、觉醒320。最终图集2304×3072、单帧384×384、脚锚(192,336)，脚底不超过336，透明像素清零。

`report.json` 记录48个独立像素哈希、边界、来源、脚锚、色板及帧时长。输出 `assets/roles/wizard.webp` 是唯一角色运行时图集；普通／觉醒共享一次纹理解码。此目录源稿、提示词、四个动画WebP和首帧仅供重建与评审，不进入游戏包。动画WebP为方便检查循环播放，运行时施法为 `Playback::Once`。

待机每帧60ms。施法时长ms：`60,80,90,90,120,180,280,100,100,100,100,100`，总1400。第5帧释放（320ms），第7帧引导（620–900ms），第8帧命中（900ms）。延迟命中时保持第7帧，实际清除后第8帧及500ms收势；收势结束从待机首帧进入。两种技能共用动作，不修改模型收益时间。完整录像、同尺寸对比和像素同步测量见[验收目录](../../../../../../analysis/issue27-wizard-animation/README.md)。
