# Memory comparison

独立 Release 基准：原生 Cocoa、Miniquad、Macroquad（音频关闭/开启）、字体/音乐逐项加载、渲染后端与真实游戏。

完整方法、数字、异常及限制见 REPORT.md。源码不复用游戏构建配置；四份 Cargo.lock 固定依赖。`python3 analysis/memory-comparison/build.py` 构建，`run.py` 各阶段采样。数据位于 data/；原始 vmmap 输出在忽略的 results* 目录。
