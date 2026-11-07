# 构建与验证

## 环境准备

需要 Rust stable（通过 rustup 安装）、Python 3.10+ 和 pip。首次构建联网下载锁定依赖和校验后的字体，后续使用 `.build-tools/` 缓存。

- macOS 需要 Command Line Tools。
- Windows 使用 Rust MSVC 工具链及 Visual Studio C++ Build Tools。
- Linux 原生音频需要 ALSA 开发包和 pkg-config，Debian/Ubuntu 可安装 `pkg-config libasound2-dev`；运行游戏还需要桌面图形环境。

首次克隆后执行 `git config --local core.hooksPath .githooks`，启用提交前检查。资源和生成产物的边界见 [目录约定](LAYOUT.md)。

## 构建与运行

```sh
# 准备工具、构建并打包当前系统原生版与 Web
python3 tools/build.py

# 只构建并打包当前系统原生版
python3 tools/build.py --native-only

# 只构建并打包 Web
python3 tools/build.py --web-only

# 显式运行完整 Rust 与工具单测，再构建打包
python3 tools/build.py --test

# 仅准备字体与素材工具
python3 tools/build.py --setup-only
```

本机已有 `.build-tools/python-path.txt` 时，使用其中指定的 Python 执行构建脚本。完成环境准备后，执行 `cargo run --release --locked` 启动游戏。

构建自动裁剪字体，校验 WebP 和仓库资源并打包。默认不执行单测，`--test` 显式执行完整 Rust、WebP 工具和平衡统计工具单测，不启动平衡对局评估。已按下表完成验证后，重新打包无需重复测试。已手动完成 Release 构建时，可以用同一 Python 执行 `tools/package.py` 重新打包，支持 `--native-only` / `--web-only`。打包会清理对应旧产物，收集依赖许可，并将包体积写入本地 `dist/measurements.json`。

| 目标 | `dist/` 中的输出 |
| --- | --- |
| macOS | `BattleBlocks.app`、`macos-<架构>.zip` |
| Windows x64 | `battleblocks-windows-x64/battleblocks.exe`、`windows-x64.zip` |
| Linux | `battleblocks-linux-<架构>/battleblocks` 及同名 ZIP |
| Web | `web/`、`web.zip` |

原生打包默认针对当前主机。原生图像、音效与压缩音乐内嵌，打开应用或对应可执行文件即可运行；Linux 仍依赖系统图形与音频库。macOS 包未签名、公证。

## 网页运行与部署

```sh
python3 -m http.server 8765 --bind 127.0.0.1 --directory dist/web
```

打开 [本地网页](http://127.0.0.1:8765/)。部署时上传整个 `dist/web/`，保留 HTML、JS、WASM、`assets/`、音频与许可目录。`assets/` 存放角色图集与特效表，模块只保留代码、配置与字体；首页只需约 1.2 MB 的 WASM 和当前英雄图集（约 0.9 MB），其余角色与特效在选人、开战时按需加载。服务器为 WASM 返回 `application/wasm`，可启用 gzip / Brotli。首次点击或按键后解锁声音。

- `.github/workflows/pages.yml` 在推送 main 或手动触发时构建并部署网页，GitHub Pages 的 Source 设为 GitHub Actions。
- `.github/workflows/windows-release.yml` 在 Windows runner 上测试、构建并打包；手动触发时填写已有 draft release tag，产物与校验文件会附加到该 release。
- macOS 包默认本地构建，`.github/workflows/macos-build.yml` 支持手动触发。
- PR 的仓库策略检查由 `.github/workflows/repository-policy.yml` 执行。

## 验证选择

先判断修改影响的行为，再选择验证。测试、真实对局回归、强度评估和发布构建分别执行，构建不会自动扩大测试范围。

| 改动范围 | 必要验证 |
| --- | --- |
| 文档、目录规则、工作流 | 链接、命令或工作流配置检查，以及仓库策略检查 |
| 图片、音频、文案、GUI | 对应资源、文案或界面检查；涉及输入、绘制、动画、音频或窗口交互时才运行窗口检查 |
| Python 制作、构建、分析工具 | 修改工具的定向测试，不启动游戏对战；构建流程用 `python3 tools/test_build.py`，平衡调度与统计用 `python3 tools/test_balance.py` |
| 单个角色技能、状态或数值 | `cargo test --locked <相关测试名或模块>`，覆盖受影响角色和交互；涉及命令、回滚或 headless 入口时增加 `tools/check_headless.py` |
| 公共战斗规则、收益、AI、随机机制、跨角色状态 | `cargo test --locked` 和 `python3 tools/check_headless.py` |
| 普通 bug 修复 | 复现与回归测试，按上述影响范围选择；只有修复导致角色明显强弱变化时才增加平衡评估 |
| 角色平衡调整、明显影响强度的修复 | 先完成规则回归，再按 [平衡评估](BALANCE-EVALUATION.md) 使用 `--focus`，只模拟包含受影响角色的配对 |

`cargo test --locked -- --list` 可查找现有测试名。定向筛选须确认实际运行了相关测试，不能以零测试通过作为验证。多个角色互相影响时覆盖双方；公共规则的影响范围覆盖整个公开名单时才做全名单平衡评估。工具自身修改涉及真实进程协议时，再运行对应集成检查。

改收益逻辑必须补测试，覆盖实际规则、消耗、资源上限、目标差异及预览不改变状态。事件或状态改动还应覆盖来源链、目标被动、状态持续与回滚。需要批量真实对局回归时使用无窗口 `--autoplay`，选择包含受影响双方的最小名单；它与统计强度评估分别按需选择。

所有修改交付前运行 `python3 tools/check_repository.py`。只有游戏代码、配置或发布资源变化时，验证完成后使用构建脚本更新 macOS 与网页包。纯文档、检查工具与验证流程改动无需重打游戏包。

Headless 使用权威 `Session` 模型，非 UI 验证共用实际战斗规则。命令、脚本与批量对局参数见 [调试说明](DEBUGGING.md)。所有日志、检测结果和统计输出只存本地。
