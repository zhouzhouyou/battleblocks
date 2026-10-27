# 图片格式与包体积实测

对照输入为 `73aed51` 的全部24张运行时 PNG：8张旧角色动画图集、7张觉醒皮肤、7张方块、角色缩略图和失败画面。原始美术源文件保留在 `asset-sources/`，不随游戏分发；运行资源 `assets/` 只保留一份 WebP。所有 WebP 使用 Pillow 的 lossless/exact、method=6，逐张比较尺寸和完整 RGBA 字节一致，旧图集的帧顺序、时长、透明度不变。

三组采用相同源码、release设置和锁定依赖，仅改变图片编码与解码功能。中间组仍加载PNG，用来单独测量WebP解码器的开销。单位为字节：

| 项目 | PNG | PNG＋WebP解码器 | 全部WebP | WebP相对PNG |
| --- | ---: | ---: | ---: | ---: |
| 图片合计 | 146,153 | 146,153 | 79,086 | −67,067 |
| macOS可执行文件 | 1,796,752 | 1,862,992 | 1,796,944 | +192 |
| 原始WASM | 1,020,283 | 1,101,355 | 1,034,291 | +14,008 |
| macOS ZIP | 1,245,031 | 1,283,724 | 1,242,053 | −2,978 |
| Web ZIP | 902,675 | 934,995 | 893,774 | −8,901 |

按最终下载包选择WebP。收益有限：macOS ZIP约省2.9 KiB，Web ZIP约省8.7 KiB；未经HTTP压缩直接传输的WASM会更大约13.7 KiB。Web启动文件gzip合计从614,716降至605,815字节；PNG本身也会被整包压缩，不能把图片文件节省的65.5 KiB直接当作包体节省。相同解码尺寸不减少GPU纹理内存。ZIP元数据或工具链变化可能带来少量差异，不以个位字节作为格式优劣依据。

Macroquad 0.4.16 的已有 `image 0.24.9` 依赖启用PNG/TGA，没有直接暴露WebP开关。项目显式依赖同版本 `image` 并仅启用 `webp`，通过Cargo特性合并启用纯Rust解码能力；不新增crate或原生libwebp。Cargo.lock仅在项目根依赖表增加image，其他版本不变。纹理加载改为按文件内容识别格式。

复现：用 `.build-tools/python-path.txt` 中的Python执行 `tools/compare_image_formats.py --png-ref 73aed51`（需Pillow）。脚本在忽略的 `.build-tools/image-formats/` 中分别构建、打包，不覆盖正式源码和 `dist/`。完整逐图和包体测量见 `analysis/image-format-comparison.json`。macOS结果针对Apple Silicon，不能推断其他系统的解码器开销。

本轮清理将57份重复候选、审阅截图和中间调参文件移到本地 `.build-tools/pr-review-archive/`；PR保留最终对照与验证CSV、汇总、规则文档以及实际运行资源。Git历史仍可追溯先前提交。
