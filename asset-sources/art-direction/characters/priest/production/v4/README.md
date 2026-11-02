# 狗头人牧师游戏素材 v4

新狗头人牧师以蓝金简洁牧师袍、纯金球头长杖和腰带正面两只口袋替换旧 P-C 图像。`animation-source.webp` 是透明源图；`grid-384.webp` 是导入后的 1536×1536 图集预览；四个动画 WebP 分别展示普通／觉醒待机及施法；`normal-idle-0.webp` 与 `awake-idle-0.webp` 是静帧预览。提示词见 [PROMPT.md](PROMPT.md)。

运行时只打包 `assets/roles/priest.webp`。每帧 384×384，16 帧，脚底锚点 (192,336)，每行最大可见高度 320，整个角色共享至多 64 色、二值透明。空白方块无图案；觉醒只在金杖球头增加少量光。检查结果见 `assets/roles/report.json`。
