# 当前美术资产与历史归档

更新：2026-11-01。按用户要求完成多轮清理：

- 2026-10-29：移除矮人旧稿 16 个旧版图片／动画文件；这些文件随后也已从 Git 历史移除，不从生成缓存恢复到当前目录。
- 2026-11-01：移除 64 个已淘汰或重复的文件，包括三位新角色的 production/v1、v2 迭代图片、重复的 `grid-preview.png`、修女旧动画源、骑士 v4–v7／v3a-b／结构探索图／首轮稿、牧师人设 v2、修女 N-B／C／D；并重写本地 Git 历史以清除这些大文件。全部提交时间与说明保留，origin/main 仍是当前分支祖先。v1/v2 目录只保留 `PROMPT.md`。
- 2026-11-01（矮人旧稿）：上述 16 个矮人旧稿文件也从 Git 历史移除；当前 Git 历史不再包含任何已淘汰草稿，仅本地历史备份保留。
- 2026-11-01（参考图）：9 张用户参考原图已从工作树与 Git 历史移除，`characters/*/references/` 整体加入 `.gitignore`；原图只在本地保留，仓库不再包含参考图。
- 2026-11-01（源图格式）：`asset-sources` 下 53 张 PNG 与 24 个 GIF 转为无损 WebP，77 个文件由 58.9MB 降至 40.4MB；`book-growth-independent.gif` 与 `dwarf-book-elevation.gif` 因动画 WebP 会合并帧并缩短总时长而保留 GIF。原始 PNG/GIF 已从 Git 历史移除（两个书本 GIF 除外）。游戏运行时始终只打包 `assets/` 下的 WebP。

用户技能、选稿和修改原文仍保留在角色档案。

## 矮人当前有效资产

- `asset-sources/art-direction/characters/dwarf/dwarf-model-sheet-black-red-v5.webp`：唯一当前造型基准，人物与武器三视图、黑红无盔、装备和表情。
- `asset-sources/art-direction/characters/dwarf/MODEL-SHEET-V5.md`：用户要求与人设图生成记录。
- `asset-sources/art-direction/characters/dwarf/references/`：用户参考原图，仅本地保存；该目录已忽略，不进入仓库。
- `asset-sources/art-direction/characters/dwarf/animations/v6/`：最新独立人物、直接从人设图裁出的书封面／纸页／底封、透明书增厚 GIF、合成垫高 GIF，以及 README 来源和核对记录。
- 矮人历史提示词文档只作追溯，已加历史标记；对应旧图片不在工作树。

## 其他目录

- `asset-sources/art-direction/e-reference.webp`：当前 E 全局参考。
- `asset-sources/art-direction/f-reference.webp`：历史 F 参考。
- `asset-sources/art-direction/archive/`：其他早期方块与角色候选，非矮人旧单体稿；本轮未删除。
- `asset-sources/art-direction/characters/knight/`：K-A4 v8 人设、黑红金四稿 v3c、石中剑修订与提示词。
- `asset-sources/art-direction/characters/priest/`：P-C v3 人设、饮酒四稿与提示词；参考图仅本地保存。
- `asset-sources/art-direction/characters/nun/`：N-A 人设、production/v1 立绘与动画、提示词；参考图仅本地保存。
- `assets/roles/nun-standee-normal.webp`、`nun-standee-awake.webp`：修女普通／暗影立绘；当前未被渲染代码引用，按用户决定保留给未来的图片展览／图鉴功能，不视为待删资源。
- `docs/art/characters/`：每位角色的用户设计、选择和最新素材指引。

`../../asset-sources/art-direction/GENERATED-IMAGE-INVENTORY.json` 按 SHA-256 分别标记当前存在、用户要求移除、未采纳的生成原图。只有状态 `present` 对应当前工作树文件；状态 `removed_from_worktree_by_user_request` 的条目仅保留文字记录，其中矮人旧稿可用 `recover_from_git` 追溯，2026-11-01 清理的候选稿已不在 Git 历史中。`user_references` 条目标记为 `local_only_not_committed`，只保留文件名与 SHA-256 用于识别本地原图，不代表文件在仓库中。不再要求把生成缓存里所有旧图重新复制回来。透明书动画来自当前人设图原书像素；未采纳的另画书图不进入当前素材目录。素材仍处于设计评审阶段，不加入游戏运行时包，不修改游戏代码。
