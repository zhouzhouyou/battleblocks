# BattleBlocks 角色平衡评估

Strength is conditional on the specified AI, normal starting-state distribution, roster and executable version.

expected score = P(win) + 0.5 P(draw); aborted games are not draws. Any interrupted pair receives [0,1] empirical bounds.

Empirical intervals: simultaneous 95% anytime-valid paired-seed confidence sequences; rating intervals: conditional Laplace approximations, not used for balance flags. Predictions on unobserved edges are model assumptions, not validation.

对局 1280；新跑 0；复用 1280；中断 0；配对覆盖 100.0%。

| 角色 | 评分（近似 95% 区间） | 综合预测得分率 | 实测同时置信序列 | 状态 |
|---|---:|---:|---|---|
| Knight | 1531 (1496–1565) | 55.5% | 31.2%–78.0% | needs_samples |
| Dwarf | 1510 (1476–1544) | 51.8% | 28.0%–74.9% | needs_samples |
| Elf Fire Mage | 1504 (1470–1538) | 50.8% | 27.7%–73.7% | needs_samples |
| Nun | 1486 (1452–1520) | 47.5% | 26.0%–69.8% | needs_samples |
| Priest | 1469 (1435–1504) | 44.5% | 22.9%–67.9% | needs_samples |

## 配对：实测、模型与先手

| A / B | 局数 | 实测得分率 | 预测 | 实测区间 | 先手得分率 | 状态 / 极端关系 |
|---|---:|---:|---:|---|---:|---|
| 7 / 8 | 128 | 42.2% | 43.4% | 19.8%–67.1% | 60.9% | uncertain  |
| 7 / 9 | 128 | 57.0% | 57.2% | 32.2%–79.6% | 57.0% | uncertain  |
| 7 / 10 | 128 | 56.2% | 56.3% | 31.5%–79.0% | 60.9% | uncertain  |
| 7 / 11 | 128 | 66.4% | 65.0% | 41.1%–86.4% | 52.3% | uncertain  |
| 8 / 9 | 128 | 40.6% | 42.3% | 18.6%–65.6% | 56.2% | uncertain  |
| 8 / 10 | 128 | 64.8% | 63.6% | 39.6%–85.4% | 50.8% | uncertain  |
| 8 / 11 | 128 | 43.8% | 44.5% | 21.0%–68.5% | 45.3% | uncertain  |
| 9 / 10 | 128 | 18.8% | 22.0% | 4.7%–42.7% | 51.6% | uncertain suspected extreme |
| 9 / 11 | 128 | 57.0% | 55.7% | 32.2%–79.6% | 52.3% | uncertain  |
| 10 / 11 | 128 | 29.7% | 31.7% | 11.0%–54.9% | 50.0% | uncertain  |

确认循环：[]
候选循环（证据尚不足）：[[7, 9, 8], [7, 11, 8], [8, 10, 9], [9, 11, 10]]

## 标记原因

- Knight: uniform-opponent score bounds [0.312, 0.780], target [0.450, 0.550]
- Dwarf: uniform-opponent score bounds [0.280, 0.749], target [0.450, 0.550]
- Elf Fire Mage: uniform-opponent score bounds [0.277, 0.737], target [0.450, 0.550]
- Nun: uniform-opponent score bounds [0.260, 0.698], target [0.450, 0.550]
- Priest: uniform-opponent score bounds [0.229, 0.679], target [0.450, 0.550]
