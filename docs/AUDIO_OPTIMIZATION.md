# 音频优化与 8-bit 风格

2026-10-14，读取当前 Macroquad 0.4.16 / quad-snd 0.2.8 源码，使用 ffprobe 盘点并生成试听文件；未替换正式游戏音乐或音效。

## 当前成本

背景音乐：12秒，48kHz双声道Vorbis，169,994字节。消除音效：1.708秒，44.1kHz单声道16-bit WAV，150,688字节。

quad-snd 原生 mixer.rs 的 load_samples_from_file 把整个声音解码成 Vec<f32>，单声道复制为双声道，非44100Hz再重采样到44100Hz。因此BGM最终采样约4,233,600字节（4.04 MiB），音效约602,576字节（0.575 MiB）；这是按时长估算的样本数据量，不含编码填充、Vec容量、复制、重采样临时缓冲和音频框架。

当前BGM从48kHz重采样，有源解码数据与目标缓冲同时存活的阶段。改44.1kHz可以避免这一次重采样及临时分配，但不减少最终12秒双声道f32数据。低位深WAV不会以低位深保留在mixer中。

网页端 audio.js 使用 decodeAudioData 并保留整段AudioBuffer，其格式是每声道32-bit浮点并重采样至AudioContext采样率。假设48kHz，12秒双声道约4.39 MiB；单声道约2.20 MiB。网页换单声道有实际样本内存收益，原生端当前自动复制成双声道则没有同等收益。

参考：[AudioBuffer](https://developer.mozilla.org/en-US/docs/Web/API/AudioBuffer)、[decodeAudioData](https://developer.mozilla.org/en-US/docs/Web/API/BaseAudioContext/decodeAudioData)。浏览器实际采样率与额外内存未实测，不将理论样本大小当作总进程内存。

## 候选文件（同一12秒原曲）

| 文件 | 文件字节 | 原生最终采样估算 |
| --- | ---: | ---: |
| 当前原曲 Ogg | 169,994 | 约4.04 MiB |
| 44.1kHz单声道16-bit WAV | 1,058,512 | 约4.04 MiB |
| 降位/降采样感 Ogg试听，44.1kHz双声道 | 110,511 | 约4.04 MiB |
| 22.05kHz单声道8-bit WAV | 264,712 | 约4.04 MiB |

8-bit WAV比现有Ogg文件还大。试听Ogg只是原曲经过bitcrusher，未转写音符、重做配器或还原某种游戏机音源。声音风格变化与运行时内存优化是不同操作。

消除音效Ogg候选30,369字节，相比现有WAV少约80%；但本机FFmpeg内置Vorbis编码器使用双声道，网页解码声道数会增加，原生最终采样基本相同。正式替换优先用支持单声道的Vorbis编码器，并试听与测试浏览器兼容性。当前未直接应用该候选。

FFmpeg此安装不含libvorbis；试听脚本使用内置实验Vorbis编码器，单声道对照用WAV表达。可在支持libvorbis的构建环境做44.1kHz单声道Ogg，在包体积和网页内存之间取得更合适的组合。

## 可行路线

1. **保留原曲，优化分发**：44.1kHz单声道Ogg；音效也压缩为单声道Ogg。包更小、网页样本数据更少，但当前原生后端约4MiB BGM采样仍存在。
2. **保留原曲，优化原生常驻内存**：实现流式解码，只保留环形PCM缓冲。现有Macroquad音频API没有提供BGM流式播放入口，需要替换或扩展音频后端，并处理原生三平台与Web适配。曲子仅12秒，收益有限，先衡量复杂度。
3. **采用真正的8-bit风格并大幅压缩音乐数据**：把音乐重新编成音符与节奏表，实时用方波、三角波、噪声等合成，只保留短输出缓冲。这能省去完整曲目PCM；需重新编曲和合成播放实现。若预先把合成曲导出Ogg，仍然会被当前后端完整解码，不获得实时合成的常驻内存优势。
4. **缩短循环**：音乐长度直接影响解码内存，但需要重新编排循环，不能盲目截断原曲。

建议短期保持正式配乐，先试听风格；追求很小包体且愿意重新编曲时，再选择实时chiptune合成。此调研不宣称12秒BGM是整个进程百MiB占用的主要来源。

生成方式：`python3 tools/prepare_audio_candidates.py`，需要ffmpeg / ffprobe。试听和数字在 analysis/audio-candidates/，不随正式游戏打包。滤镜说明见 [FFmpeg acrusher](https://www.ffmpeg.org/ffmpeg-filters.html#acrusher)。

两种路线已实现独立原生及网页实验，实测、试听和当前接入范围见 [音频实验记录](../analysis/audio-lab/README.md)。正式游戏现已接入流式原曲：原生单CPAL输出混合短音效，网页媒体音乐加Web Audio音效。历史实验数据保留，不代表接入后的整游戏内存。
