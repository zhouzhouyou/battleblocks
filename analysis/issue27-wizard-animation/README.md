# Issue #27 老巫师动画验收

基线：最新 `origin/main` **500673e2326acffb7ff0fbf971607f3c55ae2b53**（PR #26合并），从该提交创建 `codex/27-wizard-animation`。调研阅读了 #27、#11 最新讨论、PR #26实现与APPROVED评审，以及本机 Nowledge 项目记忆；以当前代码复核历史笔记中的帧数、解码与量化问题。用户批准推荐方案并明确 Draft PR 使用 `Closes #27`、`Refs #11`。

## 实现与现状更正

旧普通图为4帧半身像，觉醒为单帧全身图，没有独立施法。现为普通待机、觉醒待机、普通施法、觉醒施法各12张独立姿势，共48张。保留蓝紫帽、白胡须、蓝袍，普通补齐鞋与木杖；觉醒以站姿和蓝晶亮度区别，不增加装备。四组共享64色和比例，按鞋底而非手杖包围盒注册。

入口：选角 `src/menu.rs` 读取普通／觉醒idle；战斗 `src/app/view/hud.rs` 选择cast或idle。选角技能小棋盘仍展示原有技能预览。施法第5帧在0.32s释放，第7帧引导，第8帧匹配真实命中，随后0.5s收势。落雷、陨石当前名义命中均为0.9s，不是两套固定出手速度；差别包括清除后的落块等待0.85/1.25s与余效，整段特效约2.8s。模型收益在施法调用时结算，视觉清除发生较晚，陨石触发对手觉醒还会延迟；没有修改这些规则。

人物读取与技能特效相同的 `ImpactClock`，等待时停在第7帧，可越过旧1.4s倒计时；真实清除后进入第8帧。通用 `AnimationClip.impact` 支持同一动作适配不同名义命中时点，收势时长不变，已有测试覆盖0.4/0.9/1.8s。终局没有清除事件则按墙钟完成；回滚清空表现时钟。后续六人可复用动作语义、脚底注册、共同比例／色板、相同纹理解码与实际尺寸/事件录像检查；是否接入命中时钟按技能语义决定。

## 可直接查看的证据

- [普通同尺寸前后对比](normal-menu-comparison.webp)、[觉醒同尺寸前后对比](awake-menu-comparison.webp)：真实窗口、1:1设备像素裁剪，左右基线／当前；不是重新排版模拟。
- [实际图集48姿势](atlas-contact.webp)、[11→12→1→2脚底接缝](idle-seams.webp)：后者为运行时图集按实际选角比例采样诊断，区别于真实录像。
- [交互检查页](index.html)：本地提供角色帧、时长、延迟命中滑杆及下列视频。用仓库根目录HTTP服务打开即可。
- 普通 [落雷](current/normal-lightning.mp4) / [陨石](current/normal-meteor.mp4)，觉醒 [落雷](current/awake-lightning.mp4) / [陨石](current/awake-meteor.mp4)。
- [延迟陨石](current/delayed-meteor.mp4)、[终局陨石](current/terminal-meteor.mp4)、[施法中回滚](current/rollback.mp4)。
- [普通选角循环](current/normal-menu.mp4)、[觉醒选角循环](current/awake-menu.mp4)；`baseline/` 保留原始前对比录像。

原生窗口录制为60fps、1280×900（125%DPI，1024×720逻辑画面）；FFmpeg gdigrab裁掉窗口捕获的黑色扩展区，不缩放。场景由现有stdin控制台设置固定棋盘、种子和资源，施法、觉醒和回滚全部运行真实 Session。每段6秒，旁边同名JSONL是实际事件。`tools/capture_wizard_evidence.py` 可复录；`tools/review_wizard_animation.py` 导出画面；`tools/check_wizard_recordings.py` 从录像像素匹配实际人物图集与特效命中图，并输出 [逐帧数据](recording-measurements.json)。评审脚本另需 numpy2.4、imageio-ffmpeg0.6、Pillow13，仅本地工具缓存，不进入游戏依赖。

| 场景 | 施法事件到清除事件 | 人物第8帧 / 特效命中视频帧（从0开始） | 差值 |
| --- | --- | --- | --- |
| 普通落雷 | 0.879s | 112 / 112 | 0帧 |
| 普通陨石 | 0.892s | 115 / 115 | 0帧 |
| 觉醒落雷 | 0.904s | 115 / 115 | 0帧 |
| 觉醒陨石 | 0.895s | 114 / 114 | 0帧 |
| 对手觉醒延迟陨石 | 1.490s | 149 / 149 | 0帧 |

`*-synchronization.webp` 是命中前1帧、命中帧、后1/6帧的完整游戏画面，供同时看人物与棋盘。帧匹配受60fps录制采样限制，不声称亚帧精度。终局胜利面板遮挡特效，故不作为像素同步测量；录像显示人物正常收势回待机，单位测试验证没有清除通知也不会卡住。回滚脚本断言完整快照等于施法前状态；选角首尾与脚线人工检查未见跳动。

`capture.json` 中的 `git_head` 是录制时尚未提交改动的基线HEAD，不能当作新版源码提交；二进制SHA256才绑定实际录制文件：基线 `680ff9d5945784a2ef1e2af9eb811de2cb0d2f7c7e7f97f09d16cb2a79e2fdfd`，当前 `65eaecc5f8d81c839d40679ddd0645abb8e9f45254efa14bf7e9844f4a23bbea`。源码归属与交付提交见后续审查记录。

## 验证

基线 cargo test：354通过、1音频设备测试忽略；headless 134事件及2局autoplay通过。

当前 `.build-tools/python-path.txt` 指定Python运行 `tools/build.py`：Rust356通过、1音频设备测试忽略；Python平衡测试18通过；Windows release、WASM、Windows zip和网页zip均生成。`tools/check_headless.py` 134事件及2局autoplay通过；`check_debug_console.py` 314事件、`check_new_roles.py` 196事件、`check_ability_events.py` 事件链和完整回滚均通过。实测网页WASM普通／觉醒选角页正常显示并播放，无浏览器warning/error。

完整 `tools/prepare_assets.py` 重建通过，前后所有 assets 文件SHA256一致（变化0），其他五位角色图集未改变。macOS构建与独立审查结果在交付前补齐。待用户最终看：普通全身造型与旧半身像的差异、脸部浅色块与细密衣褶、觉醒差异是否足够明确、施法下压力度与循环节奏。角色介绍原文仍待补充，不影响已授权动画实现；主 Issue #11保留。
