# 高清投屏 · HD Screen Cast

**将 Windows 电脑桌面高清串流到电视，支持独立虚拟屏。**

[English](README.md) · [下载运行包](https://github.com/MaxwellEdisons/hd-screen-cast/releases/latest) · [Moonlight 安卓 APK](https://github.com/moonlight-stream/moonlight-android/releases)

Rust 原生桌面应用，将 Sunshine 安装、共享屏幕切换、虚拟屏配置和本地诊断集中在一个界面中。电视端继续使用 Moonlight 接收画面。

## 适合哪些场景？

当原生投屏可选分辨率不足、电脑与电视比例不一致，或只能镜像主屏时，可以使用这套高清桌面串流方案：

- 选择指定实体屏共享，或创建独立虚拟桌面。
- 设置虚拟屏可用分辨率和刷新率，不必受实体显示器分辨率约束。
- 把电视作为额外桌面，用于演示、办公、网页、视频或游戏。
- 在主机显卡、电视解码器、面板和网络均支持时，使用 2K/4K 模式。

**当前支持 Windows 10/11 x64。macOS 处于规划阶段，尚未实现。** 本项目不实现 AirPlay/Miracast 协议，也不能突破设备解码能力和网络限制。虚拟屏属于同一个 Windows 登录会话，鼠标、键盘共用。

```text
Windows 电脑：高清投屏 + Sunshine + 可选虚拟屏
                       ↓ 局域网
电视 / 安卓电视 / 电视盒子：Moonlight
```

## 下载哪个版本？

前往 [GitHub Releases](https://github.com/MaxwellEdisons/hd-screen-cast/releases/latest)。

| 运行包 | 适用情况 | 包含内容 |
| --- | --- | --- |
| `HD-Screen-Cast-v0.3.0-windows-x64-Offline.zip` | 新电脑首次使用 | 应用、Sunshine 官方 MSI、已签名虚拟屏驱动、微软运行库安装器、许可证和校验值 |
| `HD-Screen-Cast-v0.3.0-windows-x64-Controller.zip` | 已安装 Sunshine 和虚拟屏驱动 | 应用和说明文档；首次初始化需要完整离线包 |

请**完整解压**后运行 `HD-Screen-Cast.exe`，不要直接在压缩包内运行。离线包中的 `packages/` 要与程序放在一起。驱动和服务需要注册到 Windows，复制目录到另一台电脑仍需首次初始化。

打开发布版应用不需要 Python、Rust、Node.js 或 WebView。控制台使用静态 C 运行库；随附微软运行库安装器为外部组件提供依赖。

## 新电脑完整使用步骤

### 第一步：初始化电脑

1. 下载并完整解压 **Offline 离线包**。
2. 运行 `HD-Screen-Cast.exe`，进入“首次初始化”。
3. 勾选缺少的组件，点击“一键初始化所选组件”。
4. 确认 Windows 管理员授权；出现安装器或驱动发布者确认窗口时，请自行完成。
5. 应用先校验安装文件 SHA-256，再安装运行库和勾选的缺失组件。没有已有配置时，创建默认 **2560×1440 / 60Hz** 虚拟屏模式。
6. 如果 Windows 提示重启，请手动重启电脑，重新打开应用并点击“刷新状态”。

已有组件会跳过，不覆盖 Sunshine 账号、配对和已有配置。应用不会导入根证书、关闭系统驱动签名检查，也不会复制另一台电脑的配对身份。安装失败时，已安装组件会保留，处理提示后可以重新初始化。

### 第二步：设置 Sunshine

1. 电脑和电视连接同一路由器，优先用网线或信号良好的 5GHz Wi-Fi。
2. 如果 Sunshine 已停止，在“屏幕与串流”页面启动串流服务。
3. 点击“设置 Sunshine 账号”，或打开 [Sunshine 本机管理页面](https://localhost:47990)。
4. 首次访问设置管理账号和密码。本机页面使用本地证书；遇到浏览器证书提示，请先确认地址是 `localhost`，再自行处理。

### 第三步：电视安装 Moonlight 安卓 APK 并配对

**官方下载：[moonlight-stream/moonlight-android/releases](https://github.com/moonlight-stream/moonlight-android/releases)。**

1. 安卓电视、Google TV 或安卓电视盒子，可从应用商店安装 Moonlight，也可从上述官方 Releases 页面下载适用的 APK。
2. 电视无法直接下载时，用电视支持的 U 盘或文件传输方式导入 APK，再在电视上安装。能否安装及授权方式取决于电视系统。
3. 打开 Moonlight，选择自动发现的电脑；如果找不到，在 Moonlight 手动添加电脑的局域网 IPv4 地址。
4. 电视显示 PIN 后，在电脑应用点击“电视配对”，进入 Sunshine 页面输入 PIN。
5. 配对成功后，在 Moonlight 选择 **Desktop**。

Apple TV 应安装 tvOS 版 Moonlight。三星 Tizen 和 LG webOS 不能直接安装安卓 APK，应使用安卓盒子或对应平台支持的客户端。

### 第四步：选择共享屏幕

- **实体屏：** 在“屏幕与串流”点击对应显示器的“共享此屏幕”。
- **独立虚拟屏：** 启用虚拟屏，按提示重启 Sunshine、刷新列表，然后选为共享目标。
- **主屏：** 点击“跟随主屏”。

切换目标或重启组件会短暂断开串流，在电视 Moonlight 重新连接 Desktop 即可。虚拟屏起初只有壁纸是正常现象，需要将窗口移入扩展屏。可以在 Windows 显示设置调整排列，再用 `Win + Shift + 方向键` 移动选中的窗口。

### 第五步：配置虚拟屏数量和画面模式

进入“虚拟屏配置”，可设置 **1～4 块虚拟屏**，添加、修改、移除可用分辨率和刷新率模式，最多保留 12 个模式。

“保存并应用配置”会先备份，再重启已启用的驱动和正在运行的 Sunshine，并重新识别虚拟共享目标。驱动关闭时只保存配置，启用后生效。减少数量时可能需要其他共享目标；失败时尝试恢复配置，并报告恢复失败项。

**所有虚拟屏共用可用模式列表**；每块屏当前使用的模式，在 Windows 显示设置选择。保存可用模式不等于修改 Moonlight 的视频分辨率。

## 画质与流畅度

建议先在 **Moonlight 客户端**使用：

```text
2560×1440 · 60 FPS · H.264 · HDR 关闭 · 码率约 30 Mbps
```

这是开发环境验证过的起点，不代表适用于所有电视。如果 4K 清楚但鼠标延迟大，除了网络丢帧，还要看电视解码时间。开发时使用的电视在 4K H.264 下出现解码积压，2K60 更流畅。HEVC、HDR 兼容情况取决于电视和客户端。主机应用不会覆盖电视端编码偏好。

### 实测设备与数据（2026-10-30）

| 项目 | 测试环境 |
| --- | --- |
| 电脑显卡 | **Intel Arc 130T 核显**，使用 Intel Quick Sync 硬件编码；不依赖独立显卡 |
| 电视 | 小米电视，ADB 型号 **MiTV_ASTP0**；Amlogic 硬件解码器 `OMX.amlogic.avc.decoder.awesome` |
| 客户端 / 主机 | Moonlight Android **12.2** / Sunshine **2026.1028.122642** |
| 网络 | 电脑和电视均连接同一路由器的 5GHz Wi-Fi；电视显示的无线协商速率为 150 Mbps（不是实测吞吐量） |
| 编码设置 | H.264，HDR 关闭；2K 码率约 31 Mbps，4K 约 63 Mbps |

| 场景 | 观测结果 | 使用建议 |
| --- | --- | --- |
| **2K：2560×1440 / 60 FPS** | 稳定运行；约 3 分钟连续观察中，渲染基本为 **59～60 FPS**，网络丢帧 **0%**，解码通常约 **1～3 ms**；偶有约 20 ms 的短时抖动后恢复，未发现持续队列积压 | 适合鼠标操作、办公、演示等交互场景 |
| **4K：3840×2160 / 目标 60 FPS** | 该电视的解码能力成为瓶颈，出现解码队列溢出、画面帧积压和鼠标延迟；一次采样接收约 **43.54 FPS**、渲染约 **39.63 FPS**、平均解码约 **161 ms**，当时网络丢帧为 **0%** | 对延迟不敏感的**非交互场景可以尝试**；需要接受画面延迟，交互操作优先使用 2K60 |

以上是该核显电脑与该电视组合的短时实测，不是所有设备的性能保证。4K 受限于这台电视的解码硬件，不能据此认定是电脑核显性能不足，也不能保证其他电视同样表现。实测主机版本与离线包固定的 Sunshine 版本不同，离线包的全新安装组合仍需另行验证。

## 状态刷新与诊断

- 只在启动、手动刷新和管理操作完成后读取状态，**没有后台定时轮询**。
- 读取或操作期间使用居中弹出遮罩，保留页面原位置。
- Sunshine 日志只在本机查看，不自动上传。
- 可选安卓采样：自行安装 Android platform-tools，电视授权调试，在“应用设置”填写电视地址及 ADB 路径，开启 Moonlight 性能信息后点击“读取电视性能”。
- ADB 不随包提供，不影响正常投屏。采样为手动触发，近期错误日志可能来自之前的串流会话。

## 配置、权限与限制

- Sunshine：`%ProgramFiles%\Sunshine`；服务名：`SunshineService`。
- 已适配虚拟屏配置：`C:\VirtualDisplayDriver\vdd_settings.xml`。
- 应用设置和安装日志：`%LOCALAPPDATA%\HDScreenCast`。
- 备份与原文件放在一起：`sunshine.conf.backup-<时间戳>`、`vdd_settings.xml.backup-<时间戳>`。
- 关闭应用会保留服务和驱动状态，需要停止时使用对应按钮。
- 系统修改需要管理员授权；Windows 要求重启时请手动重启，应用不会自动重启电脑。
- 当前 GUI 为中文，提供中英文文档。
- 尚无独立登录会话、多用户输入隔离、macOS 后端、托盘、自动更新和第三方组件自动卸载。
- 控制台未进行 Authenticode 代码签名；随附组件保持官方原样，各自适用自己的签名和许可证。

## 开发与打包

需要 Windows x64、较新的 Rust 和 Microsoft C++ 构建工具。

```text
cargo check --locked
cargo fmt --check
cargo build --release --locked
python -X utf8 scripts/prepare_packages.py --with-sources
python -X utf8 scripts/package_release.py
```

准备脚本下载固定版本官方组件并校验，不执行安装。打包脚本使用已有 release 可执行文件生成控制台包和离线包，不自行 build 或安装组件。

GUI 位于 `src/app.rs`，系统操作位于 `src/platform/windows.rs`，初始化位于 `src/platform/windows/setup.rs`，XML 编辑位于 `src/virtual_config.rs`。`packages-manifest.json` 固定依赖版本和校验值。

## 验证范围

发布前执行类型、格式检查和 Windows release 构建。开发电脑上已检查 GUI 屏幕识别、页面切换、配置显示和遮罩刷新。全新 Windows 虚拟机安装、安装确认及重启行为、所有管理员失败恢复路径仍需更广泛验证。这是早期版本，构建成功不代表覆盖所有硬件。

## 协议与致谢

应用使用 [MIT](LICENSE)。组件保留自己的协议，见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) 和运行包中的 `licenses/`。

- [Sunshine](https://github.com/LizardByte/Sunshine)：GPL-3.0；官方原版 MSI，同一 Release 页面提供对应源码包。
- [Virtual Display Driver](https://github.com/VirtualDrivers/Virtual-Display-Driver)：MIT；官方已签名驱动。
- [Moonlight Android](https://github.com/moonlight-stream/moonlight-android)：电视另行安装。
- Microsoft Visual C++ Redistributable：微软许可条款，安装器保持原样。

本项目是独立社区项目，不是 Sunshine、Moonlight、Microsoft 或 VirtualDrivers 官方产品。
