# 可复现的角色平衡评估

本工具通过真实对局评估角色平衡，提供：总预算、稀疏采样、统一评分、不确定性、先手检测、增量复用、克制关系与基准比较。技能 EV、训练状态价值函数、修改策略 AI 和自动调参属于后续研究，不包含在此工具中。

## 运行

仅在角色平衡调整，或 bug 修复导致角色明显强弱变化时运行强度评估。普通 bug 修复按 [验证选择](BUILDING.md#验证选择) 做回归。

游戏规则或配置变化后，先用 `.build-tools/python-path.txt` 指定的 Python 执行 `tools/build.py` 更新评估程序。下文 `python` 均指该解释器。

```sh
python tools/evaluate_balance.py --budget 400 --seed 1200000 --level 1
python tools/evaluate_balance.py --budget 800 --seed 1200000 --level 1
python tools/evaluate_balance.py --budget 400 --seed 1200000 --level 2 --out .local/analysis/evaluation-advanced
python tools/evaluate_balance.py --roster all --budget 800 --out .local/analysis/evaluation-all
```

单个或多个角色调整时，用 `--roster` 指定对手范围，`--focus` 指定受影响角色。保留完整对手名单也只会模拟至少包含一名受影响角色的配对，双方先手仍等权。

```sh
python tools/evaluate_balance.py --roster all --focus 8 --budget 400 --out .local/analysis/evaluation-dwarf
python tools/evaluate_balance.py --roster all --focus 8,7 --budget 400 --out .local/analysis/evaluation-dwarf-knight
```

`--focus` 必须是 `--roster` 的子集，省略时保留全名单采样行为。预算及缓存命中仅计算选中配对，无关角色之间的历史缓存不计入本次报告，也不追加对局。相同配对在全量和定向评估间复用缓存，仍受版本、种子、难度与步数上限约束。报告记录 focus，未采样配对保留未知区间，综合评分与排名包含模型预测。定向结果主要检查受影响角色及其对手关系，不据此确认其他角色整体平衡。

默认新五人 `8,7,9,10,11`，支持任意至少两个不同角色。对手等权、双方先手等权；标准与进阶数据隔离。`--budget` 是当前条件下的**累计对局上限，包含缓存命中**，必须是正偶数；一个采样单元是同种子交换双方位置的两局。首次执行最多新跑 budget 局，再次使用相同预算不追加模拟，增大预算才追加。缓存已有对局超过预算时拒绝运行，不悄悄筛选历史结果。

`--workers` 默认 4，用多个独立进程提速；`--batch-size` 默认 8 个种子对，是采样决策更新频率，不随 worker 数变化。相同输入任务结果可复现。缓存恢复会保留现有证据并重新调度余下预算；中断／扩预算恢复的采样顺序不保证等于一次性运行。所有实际派发种子均有记录。`--max-steps` 默认 20000，超过步数的局记录为 aborted，报告写出后 CLI 退出 2；进程异常/超时则停止，之前已提交的缓存可继续使用。

现有 `--headless --autoplay` 参数、默认名单及 CSV 完全保留。

## 输出与定义

默认输出 `.local/analysis/evaluation/`：

- `report.json`：机器可读配置、编译版本、评分、区间、配对矩阵（每个无序配对一行，反向取 1-p）、先手统计、候选与确认循环。
- `report.md`：便于阅读的中文报告及角色标记原因。
- `matches.jsonl`：每个种子对的两局原始结果，包含种子、位置、难度、步数限制、胜负、终止原因、回合数、剩余生命，以及配对版本键。
- `.local/analysis/evaluation-cache.sqlite`：持久化原始种子对，事务提交，版本冲突时保留旧记录。原始数据不因模型或报告阈值改变而失效。

统一度量为**得分率 = 胜率 + 50% × 平局率**，无平局时等于胜率。超步数不当作平局；任一方向中断的种子对不参与模型拟合，该配对实测区间退回 `[0,1]`，自动判定保持保守。未完成的模拟仍消耗预算。评分反映指定 AI 与正常开局分布下的表现，不代表真人操作上限。

角色评分采用正则化 Bradley–Terry：`sigmoid(能力A - 能力B + 先手项)`。平均评分固定 1500，400 分对应通常的 log-odds 尺度；正则化避免全胜或不连通时发散。平局按半分拟合预期得分。两方向作为一个相关簇，似然按 50% 温度缩放，评分区间标注为**条件 Laplace 近似**；它不代表任意克制关系下的可信度，不用于判定失衡。

配对预测以 8 个种子对的固定权重向 BT 基线收缩，随着实测增多逐步还原局部克制。报告同时保存 BT 原始预测、实测均值和残差，避免模型掩盖克制。

正式实测区间使用 Beta(1/2,1/2) 混合检验构造的置信序列：每个种子对均值是一个 `[0,1]` 样本，按所有配对的得分和先手两个序列分配总 alpha=0.05。对于分数样本，Bernoulli 凸性上界保证检验超鞅有效；Ville 不等式支持自适应选择配对和任意停止。前提是新种子代表独立抽样，规则与策略固定。数学形式：

`log E(p) = log B(S+1/2,n-S+1/2) - log B(1/2,1/2) - S log p - (n-S) log(1-p)`。

保留 `E(p) <= 1/alpha_pair` 的 p。不要把两局当两个独立种子，也不要把模型区间当这个置信序列。角色对手等权的区间由所有配对同时区间取平均，未测关系使用 `[0,1]`。

默认综合目标 45%–55%，`--lower/--upper` 可配置。只有整个实测区间高于上界／低于下界，才标记过强／过弱；落在目标内标 balanced，其余 needs_samples。`--extreme .75` 判定极端配对；点估计越界只叫 suspected，区间完全越界才叫 confirmed。`--width .20` 是 verified 标签要求，不是承诺预算内一定收敛。局部显著胜率超过 50% 形成有向边，输出三角色循环；候选循环与确认循环分开，循环不会被自动判为整体失衡。

## 采样与复杂度

先随机排列角色并覆盖连通环，初始关系数 O(N)；随后 25% 槽位做最低覆盖探索，其余按追加一个种子对的启发式误差下降近似分配。优先级是 `v/((n+1)(n+2)) + 64r²[(n+8)⁻²-(n+9)⁻²]`：v 是配对均值的平滑实测方差，r 是实测与 BT 的残差，8 与预测收缩权重一致；两个先验观测避免短样本零方差导致停止探索。噪声项近似 `v/(n+1)` 的下降，并非整个收缩估计器真实 MSE 的精确下降；BT 参数更新及模型失配也使它只能作为启发式。这样既补充不确定关系，也避免对已经精确的大残差配对持续过采样。`--sampler uniform` 用同样的连通初始化，其后均匀追加，便于公平对照。

模拟量严格受预算 B 约束。设置 B=O(NK) 即可避免全量 O(SN²) 对局，但不保证恢复所有未测克制。当前离线实现仍枚举 O(N²) 候选／输出，稠密评分线性系统约 O(N³)，三角循环枚举 O(N³)；不把模拟复杂度宣传成整个程序复杂度。更大角色库可另行替换候选索引与稀疏求解器。小名单应优先完整覆盖，预算不足时报告会明确保留未知项。

## 版本与增量

`build.rs` 调用 `tools/balance_fingerprint.py`，将 SHA-256 清单编译进可执行程序，`--headless --evaluation-manifest` 导出。版本来自实际程序，不读取运行目录源码来假装版本一致。

- 所有 Rust 源码、技能配置、Cargo 配置／锁文件、构建指纹生成器及 target 进入共享指纹；共享规则、AI、随机机制、技能逻辑或工具链目标变化保守失效全部对局。
- `heroes.txt` 与 `ability-loadouts.txt` 按稳定角色 ID 分行进入角色指纹。只改角色 HP/MP/技能配置时，旧角色之间不受影响的配对可复用；复制技能也只在真实对局双方之间发生。
- 新增角色名单可以复用已有角色配对。如果新增角色需要修改 Rust 源码，本版本会保守使全局失效；不声称能自动推导任意代码修改的依赖闭包。
- 缓存键还包含实验根种子、难度、步数上限；实际种子从根种子、无序配对和样本序号用 SHA-256 派生。同一版本、种子返回冲突结果时立即报错，不覆盖历史记录。

## 基准与验收

先采集同一程序版本、同一难度下的两个独立种子全集，再做预算重放：

```sh
python tools/evaluate_balance.py --sampler uniform --budget 1280 --seed 121001 --out .local/analysis/evaluation-runs/train
python tools/evaluate_balance.py --sampler uniform --budget 2560 --seed 122001 --out .local/analysis/evaluation-runs/reference
python tools/benchmark_balance.py --training .local/analysis/evaluation-runs/train --reference .local/analysis/evaluation-runs/reference --budgets 100,200,400,800 --repetitions 12 --out .local/analysis/evaluation-benchmark.json
```

基准保留三个明确不同的比较列：`uniform_raw_empirical_rmse` 是原有均匀采样＋原始配对得分率；`uniform_rmse` 是均匀采样＋新收缩估计；`adaptive_rmse` 是自适应采样＋新收缩估计。第一列与第三列比较整个系统的收益；第二列与第三列使用**相同估计器、训练池与总预算**，才隔离采样策略的贡献。不能把收缩估计带来的优势归功于自适应采样。

参考种子严格不重叠，全部配对必须有数据，不能包含中断；导入逐条校验编译版本键、英雄位置、派生种子、难度及步数上限，支持 gzip 压缩的 JSONL 证据。训练池重排重复用于观察采样稳定性，重复之间复用同一有限数据池，因此重放标准误仅为描述性结果。参考胜率本身仍有误差，不能称为真实概率。

比较配对 RMSE、留出得分 Brier、综合表现排名相关性、候选极端克制精确率／召回率、正式确认数与区间覆盖参考均值的比例。报告各预算，不只挑最好一个。实际效果以本地独立种子评估结果为准，不保证所有预算、所有阵容均优于均匀采样。统计测试另用已知概率验证区间，不拿参考均值覆盖率冒充真实概率覆盖率。

验证按 [构建与验证](BUILDING.md#验证选择) 选择。统计、调度与缓存修改运行 `python tools/test_balance.py`；真实进程协议、CLI 集成修改再运行 `python tools/check_balance.py`。Rust 战斗规则变化才增加相关 Rust 与 headless 回归。

## 参考

- [Bradley–Terry 模型、顺序优势与残差诊断](https://www.jstatsoft.org/article/view/09)
- [Anytime-valid confidence sequences](https://arxiv.org/abs/1810.08240)
- [Issue #12 与技能 EV 后续研究](https://github.com/zhouzhouyou/battleblocks/issues/12)
