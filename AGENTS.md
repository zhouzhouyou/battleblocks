# BattleBlocks 开发约定

修改、生成或移动文件前，必须阅读并遵从 [目录约定](docs/LAYOUT.md)。目录职责与入库边界以该文档为准，目录变化同步更新相关文档和检查规则。

## 必须遵守

- Headless `Session` 是权威游戏模型，GUI、预览与 AI 共用规则。角色能力通过同步、有类型的事件处理器注册，通用流程不得按角色或技能 ID 分支。状态必须支持快照与回滚。
- 玩家文案全部从 `config/` 读取，同时维护中英文和占位符；禁止在实现代码中硬编码或拼接文案。具体约束见 [文案规范](docs/SKILL-RULES.md#文案维护)。
- 美术统一采用高细节像素插画。旧角色普通版的素材、帧序、时长、尺寸与施法表现保持原样，只优化觉醒待机和施法，各至少十二个独立姿势。
- 角色设计以当前档案及用户明确决定为准，生成图中的偏差标为待修；未获实现授权的评审只记录设定和意见。
- 本地候选、参考原图、预览、日志和检测输出放 `.local/` 或临时目录。禁止产物不得进入当前树或 Git 历史；docs 只保留持续有效的规则与设计依据。

## 按任务查阅

| 任务 | 必读文档 |
| --- | --- |
| 战斗、技能、状态、AI、回滚 | [代码结构](docs/ARCHITECTURE.md)、[技能规则](docs/SKILL-RULES.md) |
| 构建、验证、发布 | [构建与验证](docs/BUILDING.md)、[调试操作](docs/DEBUGGING.md) |
| 平衡评估 | [统计方法与工具](docs/BALANCE-EVALUATION.md)，结果只保存在本地 |
| 美术与角色制作 | [全局美术方向](docs/art/ART-DIRECTION.md)、[角色评审与档案索引](docs/art/CHARACTER-REVIEW.md) |
| 图片导出 | [无损 WebP 流程](docs/art/WEBP-EXPORT.md)，发布资源必须通过 `tools/optimize_webp.py --check` |

## 交付检查

- 改收益逻辑必须补测试，覆盖实际规则、消耗、上限、目标差异和预览无副作用。
- 验证按改动范围选择，统一遵从 [验证选择](docs/BUILDING.md#验证选择)。无关对战的改动不运行对战测试；角色平衡只评估涉及受影响角色的配对；普通 bug 修复只做回归，造成明显强弱变化时才增加平衡评估。窗口检查只用于输入、绘制、动画、音频或窗口交互改动。
- 游戏构建使用 `.build-tools/python-path.txt` 指定的 Python 运行 `tools/build.py`，完成后更新 macOS 与网页包。具体命令见构建文档。
- 提交前运行 `python3 tools/check_repository.py`。启用 `git config --local core.hooksPath .githooks`，不得绕过暂存区及完整历史检查。
