# BattleBlocks 角色平衡评估

Strength is conditional on the specified AI, normal starting-state distribution, roster and executable version.

expected score = P(win) + 0.5 P(draw); aborted games are not draws. Any interrupted pair receives [0,1] empirical bounds.

Empirical intervals: simultaneous 95% anytime-valid paired-seed confidence sequences; rating intervals: conditional Laplace approximations, not used for balance flags. Predictions on unobserved edges are model assumptions, not validation.

对局 2560；新跑 0；复用 2560；中断 0；配对覆盖 100.0%。

| 角色 | 评分（近似 95% 区间） | 综合预测得分率 | 实测同时置信序列 | 状态 |
|---|---:|---:|---|---|
| Elf Fire Mage | 1528 (1504–1552) | 55.0% | 37.3%–71.8% | needs_samples |
| Dwarf | 1515 (1491–1539) | 52.6% | 35.0%–69.8% | needs_samples |
| Knight | 1501 (1477–1525) | 50.2% | 32.6%–67.8% | needs_samples |
| Nun | 1491 (1467–1515) | 48.4% | 32.2%–65.0% | needs_samples |
| Priest | 1465 (1441–1489) | 43.8% | 27.4%–61.2% | needs_samples |

## 配对：实测、模型与先手

| A / B | 局数 | 实测得分率 | 预测 | 实测区间 | 先手得分率 | 状态 / 极端关系 |
|---|---:|---:|---:|---|---:|---|
| 7 / 8 | 256 | 40.2% | 40.7% | 23.6%–58.5% | 51.2% | uncertain  |
| 7 / 9 | 256 | 50.0% | 50.3% | 32.2%–67.8% | 48.4% | uncertain  |
| 7 / 10 | 256 | 52.0% | 51.9% | 34.0%–69.5% | 56.6% | uncertain  |
| 7 / 11 | 256 | 58.6% | 57.9% | 40.3%–75.4% | 54.7% | uncertain  |
| 8 / 9 | 256 | 48.0% | 48.6% | 30.5%–66.0% | 53.5% | uncertain  |
| 8 / 10 | 256 | 63.3% | 62.7% | 45.0%–79.3% | 55.5% | uncertain  |
| 8 / 11 | 256 | 39.5% | 40.0% | 23.0%–57.8% | 51.2% | uncertain  |
| 9 / 10 | 256 | 20.3% | 21.8% | 8.5%–37.3% | 50.8% | uncertain suspected extreme |
| 9 / 11 | 256 | 52.7% | 52.0% | 34.8%–70.2% | 54.3% | uncertain  |
| 10 / 11 | 256 | 29.3% | 30.2% | 14.9%–47.3% | 51.2% | uncertain  |

确认循环：[]
候选循环（证据尚不足）：[[7, 11, 8], [8, 10, 9], [9, 11, 10]]

## 标记原因

- Elf Fire Mage: uniform-opponent score bounds [0.373, 0.718], target [0.450, 0.550]
- Dwarf: uniform-opponent score bounds [0.350, 0.698], target [0.450, 0.550]
- Knight: uniform-opponent score bounds [0.326, 0.678], target [0.450, 0.550]
- Nun: uniform-opponent score bounds [0.322, 0.650], target [0.450, 0.550]
- Priest: uniform-opponent score bounds [0.274, 0.612], target [0.450, 0.550]
