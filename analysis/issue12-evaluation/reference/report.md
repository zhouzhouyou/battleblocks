# BattleBlocks 角色平衡评估

Strength is conditional on the specified AI, normal starting-state distribution, roster and executable version.

expected score = P(win) + 0.5 P(draw); aborted games are not draws. Any interrupted pair receives [0,1] empirical bounds.

Empirical intervals: simultaneous 95% anytime-valid paired-seed confidence sequences; rating intervals: conditional Laplace approximations, not used for balance flags. Predictions on unobserved edges are model assumptions, not validation.

对局 2560；新跑 752；复用 1808；中断 0；配对覆盖 100.0%。

| 角色 | 评分（近似 95% 区间） | 综合预测得分率 | 实测同时置信序列 | 状态 |
|---|---:|---:|---|---|
| Knight | 1513 (1489–1537) | 52.3% | 34.5%–69.7% | needs_samples |
| Nun | 1513 (1488–1537) | 52.2% | 35.5%–68.6% | needs_samples |
| Elf Fire Mage | 1508 (1484–1532) | 51.5% | 34.0%–68.7% | needs_samples |
| Dwarf | 1499 (1475–1524) | 49.9% | 32.4%–67.4% | needs_samples |
| Priest | 1467 (1442–1491) | 44.0% | 27.9%–61.2% | needs_samples |

## 配对：实测、模型与先手

| A / B | 局数 | 实测得分率 | 预测 | 实测区间 | 先手得分率 | 状态 / 极端关系 |
|---|---:|---:|---:|---|---:|---|
| 7 / 8 | 256 | 41.0% | 41.7% | 24.3%–59.3% | 56.6% | uncertain  |
| 7 / 9 | 256 | 56.2% | 56.3% | 38.1%–73.3% | 53.1% | uncertain  |
| 7 / 10 | 256 | 54.3% | 54.0% | 36.2%–71.6% | 54.3% | uncertain  |
| 7 / 11 | 256 | 57.8% | 57.4% | 39.6%–74.7% | 60.2% | uncertain  |
| 8 / 9 | 256 | 45.3% | 45.9% | 28.0%–63.4% | 50.8% | uncertain  |
| 8 / 10 | 256 | 57.0% | 56.5% | 38.8%–74.0% | 59.4% | uncertain  |
| 8 / 11 | 256 | 38.3% | 38.9% | 22.0%–56.6% | 56.2% | uncertain  |
| 9 / 10 | 256 | 17.6% | 19.1% | 6.7%–34.1% | 56.6% | uncertain suspected extreme |
| 9 / 11 | 256 | 60.2% | 59.2% | 41.9%–76.7% | 51.6% | uncertain  |
| 10 / 11 | 256 | 37.9% | 38.6% | 21.7%–56.2% | 48.8% | uncertain  |

确认循环：[]
候选循环（证据尚不足）：[[7, 9, 8], [7, 11, 8], [8, 10, 9], [9, 11, 10]]

## 标记原因

- Knight: uniform-opponent score bounds [0.345, 0.697], target [0.450, 0.550]
- Nun: uniform-opponent score bounds [0.355, 0.686], target [0.450, 0.550]
- Elf Fire Mage: uniform-opponent score bounds [0.340, 0.687], target [0.450, 0.550]
- Dwarf: uniform-opponent score bounds [0.324, 0.674], target [0.450, 0.550]
- Priest: uniform-opponent score bounds [0.279, 0.612], target [0.450, 0.550]
