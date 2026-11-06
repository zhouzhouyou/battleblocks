# WebP 制作与发布前无损压缩

所有随游戏发布的 WebP 都必须通过 `tools/optimize_webp.py`，包括角色图集、独立装备、技能特效和 UI。范围为 `assets/**/*.webp`；制作源、图鉴和本地预览不进入游戏包。目录边界见 [目录约定](../LAYOUT.md)。

## 无损与压缩强度

`lossless=True` 表示保留像素，不代表编码器已经投入最大压缩计算。Pillow 的 WebP 导出在无损模式下，`quality` 控制压缩投入，而不是有损画质。省略该参数会使用默认值 75；提高到 100 不减少颜色、不缩小图片、不删帧。

正式导出使用 `lossless=True, exact=True, method=6, quality=100`。`exact=True` 同时保留完全透明像素下的 RGB。不得使用有损 WebP、near-lossless、降色、降分辨率、删帧或改时长来冒充本流程的体积收益。

## 制作流程

1. 按 [全局美术规范](ART-DIRECTION.md) 制作并确认采用资源；在资源清单中登记新采用文件。压缩工具不能批准未采用的候选。
2. 使用 `.build-tools/python-path.txt` 指定的 Python；首次运行 `tools/build.py --setup-only` 初始化固定版本的 Pillow 与字体工具。
3. 运行 `tools/optimize_webp.py --write`。工具逐文件重新编码，核对尺寸、每帧完整 RGBA、帧数、帧时长和循环设置；只有更小且完全一致的候选才替换原文件，并更新已有 `config/asset-manifest.json` 条目的字节数与 SHA-256。
4. 检查资源和清单差异，再运行 `tools/build.py`，同时更新 macOS 与网页包。

旧角色普通版受 `config/normal-art-lock.json` 约束：仍参与核对，但保持原文件字节，不为压缩改变原素材或修改锁。其他已经达到当前压缩强度、或重新编码后没有变小的文件保持原样。

动画若无法保留每一帧、原时长或循环设置，工具报错并保留原文件，必须先修正导出流程；不能静默合帧。命中时刻、脚底锚点等游戏配置也不得随压缩更改。

## 发布门禁

`tools/build.py` 和 `tools/package.py` 都先运行 `tools/optimize_webp.py --check`。存在可进一步无损缩小的资源时停止发布，要求先执行上述制作流程；构建和打包不会自动改写美术或清单。

通过结果仅缓存于 `.local/asset-compression/`，按文件内容、编码参数、Pillow 和 libwebp 版本失效。缓存与测量结果不入库。资源清单、普通版锁和仓库检查仍是独立约束，缓存不能替代它们。
