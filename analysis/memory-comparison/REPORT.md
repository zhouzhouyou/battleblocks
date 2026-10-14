# 独立最小项目的完整内存对照

日期：2026-10-14。主结论：空窗口约 95 MiB RSS 在本机可以复现，但不能将其解释为游戏独占 95 MiB，也不是 Macroquad 的理论下限。音频和字体的普通加载增量解释不了整个基线；窗口/图形后端的开销及统计口径占了重要部分。另有字体首次绘制与大绘制缓冲区相关的异常高图形分配，需要与基线分开看待。

## 环境与方法

- macOS 27.0.1（26A434），MacBookPro18,1，32 GiB RAM，aarch64；rustc 1.99.0 / LLVM 23.1.1。
- Miniquad 0.4.11、Macroquad 0.4.16、quad-snd 0.2.8。所有 Rust 项目独立 Cargo manifest/lock，Release 使用 opt-level=s、LTO、单 codegen unit、strip、panic=abort；不依赖游戏的 build.rs。
- Cocoa 基准用 clang -O2 编译，只建窗口，不创建 OpenGL/Metal 渲染上下文。Rust 无窗口基准仅休眠。这两个是系统控制组，不等价于具有渲染缓冲区的游戏窗口。
- 独立图形项目窗口配置 1024×720、high_dpi=false、sample_count=1、swap_interval=Some(1)。真实游戏维持原有 swap_interval 默认值，不能用本测试比较帧率或 CPU。
- 每个场景独立进程，启动后 5/10/15 秒记录 RSS，5/15 秒记录 vmmap physical footprint。每个场景两次运行，第二轮倒序，共 26 场景、52 次运行。
- Metal 的归一化补测部分与主组同时进行，其他组内部串行。系统其他程序与驱动缓存未完全固定；几 MiB 的差异不能过度解释。数据保留两次范围，异常双峰不能仅用均值总结。
- 每个进程日志打印实际渲染尺寸和 DPI，而不只相信配置。二进制大小和 SHA-256 随原始数据记录。
- 原始 vmmap 位于 results*（Git 忽略）；可分享的数字及日志位于 data/short-runs.json 和 data/summary.json。

RSS 与 footprint 是不同口径。这里的 Cocoa 空窗口约 74 MiB RSS / 14 MiB footprint，直接说明 RSS 不能等同于进程的独占/计费内存。footprint 也不是某个 Vec 大小的简单求和，涉及图形、压缩及平台计费。不要将两者相加或用相减来精确推算共享页面。

## 1. 独立空窗口与资源增量

| 场景 | RSS 均值 MiB | footprint 均值 MiB | footprint 两次范围 MiB |
| --- | ---: | ---: | ---: |
| Rust 无窗口休眠 | 1.7 | 1.2 | 1.2–1.2 |
| 原生 Cocoa 空窗口 | 74.0 | 13.8 | 13.4–14.2 |
| Miniquad / OpenGL 空窗口 | 81.1 | 50.7 | 46.8–54.6 |
| Macroquad 无音频 / OpenGL 空窗口 | 89.6 | 59.3 | 56.1–62.5 |
| Macroquad 有音频、无文件 / OpenGL | 95.9 | 60.2 | 58.5–62.0 |
| 无音频，仅加载字体，不绘制 | 94.5 | 58.3 | 58.2–58.4 |
| 无音频，加载并绘制文字 | 102.6 | 68.0 | 67.5–68.4 |
| 有音频，仅加载音乐，不播放 | 106.3 | 69.5 | 69.0–70.1 |

依赖树验证：macro-noaudio 没有 quad-snd，macro-audio 确实启用 macroquad/audio。关闭音频的 Macroquad 使用空的 dummy AudioContext。空项目没有显式加载用户字体或图片，但 Macroquad 内部仍会初始化 QuadGl、UI、默认字体及纹理批处理器；“空应用代码”不等于“引擎没有初始化任何资源”。

从控制组看，本机 Cocoa 的窗口基准 footprint 约 14 MiB；增加 OpenGL 渲染上下文/清屏等到约 51 MiB；再加入 Macroquad 初始化到约 59 MiB。这个差分不能被当成每一项精确的独占分配，但足以排除“Rust 本身需要 95 MiB”以及“主要由用户音频字体文件造成”的解释。

仅加载字体的 footprint 没有可靠地高于空窗口（差异落在波动内）；实际绘制字形后增量约 9 MiB。音乐文件只有约 166 KiB，但 native mixer 将整段解码为 f32 并重采样至 44100 Hz 双声道。文件长 12 秒，最终样本本体约 4.04 MiB；解码临时数组、Vec 容量与分配器保留页会增加观察值。静态加载音频后还会保留 AddSound 的 mixer 数据。

## 2. 后端与实际分辨率

| 场景 | RSS 均值 MiB | footprint 均值 MiB | footprint 两次范围 MiB |
| --- | ---: | ---: | ---: |
| Miniquad / Metal（实际 2× 像素） | 83.7 | 72.8 | 65.7–79.9 |
| Macroquad 无音频 / Metal（实际 2× 像素） | 95.0 | 94.2 | 88.9–99.6 |
| Miniquad / Metal，归一化渲染像素 | 83.4 | 39.4 | 35.5–43.2 |
| Macroquad 无音频 / Metal，归一化像素 | 93.0 | 47.5 | 45.1–49.8 |
| Macroquad 有音频 / Metal，归一化像素 | 97.3 | 51.0 | 47.0–54.9 |

本机 Miniquad Metal 即使 high_dpi=false，日志仍报告 2048×1440、DPI=2；OpenGL 报告 1024×720、DPI=1。因此未经归一化的 Metal 不能与 OpenGL直接比较。

归一化组将 Metal 的逻辑窗口设为 512×360，实际渲染仍是 1024×720。它匹配渲染像素数，但逻辑窗口面积不同，所以也不是完全相同的桌面体验。此次 Metal 空窗口 footprint 比 GL 低约数至十 MiB；RSS 并未同比降低。不能直接据此承诺换后端后完整游戏的内存或视觉结果。

## 3. 文字/音频组合与异常

| 场景 | RSS 均值 MiB | footprint 均值 MiB | footprint 两次范围 MiB |
| --- | ---: | ---: | ---: |
| 启用音频，仅加载字体 | 97.5 | 64.5 | 60.9–68.2 |
| 启用音频，绘制文字，无音频文件 | 369.4 | 346.4 | 70.5–622.4 |
| 播放音乐，不加载/绘制文字 | 104.6 | 71.5 | 67.6–75.5 |
| 文字 + 加载音乐，但不播放 | 116.7 | 82.8 | 79.0–86.7 |
| 默认缓冲区，文字 + 音乐播放 | 644.4 | 628.8 | 624.4–633.2 |
| 预热字形，文字 + 音乐，默认缓冲区 | 141.6 | 113.1 | 79.6–146.6 |
| 文字，小缓冲区，启用音频但无文件 | 108.4 | 70.9 | 69.5–72.3 |
| 文字 + 音乐播放，小缓冲区 | 114.8 | 81.2 | 79.5–82.8 |
| 预热字形，文字 + 音乐，小缓冲区 | 115.4 | 82.2 | 80.5–83.9 |

默认缓冲区的“文字 + 音乐播放”两次约 624–633 MiB footprint；启用音频但没有任何音频文件的文字绘制组也出现一次约 622 MiB、一次约 71 MiB。这不是“音乐解码自然需要几百 MiB”，也不证明播放音乐必然触发问题。

异常 vmmap 的 IOAccelerator (graphics) resident 分配约 545 MiB；相应普通文字组约 13 MiB，IOSurface 仍约 14 MiB。因此几百 MiB 增量主要在图形相关分配，不在音频样本。

字形预热在两次测试中约 80 / 147 MiB，未可靠消除异常。更小的绘制容量（512 顶点 / 768 索引）在本次两个重复中让文字/音乐组合稳定在约 80–84 MiB，单纯文字组约 70–72 MiB。

Macroquad 为绘制批次预分配缓冲区的源码与前次对照一致；这里的实验支持小缓冲区能规避这条异常路径，但尚未通过驱动分配堆栈唯一定位根因，不能断言“只有上传字形”或“只因为播放音乐”。应保留最小复现；后续若启用 HiDPI、改变字体策略或升级库，需要重新验证。

## 4. 真实游戏（含实际音乐播放）

| 场景 | RSS 均值 MiB | footprint 均值 MiB | footprint 两次范围 MiB |
| --- | ---: | ---: | ---: |
| 旧游戏绘制，只加载音乐 | 133.3 | 157.2 | 150.1–164.4 |
| 优化游戏绘制，只加载音乐 | 128.8 | 92.7 | 89.9–95.5 |
| 旧游戏绘制，实际播放音乐 | 132.6 | 150.4 | 150.3–150.4 |
| 优化游戏绘制，实际播放音乐 | 129.1 | 96.5 | 94.0–98.9 |

之前的 BATTLEBLOCKS_BENCH_HERO 自动进入对战会绕过“开始游戏”中的音乐播放。本次新增仅诊断使用的 play_music 标志，真的启动 looped 音乐（音量为零以避免测试干扰，但混音仍执行）。默认用户流程不变。

真实游戏播放音乐时，旧绘制 footprint 约 150 MiB，默认优化绘制约 94–99 MiB；两者都在相同的骑士/大恶魔/字体/棋盘/音乐内容下测量。游戏默认小缓冲区已启用，本次未在这个优化路径上观察到最小复现中的 600 MiB 异常。

## 5. 60 秒稳定性

| 场景 | 秒数 | RSS MiB | footprint MiB |
| --- | ---: | ---: | ---: |
| macro_noaudio_gl | 5 | 91.2 | 58.4 |
| macro_noaudio_gl | 15 | 91.1 | 58.3 |
| macro_noaudio_gl | 30 | 91.2 | 56.1 |
| macro_noaudio_gl | 60 | 91.3 | 56.1 |
| game_play_optimized | 5 | 127.9 | 90.2 |
| game_play_optimized | 15 | 128.1 | 92.8 |
| game_play_optimized | 30 | 118.5 | 76.8 |
| game_play_optimized | 60 | 118.6 | 79.2 |

无音频空窗口在 5–60 秒基本稳定；真实游戏播放音乐的占用在 30 秒后下降，60 秒约 79 MiB footprint。这体现驱动/分配器回收等动态变化，不能把某个启动快照称为恒定下限。此次未观察到持续增长，但单次 60 秒也不能证明所有操作均无泄漏。

以上不是所有角色、窗口缩放或长时间技能操作的内存泄漏审计，也不代表 Windows/Linux/Web。高 RSS 基线目前定位到平台窗口与渲染上下文这一层，不能称为所有平台都不可避免；异常图形分配有可复现和可规避的路径，但驱动级根因尚未唯一定位。

## 复现

```sh
python3 memory-comparison/build.py
python3 memory-comparison/run.py
python3 memory-comparison/run.py --normalized
python3 memory-comparison/run.py --isolate
python3 memory-comparison/run.py --buffers
python3 memory-comparison/stability.py
```

真实游戏对照还需要先在仓库根目录构建 battleblocks 的 Release 二进制。四个 Cargo.lock 都已保留。字体是现有游戏生成的固定子集，许可证随文件保存；BGM 沿用原仓库素材，仅用于本地测试。
