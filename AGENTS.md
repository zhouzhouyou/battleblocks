# BattleBlocks 开发约定

## 全局美术方向

- 全项目美术采用 E 方案（densely detailed pixel illustration，明确像素簇与材质色阶）。生成或修改角色、环境、物件、图标、UI、特效与动画前，遵循 `docs/art/ART-DIRECTION.md` 中的全局提示词和制作规范；F 为历史方案，不再作为当前默认。
- 生成或导出图片素材时优先选择无损 WebP；运行时只打包 `assets/` 下的 WebP，`asset-sources/` 的设计源图不进入游戏包。仅当动画 WebP 无法保持帧数、时序或像素时保留 GIF。
- 角色按 `docs/art/CHARACTER-REVIEW.md` 逐人评审；候选必须在性别、轮廓、服装、性格或觉醒方式上有实质差异，不只换色。概念稿未经用户选定，不视为最终游戏素材。
- 每位角色的用户决定、设定原文、选定稿、修订图片、动画设计、待平衡参数与未决规则，都要记录到 `docs/art/characters/` 对应档案，并更新评审索引。提示词保存在 `asset-sources/art-direction/characters/`，采用版本化文件名，不覆盖历史定稿。用户提供的参考原图（`characters/*/references/`）不得提交到仓库，只保留在本地并标注来源；仓库内以角色档案的文字说明和提示词为记录。
- 每位角色评审都要包含「角色介绍」；未提供时提醒用户补充。矮人以 `docs/art/characters/DWARF.md` 为当前权威设计：造型强调矮壮，但最新显示要求不再缩成骑士一半；三位新角色统一显示高度与像素密度；最新为黑红服装、无头盔、双端对称精良战锤与锤形符文；觉醒不增加护甲或头发，只改变锤子的红色符文发光；仇恨之书另作脚下垫高层。
- 未获实现授权的角色评审只记录用户设定、选稿与修改意见。用户已授权矮人、新骑士、牧师完整开发与游戏接入。生成图不符合明确设定的地方必须标为待修，不能据图反过来修改用户设定。
- 用户要求移除旧稿时，旧素材从工作树删除、历史通过 Git 追溯；不要从生成缓存重新复制到当前资产目录。矮人造型以 v5 人设为准，当前人物动画为 `production/v3/`，书本拆分仍取 `animations/v6/`，仇恨之书封面取自该人设装备拆分，不另行改图案。

## 验证与交付

- Headless 模式是权威游戏模型；GUI 只是同一模型的输入、帧循环和可视层。非 UI 逻辑验证不得另写一套规则，也不应依赖窗口。
- 改收益逻辑必须补测试：实际规则、消耗、资源上限、目标差异、预览不改变状态。
- 构建用 `.build-tools/python-path.txt` 指定的 Python 运行 `tools/build.py`；完成后更新 macOS 与网页包。
- 无窗口验证优先使用：
  ```bash
  cargo run --release -- --headless --log /tmp/battleblocks.jsonl \
    --command 'pause on' --command 'board load ...' --command 'move 1 1 2 1'
  ```
  命令较多时使用脚本：`--script path/to/commands.txt`；`--script -` 从 stdin 读取。默认只输出命令反馈和最终摘要；`--verbose` 才把逐条事件打印到终端，完整事件始终写入 `--log` 指定的 JSONL。`--headless-max-steps N` 防止模拟死循环并以退出码 2 报错；`BATTLEBLOCKS_HEADLESS_SEED` 固定随机种子。
- 非 UI 改动先运行 `cargo test` 和 `tools/check_headless.py`。`check_headless.py` 会无窗口验证控制台调试编辑、移动、施法、回滚、AI 回合、待结算消除及批量对局。
- 批量平衡与规则验证使用无窗口 `--headless --autoplay`，不要为了跑对局启动 GUI：
  ```bash
  cargo run --release -- --headless --autoplay --samples 48 --seed 900000 --level 1 \
    --roster 8,7,9,10,1,2 --out analysis/autoplay.csv
  ```
  它逐局调用同一个 `Session` 和 `strategy`，冷却、口袋、状态、连续施法加价与终局加压都走正常结算，只跳过动画和 JSONL 回滚快照。`--level 0/1/2` 对应入门／标准／进阶。
- 只有输入、绘制、动画、音频或窗口交互本身变更时，才运行需要窗口的 `tools/check_debug_console.py`、`tools/check_ability_events.py`、`tools/check_new_roles.py`；能力事件或状态逻辑变更应优先用 `cargo test`、`tools/check_headless.py` 和 `--autoplay` 组合验证来源链、目标被动、状态和回滚。

## 技能文案

- 技能数值优先使用百分比描述，例如「50%」，不用「一半」。
- 技能文案从配置读取；默认消除触发方块效果、默认释放技能结束当前回合，无须重复解释基础规则；不结束回合的技能须明确标注。

## 代码组织

- `main.rs` 只负责 GUI／headless 启动分流和窗口配置；headless 核心入口在 `src/headless.rs`，模型驱动在 `src/app/headless.rs`，输入、战斗结算、界面绘制分别放在 `src/app/` 下。不要将新增角色逻辑写回入口文件，也不要为本模式复制战斗规则。
- 技能效果放在 `src/content/skills/`，主动、被动与临时状态的事件处理放在 `src/content/handlers/`，身份与阶段配置从 `assets/ability-loadouts.txt` 读取。通用收益、消耗、预览、AI 和移动代码只构造事件、分发及应用结果，不按角色或技能 ID 写规则分支；新技能通过处理器注册接入。使用同步、有类型的事件，不引入异步全局监听或插件加载层。
- 角色／回合状态必须进入快照与回滚；收益规则供实战、预览和 AI 共用。新增治疗转换由技能／状态事件处理器派生新事件，保留完整来源链，最近来源表示当前实际产生效果的能力；位置规则只检查其明确声明的来源，不穿透转换链。临时与跨回合状态进入状态列表，记录施加者与持续时间。统一收益结算入口负责分发与结算，重力与填充从 `src/board/gravity.rs` 扩展，不能分别在技能与动画里复制规则。具体职责见 `docs/ARCHITECTURE.md`。
