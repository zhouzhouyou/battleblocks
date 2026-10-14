# 流式播放与实时合成实验

2026-10-14。两种路线均已实现可运行的独立原生实验和网页试听。正式游戏仍使用原来的Macroquad音频后端，尚未将实验后端接入游戏。

## 内存为什么降低

当前游戏保留整曲解码后的PCM。原生44.1kHz双声道f32的12秒BGM约4.04 MiB，长度翻倍，样本内存翻倍；压缩Ogg仅约166 KiB，不等于解码后的内存。

本实验的流式播放仍保留嵌入的压缩Ogg，逐包解码到固定128 KiB的PCM环形队列。48kHz双声道时约可缓冲341ms。队列满就暂停解码，播放取走后才补充，循环不会累积整首PCM；另有解码器、单包数据、输出后端等开销，所以128 KiB不是总进程内存。

实时合成只保存音符表、振荡器状态，在音频回调中生成当前输出块。score.json约数百字节，没有整曲PCM。导出试听WAV仅为了分享，并不参与实时播放。

## 原生实现与实测

CPAL 0.18.2输出，Lewton 0.9.4按包解码。输出设备为48kHz双声道；同一Release程序提供idle、buffer、stream、synth四种模式，静音增益仍执行取样、解码或合成。每种两次、16秒、第二轮反序；采样5/10/14秒。Windows/Linux通过CPAL提供平台输出路径，但未在这两个系统构建或运行验收；原生实验不是WASM程序。

首次对照（流式每次循环重新创建解码器）：

| 模式 | 14秒footprint均值 | RSS均值 | PCM存储 |
| --- | ---: | ---: | --- |
| 仅打开输出设备 | 4.67 MiB | 16.77 MiB | 无 |
| 整曲加载播放 | 16.25 MiB | 28.49 MiB | 4,608,000字节，约4.39 MiB |
| 流式播放原曲 | 6.03 MiB | 18.31 MiB | 固定128 KiB队列 |
| 实时合成 | 4.65 MiB | 16.74 MiB | 无整曲PCM |

CPU的14秒快照：流式1.0–1.8%，合成0–0.7%；它们不是精确的长期平均值。整曲模式的进程内存包含初始化解码/重采样分配后的分配器余留，差额不完全等于PCM大小。实验没有图形窗口，不可把这些总量当作完整游戏内存，也不能保证接入游戏后获得相同RSS差值。

循环重建版本的60秒测试中，footprint从5.50增至6.41 MiB。随后改为复用解码器并回到起点，跳过seek后已识别的Vorbis头包，避免每轮重读、创建整个setup。测试验证循环开始的512帧逐样本一致。

**最终复用版本60秒实测：5 / 30 / 58秒footprint均为5.44 MiB，RSS约17.61 / 17.72 / 17.72 MiB；60秒跨过4次循环，解码错误0，PCM队列欠载0。** CPU快照1.4–2.2%。这证明本次短测稳定，不是无限时长、所有输出设备或CPU负载下的保证。

音频回调不解码、不等待锁、不构造采样Vec；原曲解码和队列满时的等待在生产线程。原生使用线性重采样，可适配设备输出采样率；尚未专门测试其他采样率的设备，也没有对循环接缝做主观听感验收。欠载计数只监测本程序PCM队列，不监测所有设备/驱动故障。

4项Release测试通过：队列容量/环绕和立体声对应、跨线程发布顺序、解码循环起点一致、合成有限值/振幅边界。首次比较见measurements.json；重建版本长测见stability-recreate.json；最终长测见stability.json。原始vmmap日志在忽略的results/。

## 网页实现

流式原曲使用HTML音频元素，preload=none、循环播放，避免在应用代码中调用decodeAudioData预解码整曲。浏览器自行管理解码和缓存，仍可能下载/缓存整个压缩文件，不能保证它采用原生实验的128 KiB缓冲策略；未量化网页内存。

实时合成使用AudioWorklet，在音频线程按块生成方波、三角波、噪声和鼓声。浏览器已验证两个播放入口、合成回调持续超过70秒、停止与重新启动。合成曲为新的8秒循环机制演示，未转写/重新编配原曲，不是把原曲加bitcrusher。

参考：[CPAL](https://docs.rs/cpal/0.18.2/cpal/)、[Lewton](https://docs.rs/lewton/0.9.4/lewton/)、[AudioWorkletProcessor](https://developer.mozilla.org/en-US/docs/Web/API/AudioWorkletProcessor)、[HTMLMediaElement](https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement)。

## 运行

在仓库根目录：

```sh
cargo build --release --locked --manifest-path analysis/audio-lab/Cargo.toml
analysis/audio-lab/target/release/battleblocks-audio-lab stream 30
analysis/audio-lab/target/release/battleblocks-audio-lab synth 30
```

还支持buffer、idle；第二参数是运行秒数。默认音量25%，可用AUDIO_LAB_GAIN=0静音测量。默认输出支持f32/i16/u16，不支持的设备格式会明确报错。

网页：

```sh
python3 tools/prepare_audio_lab.py
python3 -m http.server 8765 --bind 127.0.0.1 --directory dist/web
```

打开 http://127.0.0.1:8765/audio-lab/ 。localhost支持AudioWorklet所需安全上下文；公开部署应使用HTTPS。网页不会随默认游戏构建自动发布实验。

macOS复测：python3 tools/benchmark_audio_lab.py；python3 tools/check_audio_stream_stability.py。后者检测欠载与解码错误，保存60秒稳定性数据。

## 接入选择

建议先接流式原曲：保持音乐内容，音效仍可一次解码，因为音效短且要快速重复触发。需要把BGM和音效统一到一个原生输出/混音后端，避免额外初始化两个设备输出；接入时验证暂停、静音、重开、退出和后台切换。当前独立实验不修改游戏分发依赖或包体。

实时合成可以作为另一种配乐选择；正式使用需完成编曲、听感调整和各设备验收。方波演示没有抗混叠振荡器，不声称是特定游戏机的音源模拟。

CPAL为Apache-2.0，Lewton为MIT或Apache-2.0，Ogg容器依赖为BSD-3-Clause；主要组件许可放在licenses/。原生Release实验二进制约0.69 MiB（包含原曲、三种模式及相关依赖），不是正式游戏接入后的新增体积。完整原生分发时还需收集所有实际链接依赖许可。原曲素材许可状态沿用原项目；实验旋律由本次实现的音符表定义。
