# BattleBlocks 方块战斗

Rust + Macroquad 开发的单机回合制三消战斗游戏。双方共用棋盘，通过消除、角色技能与觉醒对战，支持中文和英文。

[在线游玩](https://zhouzhouyou.github.io/battleblocks/) · [下载安装包](https://github.com/zhouzhouyou/battleblocks/releases)

## 从源码运行

需要 Rust stable、Python 3.10+ 和 pip，以及对应系统的编译工具。首次构建需要联网下载依赖与字体。

```sh
python3 tools/build.py
cargo run --release --locked
```

构建脚本将当前系统的原生包和网页包输出到 `dist/`。验证按改动范围选择，需要完整单测时加 `--test`。系统依赖、单独构建与部署方法见 [构建与验证](docs/BUILDING.md)。

## 基本操作

- 点击相邻方块交换，消除并完成连锁后换人。
- 点击技能或按 `1` / `2` / `3` 施法，范围技能点击棋盘确认，右键或 `Esc` 取消。
- 按住 `H` 查看可行移动，`P` 暂停，`R` 重开，`M` 静音，`L` 切换语言。
- 角色能力与数值在游戏内查看，右上角 `?` 提供规则说明；设置 → 实验可开关 AI 推演的对手技能、反击、后续回合与传送门因素。

## 开发文档

- [目录与资源入库约定](docs/LAYOUT.md)
- [开发约定](AGENTS.md) · [代码结构](docs/ARCHITECTURE.md)
- [调试与无窗口运行](docs/DEBUGGING.md) · [平衡评估](docs/BALANCE-EVALUATION.md)
- [技能规则](docs/SKILL-RULES.md) · [美术规范](docs/art/ART-DIRECTION.md) · [角色档案](docs/art/CHARACTER-REVIEW.md)

## 授权

依赖与字体许可随安装包提供。原素材的美术与音频授权仍待核实，macOS 包尚未签名、公证。
