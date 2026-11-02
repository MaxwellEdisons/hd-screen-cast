# Third-party software / 第三方组件说明

HD Screen Cast is an independent MIT-licensed management application. Installers
and drivers are separate, unmodified programs and retain their own licenses.

| Component | Pinned release | License / terms | Source |
| --- | --- | --- | --- |
| Sunshine | v2026.914.233613 | GNU GPL v3 | https://github.com/LizardByte/Sunshine/tree/v2026.914.233613 |
| Virtual Display Driver | 25.7.23; INF 11.30.4.434 | MIT | https://github.com/VirtualDrivers/Virtual-Display-Driver/tree/25.7.23 |
| Microsoft Visual C++ x64 Redistributable | SHA-256 pinned in packages-manifest.json | Microsoft software license terms | https://learn.microsoft.com/cpp/windows/latest-supported-vc-redist |

Release `licenses/` contains upstream license texts and Rust dependency notices.
`packages-manifest.json` records provenance and hashes. Moonlight APKs and ADB
are not redistributed.

## Sunshine corresponding source

The Sunshine MSI is unmodified. Its source, build files, and checked-out recursive
submodule sources are offered as
`Sunshine-v2026.914.233613-corresponding-source.tar.gz` on the same GitHub Release
page. `SOURCE-REVISION.txt` records revisions. Follow upstream build instructions
and dependency licenses. HD Screen Cast communicates with the service and does
not compile Sunshine source into its executable.

## Microsoft runtime

The official `vc_redist.x64.exe` prerequisite installer is distributed unmodified.
Microsoft license terms apply to installation and use. It is not MIT-licensed
HD Screen Cast source code.

## 中文

应用本体 MIT；Sunshine、驱动和微软运行库分别保留自己的协议。安装器保持官方原样，
版本和哈希固定在 packages-manifest.json。Sunshine 对应源码及递归子模块在同一
Release 页面提供。Moonlight APK 与 ADB 由用户另行从官方渠道获取。
