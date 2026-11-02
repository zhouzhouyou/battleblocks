# 狗头人牧师动画源 v4

使用内置 imagegen，以 [狗头人人设图 v1](../../priest-kobold-model-sheet-e-v1.webp) 为身份参考，生成透明背景 4×4 源图 `animation-source.webp`。

## 生成提示词

Create a perfectly aligned 4 columns × 4 rows grid of sixteen distinct full-body frames of the same BattleBlocks dog-headed kobold priest from the selected model sheet. Square overall canvas, genuinely transparent background, no panel borders, no labels, no floor. One character in each equal-sized cell; aligned feet, same pixel scale and 3/4 front camera. Do not crop staff, ears, feet, tail or pockets.

Lock design: adult brown dog-headed kobold, amber eyes, uneven triangular ears, broad muzzle, compact wiry build; royal-blue simple high-collar priest robe with modest gold edging, cream prayer stole and small original gold sun emblem; exactly two brown leather square-tile pockets on the front belt; long solid-gold staff nearly full character height, simple smooth round gold-ball tip and a few raised bands. Staff stays in the same hand. No book, cup, jewelry, armor, halo or extra bags.

Row 1: normal idle, subtle breathing and blink. Row 2: awakened idle, same clothes, only small controlled warm-gold light at the staff sphere. Row 3: normal cast, glance toward pocket, draw blank square tile, present it, return. Row 4: awakened cast, same action with restrained sphere light. Every tile is completely plain and single-color, with no pattern, emblem, sun, icon, writing or ornament. Crisp E-style pixel clusters and stepped hue ramps, no blur or random noise.

## 导入

`tools/prepare_role_assets.py` 从此目录读取源图，只做确定性的裁切、锚点对齐、最近邻缩放、64 色共享色板和二值透明导入。帧序：普通待机 0–3、觉醒待机 4–7、普通施法 8–11、觉醒施法 12–15。觉醒视觉光点是动画表现，不改变技能规则。源图保留供追溯；`assets/roles/priest.webp` 是运行时资源。
