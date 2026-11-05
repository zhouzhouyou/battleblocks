# 音频播放实验

比较 idle、整曲缓冲、流式解码与实时合成四种模式。使用 CPAL 输出与 Lewton 解码；score.json 是实时合成使用的音符输入，必须保留。实验程序与游戏共享正式压缩音乐输入，但独立构建，不改变游戏规则。

从本目录运行 `cargo test --locked` 与 `cargo build --release --locked`；运行 `target/release/battleblocks-audio-lab stream 30`，模式也可选 idle、buffer、synth。网页实验用仓库工具 `tools/prepare_audio_lab.py` 导出至 dist 后由本地网页服务器打开。

`tools/benchmark_audio_lab.py` 与 `tools/check_audio_stream_stability.py` 在 macOS 测量播放表现，输出仅写入已忽略的 results 目录。设备与运行环境会影响结果，单次测量、日志和报告禁止入库。目录职责见 [LAYOUT](../../docs/LAYOUT.md)。
