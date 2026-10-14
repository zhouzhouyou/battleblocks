# 内存分析（2026-10-14）

迁移代码已先提交到本地 Git：dee29f9。以下是提交之后的分析。现已将分组绘制和小缓冲区设为默认，保留原始背景图片。用 legacy_batches,large_buffers 可复现旧版基线，其他资源实验仍由 BATTLEBLOCKS_MEMORY_FLAGS 启用。

## 测量方法

本机 macOS arm64 Release 构建；骑士对大恶魔的后台静置场景，不模拟整局操作。每个场景单独启动进程，预热 2 秒，然后每 0.25 秒读取一次 ps RSS，共 6 次；随后执行 vmmap -summary 获取 physical footprint。各场景重复两次，第二轮倒序以减小顺序影响。表内 RSS 是两次采样中位数的均值，footprint 是两个 vmmap 快照的均值。不是长期峰值或泄漏测试。

RSS 和 physical footprint 的计算口径不同。RSS 有共享/可回收页面等影响，也不能完整表达 GPU 和其他计费内存；footprint 更适合在本次 macOS 对照中观察内存压力。不要相加两个指标。它们也不能直接套用到浏览器、Windows 或 Linux。

## 第一轮：关闭各项资源

| 场景 | RSS MiB | physical footprint MiB |
| --- | ---: | ---: |
| 原始绘制 | 131.9 | 151.2 |
| 不加载背景音乐 | 127.6 | 145.9 |
| 直接绘制棋盘背景和格子 | 125.4 | 146.2 |
| 不加载和绘制角色 | 122.9 | 139.6 |
| 不加载和绘制文字 | 128.7 | 144.6 |
| 不加载消除音效 | 131.0 | 148.9 |
| 同时关闭音乐/音效/角色/文字并直接绘制棋盘 | 99.0 | 113.4 |
| 窗口缩小一半 | 132.0 | 135.9 |
| 实验性 HiDPI | 306.9 | 400.4 |
| 只清屏的 Macroquad 空窗口 | 95.5 | 57.8 |

原图背景与格子改为矩形、边线后，RSS 约减少 6.5 MiB，footprint 约减少 5.0 MiB。与 1120×1120 RGBA 主背景约 4.8 MiB 的量级相符。程序化版本保留砖框/格子/配色，但只是对照候选，不是原图逐像素复刻。

角色移除可省约 9 MiB RSS / 12 MiB footprint；音乐约省 4–5 MiB。这些效果不应线性相加，因为分配器缓存、渲染批次和驱动资源会相互影响。

原生 quad-snd 的 mixer.rs 会将整个音乐解码成 Vec<f32>，而非按需流式解码。因而压缩音乐能减小包体积，却不按同样比例减小运行内存。

空窗口约 95.5 MiB RSS，但 footprint 约 57.8 MiB，说明系统/窗口/渲染初始化有明显基础开销，同时证明两个指标差别较大。空窗口是对照，不是无法降低的理论下限。

## 第二轮：绘制批次与缓冲区

| 场景 | RSS MiB | physical footprint MiB |
| --- | ---: | ---: |
| 原始绘制 | 131.0 | 148.9 |
| 底图先画完，图标按纹理分组 | 130.1 | 100.2 |
| 缓冲区改为 512 顶点 / 768 索引 | 132.0 | 100.9 |
| 分组 + 小缓冲区 | 128.4 | 91.5 |
| 分组 + 小缓冲区 + 直接绘制棋盘 | 122.3 | 85.0 |
| HiDPI 但不加载/绘制文字 | 127.4 | 200.6 |

Macroquad quad_gl.rs 为每个绘制批次创建顶点/索引缓冲区；默认最大容量是 10,000 顶点、5,000 索引。原版逐格交替画格子底图和方块图标，纹理切换会拆分批次；这是不必要的绘制结构开销。

将所有底图先画完，再按图标纹理分组，可保持原画面，把 footprint 从约 148.9 MiB 降到 100.2 MiB。只减小每批容量也降至约 100.9 MiB；两者结合约 91.5 MiB，降低约 38.5%。RSS 却只从约 131.0 降至 128.4 MiB，所以此前只报 RSS 会漏掉明显改善。

进一步直接绘制背景，footprint 约 85.0 MiB，较保留原图的优化版本再减 6.5 MiB。推荐先采用不改变画面的分组绘制及小缓冲区，再决定是否接受程序化背景；不需要为了省这一项内存更换引擎或重做角色美术。

## HiDPI

当前原版默认 high_dpi=false，sample_count=1。前面把 Retina 当成当前高占用原因只是猜测，已被配置检查排除。

实验开启 HiDPI 后，RSS 两次约 410 / 204 MiB，footprint 约 503 / 298 MiB，波动很大；关闭文字的 HiDPI 场景 footprint 约 201 MiB。内存映射显示 IOSurface 从约 14 MiB 增到 56 MiB，图形相关分配也增大。

字形图集及其上传与 HiDPI 异常有关系，但现有对照不足以唯一归因；需要跟踪字形尺寸、图集增长及驱动资源再定位。当前保持 HiDPI 关闭，不把这项实验的高数值当成发布版内存。

## 复现

```sh
cargo build --release --locked --bin battleblocks --bin memory_probe
python3 tools/analyze_memory.py
python3 tools/analyze_memory.py --render
BATTLEBLOCKS_BENCH_HERO=0 BATTLEBLOCKS_MEMORY_FLAGS= cargo run --release --bin battleblocks
BATTLEBLOCKS_BENCH_HERO=0 BATTLEBLOCKS_MEMORY_FLAGS=procedural cargo run --release --bin battleblocks
```

汇总数据在 analysis/*.json；原始 vmmap 输出在 dist/memory-analysis 与 dist/memory-render-analysis（构建输出目录不进入 Git）。10 项玩法/技能测试通过。诊断参数不在 Web 启用；尚未验证候选优化的全部角色、长时间交互及浏览器效果。
