# Sunshine Control — Windows 版

Rust 原生 GUI，用于管理已安装的 Sunshine 和 MTT Virtual Display Driver。电视继续使用 Moonlight。

## 功能

- 列出 Windows 当前连接的屏幕、主屏、虚拟屏、分辨率和刷新率。
- 查看当前捕获目标，切换到指定屏幕或跟随主屏。
- 启动、停止、重启 Sunshine 服务。
- 启用、关闭已安装的 MTT 虚拟显示驱动；关闭正在共享的虚拟屏时回退到主屏。
- 配置 1～4 块虚拟屏，添加、修改或移除可用画面模式（分辨率和刷新率）。所有虚拟屏共用此模式列表，每块屏的当前模式在 Windows 显示设置中选择。
- 修改驱动配置前备份；应用配置时重启已启用的虚拟驱动，重新识别共享目标，失败时尝试恢复原配置和原运行状态。未启用驱动时只保存配置。
- 切换目标前备份配置；应用后台处理操作，按需申请 Windows UAC 授权。
- 打开 Sunshine 管理和配对页面、Windows 显示设置。
- 显示 Sunshine 日志；可选通过 ADB 读取 Moonlight 性能信息和近期解码阻塞记录。
- 只在启动、手动刷新和管理操作完成后读取状态；没有定时轮询。读取或操作时显示居中的弹出遮罩，保留主页面布局。

## 前置条件

1. Windows 10/11 x64，Sunshine 安装在默认的 `Program Files\Sunshine`，并注册 `SunshineService` 服务。
2. 如需虚拟屏，先安装 MTT Virtual Display Driver，并在 Windows 显示设置中选用扩展桌面。应用不附带、下载或安装驱动和证书，也不会修改实体屏分辨率。
3. 电视已安装 Moonlight，并与 Sunshine 配对。
4. 可选：安装 Android platform-tools，电视允许 ADB 调试；在应用设置中填写电视 IP 和 ADB 路径。

Sunshine 的显示器 ID 来自其启动日志。新接入的显示器需要重启 Sunshine 后刷新列表；应用不会凭屏幕编号猜测稳定 ID。切换屏幕或重启服务会中断串流，需要在 Moonlight 中重新连接。

## 开发与检查

在此目录执行：

```text
cargo check
cargo fmt --check
```

运行或发布需要另行生成可执行文件：

```text
cargo run
cargo build --release
```

`cargo check` 只做编译与类型检查，不会生成可运行的应用。遵守当前工作区规则，开发阶段不自动执行后两条命令。

## 使用

1. 启动应用，等待读取服务和屏幕状态。
2. 需要独立桌面时点击“启用虚拟屏”，在屏幕列表点击该屏幕的“共享此屏幕”。如果驱动已启用但没有可用屏幕，在 Windows 显示设置中启用扩展桌面。
3. 接受 Windows 管理员提示，然后在电视 Moonlight 中连接 Desktop。
4. 用 Windows 显示设置调整扩展屏排列；选中窗口后使用 `Win + Shift + 方向键` 移动窗口。
5. 结束使用可关闭虚拟屏，或停止 Sunshine 服务。关闭应用本身保留当前运行状态。

电视端可先使用已验证的 `2560×1440 / 60 FPS / H.264 / HDR 关闭 / 约 31 Mbps`。这些设置由 Moonlight 管理，控制台不会假装修改客户端编码参数。

## 配置与恢复

- 应用设置：`%LOCALAPPDATA%\SunshineControl\preferences.json`，不保存 Sunshine 密码。
- 管理动作：当前可执行程序以管理员身份执行受限操作；命令请求仅允许服务启停、选择当前屏幕、启停指定虚拟驱动。
- Sunshine 配置备份：`Program Files\Sunshine\config\sunshine.conf.backup-<时间戳>`。
- 当前适配的 MTT 驱动配置：`C:\VirtualDisplayDriver\vdd_settings.xml`，备份保存在同目录 `vdd_settings.xml.backup-<时间戳>`。仅替换数量和模式列表，保留 GPU、HDR 等其他节点。配置位于其他路径时不会猜测或创建新配置文件。
- 屏幕切换导致服务重启失败时，应用尝试恢复配置并启动服务；恢复错误会明确显示备份路径。
- 若需要手动恢复，停止 Sunshine，使用所需备份覆盖 `sunshine.conf` 后重新启动服务。

## 当前范围

- 本版本面向 Windows；平台代码独立放在 `src/platform/windows.rs`，尚未实现 macOS 后端。
- 不重写 Sunshine 的视频、音频和网络协议，不替代 Moonlight。
- 不包含托盘、自启动、自动安装、在线更新和远程电脑管理。
- ADB 数据为手动采样，近期日志事件可能属于之前的串流会话，不能据此认定当前持续积压。
- 已通过类型检查、格式检查和 Windows release 构建；已启动 GUI 检查首页、诊断及设置页面，确认能读取本机四块屏幕和当前虚拟屏共享目标。
- UAC、屏幕切换、服务启停、虚拟屏启停、配置应用及失败恢复尚未进行应用内实际操作验收；构建成功不等同于这些操作已验证。
