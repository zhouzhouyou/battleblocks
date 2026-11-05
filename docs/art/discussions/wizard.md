# 制作讨论与提示词

以下文字记录设计讨论与制作过程；当前采用资源以角色档案为准。

---

# 老巫师 · 仅觉醒细像素动画

用户最终明确：普通版保持不变、保持粗像素；觉醒版保持细像素，本Issue只优化觉醒相关动画。已先更新#11、#27、#28、#29范围，再改本实现。

**普通完全不由本导入器处理。** 普通运行时仍读取原 `assets/roles/wizard/runtime/normal.webp`，300×400帧、索引0/1/2/3、每帧0.1秒、无新增施法clip、无新脚锚，原缩略图恢复。`tools/check_wizard_normal_unchanged.py` 相对500673e验证普通素材与缩略图字节、普通注册表与元数据未变。

本目录只包含觉醒待机和觉醒施法，各12张独立姿势。两份 `*-source-.webp` 是已有细像素觉醒源图的逐字节复制，没有重新生成或重新设计角色。两份 `*-original-prompt.md` 原样记录当时内置imagegen提示词；只继承觉醒源图，提示词中涉及普通形态统一的历史描述不再作为需求。原始角色参考仍是 `assets/awakened/wizard-awakening.webp`。

导出保持细像素簇、64共享色、二值alpha、无抖动、最近邻，帧384×384、鞋底锚点(192,336)。源格362×362，同一缩放与逐帧鞋底注册用于两个动作；不使用的24×24粗像素网格。图集2304×1536（6×4），0–11觉醒待机，12–23觉醒施法，共24帧。`report.json`记录每帧像素哈希与边界；该24帧与中的对应觉醒帧像素相同，普通帧没有被带入。

待机12×60ms；施法ms为60,80,90,90,120,180,280,100,100,100,100,100。第5帧出手、第7帧引导、第8帧命中，命中后0.5s收势。只有觉醒施法采用真实命中时钟；普通原有表现保持。导入使用仓库Python运行 `tools/prepare_character_animations.py --role wizard`，再 `--registry-only`；完整 `tools/prepare_assets.py` 也包含该入口。动画WebP为循环评审文件，运行时施法Once。

当前验收、普通未变化证据与觉醒实战。


---

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.
Asset: BattleBlocks legacy wizard AWAKENED CAST, exactly TWELVE independently drawn poses in a strict equal 4 columns by 3 rows transparent 4:3 sprite sheet. Image1 is awakened idle identity/first-last pose reference; Image2 is normal cast timing reference. Keep same wizard face, blue-purple bent hat, white beard, blue robe, brown shoes, wooden staff and blue crystal. No new clothes, armor, crown, equipment or effects. Awakened posture more upright, disciplined and sure than normal cast, crystal slightly brighter in crisp pixel steps; real joint/silhouette differences, not just recolor. Densely detailed hard pixel clusters, transparent empty gutters, no lettering or grid.
Read in order: 1 upright neutral matching awakened idle, 2 slight knee sink elbows draw inward, 3 raise staff along body with straight back, 4 lift wrist and open empty hand, 5 decisive OUTWARD WRIST EXTENSION summoning/release, 6 upright extended guiding arms, 7 expanded stable control stance holding guide for impact, 8 distinct short strong DOWNWARD WRIST PRESS at impact, 9 bend elbow to unload fingers curl, 10 lower staff vertically and retract empty hand, 11 upright neutral with trailing sleeve, 12 almost idle first pose with last small beard/sleeve settling. Staff remains in viewer-right hand, free hand viewer-left. Strong articulate arm poses in frames4-9, soles stationary and separated throughout. Foot midpoint46% cell width, sole baseline93% cell height, at least12px transparent margin all sides. All cells same body/head scale as reference; do not shrink poses to accommodate staff. Preserve staff length and body proportions. No beams, aura, projectiles, magic rings, ground shadows. The unchanged board effects provide spells.


---

Act as the art director for an original game. Establish densely detailed pixel illustration as the global visual language for the entire project. Apply this direction consistently to characters, environments, props, items, UI, icons, visual effects, animation, loading screens and promotional art. Keep the project's existing genre, mechanics, camera and target platform; when one is unspecified, state an assumption before proposing assets.

Use carefully shaped pixel clusters, nuanced hue shifts and richer stepped ramps to describe cloth, foliage, stone and architecture. Keep a consistent pixel scale and concentrate intricate detail around focal assets, allowing quieter surfaces and distance to preserve hierarchy. Give characters expressive but readable frame animation and materials that match the environment's cluster language. Design UI icons at intentional resolutions rather than downscaling painted images. Effects may use controlled soft light, but must preserve the underlying pixel contours. Avoid random pixel noise, inconsistent sprite density and detail that vanishes or flickers at the target display size.

Create a compact art bible with a locked palette, silhouette and proportion rules, material examples, lighting hierarchy, UI/icon rules, motion guidelines and a short list of visual mistakes to avoid. Define measurable asset specifications appropriate to the chosen platform and camera, then keep them consistent across every batch. Preserve gameplay readability, accessible text and clear interactive targets at the actual display size. Use original designs and compositions rather than copying a particular game's characters, logos or artwork. When asked for one asset, deliver that asset within this art direction; do not redesign the style for each request.
Asset: BattleBlocks legacy wizard AWAKENED IDLE sprite sheet, exactly 12 independently drawn poses in equal 4 columns x 3 rows, transparent background. The provided sheet is the NORMAL form character reference. Preserve exactly that wizard identity, blue-purple bent hat, shaded face, white beard, robe structure, brown shoes and wood staff with blue crystal. Same scale, same camera, same outfit and equipment. Change the posture to a little taller/straighter and more controlled, not a different body; keep feet on the ground. Brighten ONLY the blue crystal slightly with crisp stepped pixels, no aura or particles. No armor, crown, new ornaments or new equipment. Maintain densely detailed hard pixel clusters, not smooth painting.
Square cells on a 4:3 sheet. Same foot midpoint and sole baseline in every cell, boots clearly separated, all 12 wholly inside cells with transparent gutters and at least 14 pixels margin above hat and below soles. Staff viewer-right, free hand viewer-left; do not center using staff extent. Subtle but readable independent poses, in reading order: 1 upright neutral, 2 inhale chest opens elbows soften, 3 chin lifts gaze ahead beard follows, 4 half-blink free hand opens slightly, 5 short blink fingers fully open, 6 eyes open free hand returns, 7 exhale elbows draw in, 8 beard settles shoulders relax, 9 wrist straightens robe edge trails, 10 staff returns close to body, 11 beard and hem swing back, 12 nearly neutral with last trailing movement into pose1. Never move the soles, resize individual frames, duplicate or recolor poses to fake motion. Preserve body proportions and staff length. No text, numbers, grid lines, floor, drop shadow or unrelated effects.
