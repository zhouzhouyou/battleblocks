# BattleBlocks 开发约定

修改、生成或移动文件前，必须阅读并遵从 [目录约定](docs/LAYOUT.md)。该文件是目录职责、资源入库及特殊目录用途的权威说明；目录变化必须同步更新它和检查规则。

## 全局美术方向

- 全项目美术统一采用高细节像素插画（densely detailed pixel illustration，明确像素簇与材质色阶）。生成或修改美术前，遵循 `docs/art/ART-DIRECTION.md` 的全局提示词和制作规范；不保留早期画风比较稿及其引用。
- 旧角色普通版保持原素材、帧数、顺序、时长、显示尺寸与既有施法表现；只优化觉醒版细像素待机与施法，各至少十二个独立姿势。巫师普通资源为 `assets/roles/wizard/runtime/normal.webp`，觉醒制作输入为 `assets-source/roles/wizard/animation/`。
- 生成或导出图片素材时优先选择无损 WebP；运行时只打包 `assets/` 下的 WebP，设计源图不进入游戏包。仅当动画 WebP 无法保持帧数、时序或像素时保留 GIF。
- 角色按 `docs/art/CHARACTER-REVIEW.md` 逐人评审；候选必须在性别、轮廓、服装、性格或觉醒方式上有实质差异，不只换色。概念稿未经用户选定，不视为最终游戏素材。
- 用户决定、设定原文、动画设计与未决规则记入 `docs/art/characters/` 并更新索引；讨论与提示词文字允许入库。所有参考原图禁止入库，仅保留文字说明。正式资源按 `docs/LAYOUT.md` 分入 `assets/`、`assets-source/` 或 `gallery/`。
- 每位角色评审都要包含「角色介绍」；未提供时提醒用户补充。矮人以 `docs/art/characters/DWARF.md` 为当前权威设计：造型强调矮壮，但最新显示要求不再缩成骑士一半；三位新角色统一显示高度与像素密度；最新为黑红服装、无头盔、双端对称精良战锤与锤形符文；觉醒不增加护甲或头发，只改变锤子的红色符文发光；仇恨之书另作脚下垫高层。
- 未获实现授权的角色评审只记录用户设定、选稿与修改意见。用户已授权矮人、新骑士、牧师完整开发与游戏接入。生成图不符合明确设定的地方必须标为待修，不能据图反过来修改用户设定。
- 未采用讨论稿、旧稿、预览、拼图与录屏不得进入当前树或 Git 历史，不从缓存恢复。只保留当前正式资源，不在文件、路径和文档中添加迭代版本号。矮人书本使用当前人设拆分的原封面。

## 验证与交付

- 角色平衡评估使用 [BALANCE-EVALUATION.md](docs/BALANCE-EVALUATION.md) 中的工具与统计方法；结果仅保存在本地输出目录。

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
    --roster 8,7,9,10,1,2 --out .local/analysis/autoplay.csv
  ```
  它逐局调用同一个 `Session` 和 `strategy`，冷却、口袋、状态、连续施法加价与终局加压都走正常结算，只跳过动画和 JSONL 回滚快照。`--level 0/1/2` 对应入门／标准／进阶。
- 只有输入、绘制、动画、音频或窗口交互本身变更时，才运行需要窗口的 `tools/check_debug_console.py`、`tools/check_ability_events.py`、`tools/check_new_roles.py`；能力事件或状态逻辑变更应优先用 `cargo test`、`tools/check_headless.py` 和 `--autoplay` 组合验证来源链、目标被动、状态和回滚。

## 技能文案

- 技能数值优先使用百分比描述，例如「50%」，不用「一半」。
- 技能文案从配置读取；默认消除触发方块效果、默认释放技能结束当前回合，无须重复解释基础规则；不结束回合的技能须明确标注。

## 代码组织

- `main.rs` 只负责 GUI／headless 启动分流和窗口配置；headless 核心入口在 `src/headless.rs`，模型驱动在 `src/app/headless.rs`，输入、战斗结算、界面绘制分别放在 `src/app/` 下。不要将新增角色逻辑写回入口文件，也不要为本模式复制战斗规则。
- 技能效果放在 `src/content/skills/`，主动、被动与临时状态的事件处理放在 `src/content/handlers/`，身份与阶段配置从 `config/ability-loadouts.txt` 读取。通用收益、消耗、预览、AI 和移动代码只构造事件、分发及应用结果，不按角色或技能 ID 写规则分支；新技能通过处理器注册接入。使用同步、有类型的事件，不引入异步全局监听或插件加载层。
- 角色／回合状态必须进入快照与回滚；收益规则供实战、预览和 AI 共用。新增治疗转换由技能／状态事件处理器派生新事件，保留完整来源链，最近来源表示当前实际产生效果的能力；位置规则只检查其明确声明的来源，不穿透转换链。临时与跨回合状态进入状态列表，记录施加者与持续时间。统一收益结算入口负责分发与结算，重力与填充从 `src/board/gravity.rs` 扩展，不能分别在技能与动画里复制规则。具体职责见 `docs/ARCHITECTURE.md`。

## 仓库资源检查

- `assets/` 仅放当前游戏加载资源，按 roles、effects、environment、ui、audio 分类；正式人设与关键制作输入放 `assets-source/`，未实现图鉴资源放 `gallery/`，游戏配置放 `config/`。
- 提交前运行 `python3 tools/check_repository.py`；新制作资源确认采用后才更新 `config/asset-manifest.json`。本地候选和输出统一放 `.local/`。
- 本地启用 `git config --local core.hooksPath .githooks`，提交钩子通过 `--staged` 检查实际暂存内容。GitHub 的 Repository policy 工作流对提交和 PR 检查完整历史；不得绕过检查提交禁止产物。

## 禁止入库的生成产物

- 必须遵从 `docs/LAYOUT.md`：战斗日志、检测数据、工具输出、自动报告、清洗记录、单次验收和一次性排查记录禁止进入当前树及 Git 历史，也不得存到 docs；输出放 `.local/` 或临时目录。
- 只保留长期规则、设计决定及复现所必需的代码和输入。结构化制作参数与测试输入在 `config/repository-inputs.json` 声明用途；不能提交采样、状态快照或检查结果充当输入。

- docs 只保存持续有效的架构、操作、制作规范与当前设计依据；不存历史需求、阶段计划、OpenSpec 式变更提案或重复规则副本。相同资源只保留一份，制作与实验直接引用 assets 中的现有文件。
