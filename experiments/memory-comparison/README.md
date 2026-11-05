# 内存对照实验

隔离比较原生 Cocoa、Miniquad、Macroquad、字体加载、音乐播放及渲染缓冲。各工程的 Cargo.lock 固定依赖；本目录 assets 仅保留字体输入，音乐直接引用仓库 `assets/audio/runtime/bgm.ogg`；不复制游戏资源。

从仓库根目录运行 `python3 experiments/memory-comparison/build.py` 构建，再运行 `python3 experiments/memory-comparison/run.py`；可用 `--normalized`、`--isolate`、`--buffers`、`--font-cache`、`--font-frame` 选择场景。`stability.py` 检查持续运行。实际采样需要 macOS 窗口与 ps、vmmap，单次数字受主机和显示环境影响。

本实验不模拟战斗规则。结果写入已忽略的 results 目录；日志、采样数据、单次审计及报告不入库。目录职责见 [LAYOUT](../../docs/LAYOUT.md)。
