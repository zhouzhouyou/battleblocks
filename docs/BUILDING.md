# 构建与验证

## 环境准备

需要 Rust stable（通过 rustup 安装）、Python 3.10+ 和 pip。首次构建联网下载锁定依赖和校验后的字体，后续使用 `.build-tools/` 缓存。

- macOS 需要 Command Line Tools。
- Windows 使用 Rust MSVC 工具链及 Visual Studio C++ Build Tools。
- Linux 原生音频需要 ALSA 开发包和 pkg-config，Debian/Ubuntu 可安装 `pkg-config libasound2-dev`；运行游戏还需要桌面图形环境。

首次克隆后执行 `git config --local core.hooksPath .githooks`，启用提交前检查。资源和生成产物的边界见 [目录约定](LAYOUT.md)。

## 构建与运行

```sh
# 准备工具、测试、构建并打包当前系统原生版与 Web
python3 tools/build.py

# 只构建并打包当前系统原生版
python3 tools/build.py --native-only

# 只构建并打包 Web，仍运行原生 Rust 测试
python3 tools/build.py --web-only

# 仅准备字体与素材工具
python3 tools/build.py --setup-only
```

本机已有 `.build-tools/python-path.txt` 时，使用其中指定的 Python 执行构建脚本。完成环境准备后，执行 `cargo run --release --locked` 启动游戏。

构建自动裁剪字体，校验 WebP 和仓库资源，运行测试并打包。已手动完成 Release 构建时，可以用同一 Python 执行 `tools/package.py` 重新打包，支持 `--native-only` / `--web-only`。打包会清理对应旧产物，收集依赖许可，并将包体积写入本地 `dist/measurements.json`。

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

打开 [本地网页](http://127.0.0.1:8765/)。部署时上传整个 `dist/web/`，保留 HTML、JS、WASM、音频与许可目录。服务器为 WASM 返回 `application/wasm`，可启用 gzip / Brotli。首次点击或按键后解锁声音。

- `.github/workflows/pages.yml` 在推送 main 或手动触发时构建并部署网页，GitHub Pages 的 Source 设为 GitHub Actions。
- `.github/workflows/windows-release.yml` 在 Windows runner 上测试、构建并打包；手动触发时填写已有 draft release tag，产物与校验文件会附加到该 release。
- macOS 包默认本地构建，`.github/workflows/macos-build.yml` 支持手动触发。
- PR 的仓库策略检查由 `.github/workflows/repository-policy.yml` 执行。

## 验证选择

游戏逻辑改动先运行 `cargo test` 和 `tools/check_headless.py`。Headless 使用权威 `Session` 模型，非 UI 验证共用实际战斗规则。

```sh
cargo test
python3 tools/check_headless.py
python3 tools/check_repository.py
```

改收益逻辑必须补测试，覆盖实际规则、消耗、资源上限、目标差异及预览不改变状态。事件或状态改动还应覆盖来源链、目标被动、状态持续与回滚，并用无窗口 `--autoplay` 验证批量对局。

只有输入、绘制、动画、音频或窗口交互本身变化时，才运行需要窗口的 `tools/check_debug_console.py`、`tools/check_ability_events.py`、`tools/check_new_roles.py`。纯文档改动检查链接与仓库策略即可。游戏改动验证完成后，使用构建脚本更新 macOS 与网页包。

Headless 命令、脚本与批量对局参数见 [调试说明](DEBUGGING.md)，角色强度评估遵循 [平衡评估](BALANCE-EVALUATION.md)。所有日志、检测结果和统计输出只存本地。
