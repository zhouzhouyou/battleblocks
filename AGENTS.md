# BattleBlocks 开发约定

## 验证与交付

- 改收益逻辑必须补测试：实际规则、消耗、资源上限、目标差异、预览不改变状态。
- 构建用 `.build-tools/python-path.txt` 指定的 Python 运行 `tools/build.py`；完成后更新 macOS 与网页包。
- 战斗、日志或回滚逻辑变更后，运行 `tools/check_debug_console.py` 验证真实游戏流程。
