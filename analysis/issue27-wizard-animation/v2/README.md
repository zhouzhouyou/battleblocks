# Issue #27 · v2恢复粗像素小人

用户试玩v1后明确要求：「改回去，粗像素小人保留」。本版本撤回细密全身重绘，保留十二帧播放、命中同步和原战斗规则。普通恢复原始半身、大色块、方块眼、长鼻、白胡须与弯尖帽；待机首帧直接保留旧图像素。觉醒沿用旧小全身的蓝袍、鞋与木杖轮廓，以粗像素重新扩展动作，不再与普通强制统一全身构图。

## 当前素材与实际显示

四组各12独立姿势，共48帧。24×24逻辑像素格按16倍最近邻封装到384×384；原七色加两种晶石强调色，二值透明，图集5,932字节。普通稳定的是半身下边缘，觉醒稳定的是鞋底，均在y=336；不能将普通称为“有脚的全身”。原始普通／觉醒文件未修改。[版本化源稿、提示词与导出规范](../../../asset-sources/art-direction/characters/wizard/production/v2/README.md)。

- [普通同尺寸前后对比](normal-menu-comparison.webp) / [觉醒同尺寸前后对比](awake-menu-comparison.webp)：左为main原素材，右为当前v2，真实窗口原尺寸裁剪。
- [48帧图集](atlas-contact.webp) / [11→12→1→2接缝与基线](idle-seams.webp)：图集诊断，区别于真实窗口录像；普通头帽会按粗像素格变化，旧画法的大色块不能用平滑插值掩饰。
- [普通待机录像](current/normal-menu.mp4) / [觉醒待机录像](current/awake-menu.mp4)。
- 普通 [落雷](current/normal-lightning.mp4) / [陨石](current/normal-meteor.mp4)；觉醒 [落雷](current/awake-lightning.mp4) / [陨石](current/awake-meteor.mp4)。
- [延迟陨石](current/delayed-meteor.mp4) / [终局](current/terminal-meteor.mp4) / [回滚](current/rollback.mp4)。
- [交互帧检查页](index.html)：本地仓库HTTP服务打开；图集采样与完整实战视频分别标注。

## 命中复验

沿用既有ImpactClock和动作时长：第5帧出手、第7帧引导等待、第8帧匹配真实清除，再0.5s收势。录像通过已有控制台驱动真实Session，60fps、1280×900，旁边同名JSONL为原始事件。新图集重新进行像素匹配，未复用v1测量。

| 场景 | 施法到真实清除 | 人物第8帧 / 特效命中视频帧（从0开始） | 差值 |
| --- | --- | --- | --- |
| 普通落雷 | 0.895s | 113 / 113 | 0帧 |
| 普通陨石 | 0.907s | 113 / 113 | 0帧 |
| 觉醒落雷 | 0.947s | 116 / 116 | 0帧 |
| 觉醒陨石 | 0.908s | 114 / 114 | 0帧 |
| 延迟陨石 | 1.515s | 150 / 150 | 0帧 |

[逐帧数据](recording-measurements.json)与各 `*-synchronization.webp` 可复核。终局胜利面板遮住特效，不计入同步测量；终局正常收势，回滚脚本断言完整快照恢复。拍摄时HEAD尚为v1的5f5c876，`current/capture.json`中的git_head不是v2源码归属；通过二进制SHA及追加的source-binding绑定本次实现。

## 验证与审查

新增回归测试锁定原普通中立帧的像素，以及全图每个16×16块必须为同色，最多9色；不会只凭“像素风”字样放行细密重绘。四组独立姿势、alpha、基线、图集布局测试仍保留。

当前Rust357通过、1音频设备测试忽略；Python18通过；headless 134事件与2局autoplay通过。Windows默认GBK曾导致验证脚本解码失败，设置 `PYTHONUTF8=1` 后完整通过，没有修改游戏逻辑来绕过。完整素材重建后所有assets哈希变化0。Windows release、WASM和网页包已重建，release中包含当前图集。GUI、macOS与独立审查结果在本目录交付记录中补充。

旧v1证据位于父目录，只反映已撤回的细密画法。此次仍保持Draft PR，Closes #27、Refs #11；用户最终查看粗像素表情、头帽运动和施法姿势，主Issue #11保持打开。
