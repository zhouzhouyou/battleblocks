# 老巫师觉醒动画 · 当前验收

普通版保持原来的粗像素素材和播放方式；本 PR 只增加细像素觉醒待机与施法，各 12 个独立姿势，共 24 帧。普通版未扩帧、未重绘，仍为 300×400、四帧、每帧 0.1 秒，沿用原缩略图、缩放和施法回退。

已按用户要求先更新 [主 Issue #11](https://github.com/zhouzhouyou/battleblocks/issues/11) 和全部子项 [#27](https://github.com/zhouzhouyou/battleblocks/issues/27)、[#28](https://github.com/zhouzhouyou/battleblocks/issues/28)、[#29](https://github.com/zhouzhouyou/battleblocks/issues/29)，回读确认后才改代码。[范围正文快照](issue-scope/) 保留本次核实内容。Draft PR 使用 `Closes #27`、`Refs #11`；主 Issue 保持打开。

## 看图与实战

- [普通同尺寸对比](normal-menu-comparison.webp)：左为 main，右为当前；同一原始姿势的真实窗口裁剪。普通素材、缩略图字节及普通注册配置与基线一致。
- [觉醒同尺寸对比](awake-menu-comparison.webp)：左为 main 单帧，右为细像素动画；[实际选角录像](current/awake-menu.mp4)。
- [24 帧图集](atlas-contact.webp)、[待机 11→12→1→2 与鞋底线](idle-seams.webp)：按实际选角尺寸检查接缝，不是实战录像。
- 觉醒实战：[落雷](current/awake-lightning.mp4)、[陨石](current/awake-meteor.mp4)、[延迟命中陨石](current/awake-delayed-meteor.mp4)、[终局](current/awake-terminal-meteor.mp4)、[回滚](current/awake-rollback.mp4)。
- 普通回归录像：[待机](current/normal-menu.mp4)、[落雷](current/normal-lightning.mp4)、[陨石](current/normal-meteor.mp4)。
- [交互检查页](index.html)：通过本地仓库 HTTP 服务打开；可逐帧看图集、切换实际选角／战斗尺寸，并播放真实窗口录像。
- [版本化觉醒源图、提示词及导出报告](../../asset-sources/art-direction/characters/wizard/production/v3/README.md)。

觉醒图集为 2304×1536、384×384 帧、64 共享色、二值透明、鞋底锚点 (192,336)。0–11 为待机，12–23 为施法，普通帧不在此图集中。最终用户视觉验收仍需看觉醒脸部浅色块、袍褶密度、施法幅度及待机循环；测试通过不代替审美确认。

## 两技能共用一组觉醒施法

第 5 帧出手，第 7 帧引导等待，第 8 帧命中，随后 0.5 秒收势。人物和两技能特效共用实际命中时钟；对手觉醒延迟清除时保持引导姿势，收到真实清除通知后一起命中。终局无清除通知时按墙钟结束，回滚重置表现时钟。普通版没有新增施法 clip，不进入此同步分支。

以下为重新录制的真实游戏窗口，目标录制率60fps、1280×900设备像素；源录像为可变帧率（VFR），测量与导图共用显式60Hz采样流；通过原控制台驱动同一个 Session，同名 JSONL 为实际事件。同步由人物图集和特效命中图的录像像素匹配定位，未把控制台发送时刻当作命中时刻。

| 觉醒场景 | 施法至清除事件 | 人物第8帧 / 特效命中采样编号（60Hz，从0开始） | 差值 |
| --- | --- | --- | --- |
| 落雷 | 0.915s | 115 / 115 | 0帧 |
| 陨石 | 0.878s | 112 / 112 | 0帧 |
| 延迟陨石 | 1.459s | 152 / 152 | 0帧 |

[逐帧测量](recording-measurements.json)及三张 `*-synchronization.webp` 可复核。终局胜利面板遮挡特效，不计入像素同步测量；回滚脚本断言完整快照恢复。

## 验证与复现

基线为 `500673e2326acffb7ff0fbf971607f3c55ae2b53`。录制时 HEAD 尚为 `6961e8d`，实现改动在工作树；`current/capture.json` 的 HEAD 不是本版源码提交。最终实现、捕获二进制与源文件哈希由 `source-binding.json` 绑定。

- 普通不变检查：[GREEN](validation/normal-unchanged-green.json) 全部通过；同一个检查在误改普通版的 `77d5769` 上 [RED](validation/normal-unchanged-red.json) 正确失败。WebP 与缩略图按字节比较，JSON 和 Rust 注册配置忽略 CRLF/LF 差异。
- Rust 357 通过、1 个音频设备测试忽略；Python 18 通过。新增测试锁定普通原配置、觉醒独立姿势、细像素密度、透明度、图集和脚底边界。
- Headless 134 事件与 2 局 autoplay 通过；控制台、角色及能力事件三组窗口回归通过。
- 完整素材重建后 `assets/` 哈希变化为 0；Windows release、WASM、Windows 包及网页包构建通过。macOS CI已通过；结果与独立审查见[交付记录](independent-review.md)。
- 日志位于 [validation](validation/)。Windows 脚本使用 `.build-tools/python-path.txt` 指定 Python，`PYTHONUTF8=1`。

复现顺序：`tools/check_wizard_normal_unchanged.py`；`tools/prepare_assets.py`；`tools/build.py`；设置 `BATTLEBLOCKS_BIN=target/debug/battleblocks.exe` 后运行 headless 与三组 GUI 检查；`tools/capture_wizard_evidence.py --binary target/debug/battleblocks.exe --label current`；`tools/check_wizard_recordings.py`；`tools/review_wizard_animation.py`。录制脚本拒绝把不同二进制的录像混进同一目录；重新录制不同构建时先将旧证据归档到仓库外。

本目录只保留最终版本。`baseline/` 是原 main 的必要对比输入，不是另一版实现；被替代的 v1/v2 验收与生产源稿已移出工作树，可通过 Git 历史追溯。后续六人可复用“普通不变检查、仅觉醒接入、脚底注册、关键帧与真实命中时钟分离、同尺寸与实战双重验收”的方式，按各自技能指定命中关键帧。
