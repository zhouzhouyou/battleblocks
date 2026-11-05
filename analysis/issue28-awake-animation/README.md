# Issue #28：三位旧角色觉醒动画

仅优化旧骑士、精灵、哥布林的觉醒待机/施法，各12独立姿势，共72帧。普通素材、帧数、顺序、时长、缩略图、显示尺寸和既有施法表现保持原样。用户2026-11-06要求精简文件、推送并转Ready；Ready不等于最终视觉验收，主Issue #11保持打开。

## 版本与复现

- 开工基线：`c0aa438fa4077785706a26276795efbb93e886d6`，当时最新origin/main，含#31。
- 主实现：`c6e5b9f81e4b62aed203152c9bc115641fabae4a`，对应[source-binding.json](source-binding.json)。审查后修复：`1cea8b3865a948075be47a6bfb2d15d54ac86cdd`，只调整觉醒坚守护环淡出；最终源文件/二进制/包哈希见[review-fix/source-binding.json](review-fix/source-binding.json)。后续清理不改变运行时或生产素材。
- [批准方案](PLAN.md) · [独立审查及修复复核](INDEPENDENT-REVIEW.md) · [角色评审索引](../../docs/art/CHARACTER-REVIEW.md)。三人介绍沿用既有游戏文案，原始用户设定来源待补。
- 源图、实际完整提示词、注册参数、64色色板保留在`asset-sources/art-direction/characters/{knight-legacy,elf-legacy,goblin-legacy}/production/v1`。图像由内置imagegen生成；旧骑士施法最终源表v3，其他源表v1。384×384帧、2304×1536图集、二值alpha、最近邻、无损WebP，脚锚(192,336)。
- 用`.build-tools/python-path.txt`指定的Python运行`tools/prepare_character_animations.py --role knight-legacy`（也支持elf-legacy/goblin-legacy）及`--registry-only`，或完整`tools/prepare_assets.py`。生成器不写普通配置或手写播放逻辑；[两次全量重建哈希](rebuild.json)一致。

## 精选视觉证据

实际1280×900游戏窗口裁图，未二次缩放：

| 角色 | 觉醒选角前后 | 觉醒战斗前后 | 普通选角前后 | 24姿势 / 接缝 |
|---|---|---|---|---|
| 旧骑士 | [对比](knight-awake-menu-before-after.webp) | [对比](knight-awake-battle-before-after.webp) | [对比](knight-normal-menu-before-after.webp) | [姿势](art/knight-legacy-poses.webp) / [接缝](art/knight-legacy-seam.webp) |
| 精灵 | [对比](elf-awake-menu-before-after.webp) | [对比](elf-awake-battle-before-after.webp) | [对比](elf-normal-menu-before-after.webp) | [姿势](art/elf-legacy-poses.webp) / [接缝](art/elf-legacy-seam.webp) |
| 哥布林 | [对比](goblin-awake-menu-before-after.webp) | [对比](goblin-awake-battle-before-after.webp) | [对比](goblin-normal-menu-before-after.webp) | [姿势](art/goblin-legacy-poses.webp) / [接缝](art/goblin-legacy-seam.webp) |

![实际选角：旧骑士、新骑士、精灵、哥布林](awake-menu-lineup.webp)

旧骑士青蓝方盔/橙缨/单剑与新骑士黑红金甲/长辫/石剑可区分；精灵施法空手与弓臂、哥布林收拢空手与匕首分离。装备、脚线及待机11→12→1→2接缝已观察。普通截图待机相位可能不同，字节/逐帧一致由[普通保护结果](normal-check.json)与[各角色基线锁](research/normal-lock.json)证明。

| 技能 | 精选实战 | 播放依据 |
|---|---|---|
| 冲锋 | [录像](final/knight-awake-charge.mp4) | 1–6名义0.95秒，6等待真实伤害；7命中，0.45秒收势 |
| 坚守 | [修复后录像](review-fix/knight-awake-guard.mp4) | 治疗即时；6等待本次实际目标清除，7–12收势1秒；空目标不等待 |
| 消除增幅 | [录像](final/elf-awake-strike.mp4) | 状态即时，不结束回合；1.4秒单次，下一行动可接管 |
| 自然转化 | [录像](final/elf-awake-nature.mp4) | 转块/治疗/伤害即时；1.4秒，不因后续连锁重播 |
| 区域掠夺 | [录像](final/goblin-awake-greedy.mp4) | 额外收益即时；6等待初始2×2清除，7开始1秒收束 |
| 魔力换血 | [录像](final/goblin-awake-blood.mp4) | 治疗即时，满血实际0；1.4秒单次 |

另保留[延迟冲锋录像](final/knight-awake-charge-delayed.mp4)、[命中逐帧图](charge-delayed-landmarks.webp)和[护环修复前后](review-fix/guard-fade-before-after.webp)。这7段录像与配套原始JSONL均未改写，[精选清单](selected-captures.json)绑定各自源码、二进制、视频/日志哈希、命令和真实事件。

## 验证结论与限制

**自动检查通过**：普通源图/atlas字节、解码逐帧像素、原序列/时长、缩略图、普通注册及显示helper一致，4类故意破坏被拒绝；[素材检查](art/checks.json)确认72个独立非平移重复姿势、二值透明、≤64色、安全边界及固定脚线。

最终修复版：Rust主程序176、两个probe各93通过（1既有忽略），Python18通过；debug/release headless各134事件；GUI console322、ability events、new roles180事件通过。主实现12局固定seed自动对战CSV与基线逐字节相同。Windows/Web按指定Python运行tools/build.py完成打包。主实现41场景及修复5场景的真实事件断言分别见[原场景摘要](final/scenario-checks.json)、[修复场景摘要](review-fix/scenario-checks.json)；覆盖延迟、终局、回滚、待冲锋恢复、形态切换、AI、空目标、满血、无连锁和后续行动。

**实际观察/测量**：六招玩家无中断主样本均观察到12姿势，敌方AI单段不保证完整12姿势。[测量摘要](measurement-summary.json)从已验证原报告提取，未重写测量。导图和测量统一使用显式60Hz采样流，n对应n/60秒；导出截图与同一采样逐像素核对。延迟冲锋相对模型间隔偏差65.72ms，超过3采样阈值，此项失败保留；实际伤害反馈与关键姿势同在采样113出现。不能将相对间隔失败写成绝对时钟校准通过。

medium独立审查发现1项P2（觉醒坚守护环突消），已新提交修复，并由同一代理独立核对debug/release原录像及哈希，关闭P2，无新增确认问题。原RED记录在归档中保留。最终修复版[网页观察](review-fix/web-observations.json)确认启动、三人觉醒预览及形态页切换，warn/error为空。

macOS [CI42051086270](https://github.com/zhouzhouyou/battleblocks/actions/runs/42051086270)构建、Rust/Python测试、headless和打包通过。PR head749551b1、实际checkout09d08efd与最终运行时1cea8b3的运行时Git差异为空，见[版本绑定](macos/source-binding.json)。[下载包检查](macos/package-check.json)确认ZIP、arm64 Mach-O、0755、Info.plist及无asset-sources；[artifact](https://github.com/zhouzhouyou/battleblocks/actions/runs/42051086270/artifacts/13426403492)保留至2026-11-19，本机dist/macos-arm64.zip已更新。

**待视觉验收/未验证**：材质、呼吸/施法节奏、脚线切换观感仍由用户确认；macOS GUI、完整网页实战矩阵未验证。Ready状态不替代这些结论。

## 完整记录归档

按用户2026-11-06要求，仅保留上述精选证据。重复采样截图、多轮非精选录像、原始逐采样JSON、运行日志、Issue/PR副本和5个本次取证脚本已移出当前树；完整记录仍可在[固定归档提交c3478e1](https://github.com/zhouzhouyou/battleblocks/tree/c3478e146ad4c28485567ee627b2cf1bb4c6e9ef/analysis/issue28-awake-animation)查看，原脚本可从该提交的tools/读取。原独立审查与历史测试仍绑定它们实际检查的提交。

本机另存`.build-tools/issue28-full-evidence-c3478e1.zip`，ZIP完整性校验通过，SHA256 `8c684ca9fd6a50fa45d1805374e6ee48264f1bbae4fd8269d09d83954cebf292`。此次使用普通删除提交，不改写既有提交或审查记录；缩减当前PR文件差异，历史Git对象仍保留。
