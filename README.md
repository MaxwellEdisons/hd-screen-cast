# HD Screen Cast · 高清投屏

**High-resolution PC-to-TV desktop streaming with independent virtual displays.**

[简体中文](README.zh-CN.md) · [Download](https://github.com/MaxwellEdisons/hd-screen-cast/releases/latest) · [Moonlight Android APK](https://github.com/moonlight-stream/moonlight-android/releases)

HD Screen Cast is a Rust desktop app that combines Sunshine installation, screen selection, virtual-display configuration, and local diagnostics. Your TV receives the desktop using Moonlight.

## Use cases

Built-in casting can offer limited resolution choices, an unsuitable aspect ratio, or only a copy of your primary monitor. HD Screen Cast provides another workflow:

- Share a selected physical monitor, or create an independent virtual desktop.
- Configure virtual resolutions and refresh rates independently of your physical monitor's resolution.
- Use the TV as an additional desktop for presentations, documents, browsing, video, and games.
- Use 1440p/4K modes when your host GPU, client decoder, TV, and network support them.

**Current release: Windows 10/11 x64. macOS is planned, not implemented.** This is not an AirPlay/Miracast implementation. It cannot bypass hardware or network limits. Virtual screens share the same Windows session, mouse, and keyboard.

```text
Windows PC: HD Screen Cast + Sunshine + optional virtual display
                          ↓ local network
TV / Android TV / TV box: Moonlight
```

## Downloads

Get files from [GitHub Releases](https://github.com/MaxwellEdisons/hd-screen-cast/releases/latest).

| Package | Intended use | Contents |
| --- | --- | --- |
| `HD-Screen-Cast-v0.3.0-windows-x64-Offline.zip` | A new Windows PC | App, official Sunshine MSI, signed VDD, Microsoft runtime installer, licenses and checksums |
| `HD-Screen-Cast-v0.3.0-windows-x64-Controller.zip` | Components already installed | App and documentation; no component installers |

**Extract the entire ZIP**, then run `HD-Screen-Cast.exe`. Keep `packages/` next to the executable in the Offline package. Drivers and services must be registered with Windows: copying the folder to another PC still requires first-time initialization.

The published GUI needs no Python, Rust, Node.js, or WebView installation. It uses the static C runtime; the bundled VC++ installer provides external component prerequisites.

## Complete setup instructions

### 1. Initialize the Windows PC

1. Download and extract the **Offline** package.
2. Run `HD-Screen-Cast.exe`, then open **首次初始化** (First-time setup).
3. Select missing components and click **一键初始化所选组件** (Initialize selected components).
4. Accept Windows administrator approval and complete any installer/driver-publisher confirmation windows yourself.
5. The app verifies package SHA-256 hashes, installs the runtime and selected missing components, and creates a default **2560×1440 / 60Hz** VDD configuration only if none exists. A newly installed virtual driver is disabled by default, leaving no idle extra desktop.
6. If Windows requests a restart, restart manually, reopen the app, and click **刷新状态** (Refresh).

Existing installations are skipped. The app does not import root certificates, disable driver signature enforcement, overwrite existing accounts, or copy another PC's pairing identity. After an installation failure, completed components remain installed; resolve the reported problem and retry.

### 2. Set up Sunshine

1. Connect PC and TV to the same router. Ethernet or a good 5GHz Wi-Fi connection is preferable.
2. Start the streaming service under **屏幕与串流** (Screens & streaming), if stopped.
3. Open [Sunshine local management](https://localhost:47990), or use the app's setup-account button.
4. Create the Sunshine username and password. The local interface uses a local certificate; check the `localhost` address and handle browser certificate prompts yourself.

### 3. Install and pair Moonlight on the TV

**Official Android APK downloads: [moonlight-stream/moonlight-android/releases](https://github.com/moonlight-stream/moonlight-android/releases).**

1. Install Moonlight from the Android TV/Google TV app store, or download the appropriate APK from that official page.
2. If the TV cannot download directly, transfer the APK using a supported USB or file-transfer method, then install it on the TV. Installation availability and permission requirements depend on the TV system.
3. Open Moonlight and select your PC. If discovery fails, add the PC's LAN IPv4 address manually.
4. Moonlight displays a PIN. Click **电视配对** (Pair TV) on the PC and enter that PIN in Sunshine.
5. After pairing, select **Desktop** in Moonlight.

Apple TV uses the tvOS Moonlight client. Samsung Tizen/LG webOS cannot install an Android APK; use an Android TV box or a client supported by that platform.

### 4. Choose a screen

- Physical monitor: click **共享此屏幕** (Share this screen) beside that display.
- Independent desktop: click **开启虚拟屏串流** (Start virtual-screen streaming). The app enables virtual screens on demand and binds a capture target; reconnect Desktop on the TV. The driver only needs to be installed once.
- Primary monitor: choose **跟随主屏** (Follow primary).

Changing the target/restarting a component can disconnect streaming. Reconnect Desktop on the TV. A new virtual desktop may show only wallpaper: move a window into it with Windows display arrangement or `Win + Shift + Arrow`.

### 5. Configure virtual screens

Open **虚拟屏配置** (Virtual display settings): choose **1–4 displays**, and add/edit/remove up to 12 resolution/refresh-rate modes. Saving creates a backup, restarts an enabled driver and running Sunshine, and re-resolves the virtual capture target. A disabled driver only receives saved configuration until enabled.

All virtual screens share the available-mode list. Select each screen's **active mode in Windows Display Settings**. Saving a supported mode does not change Moonlight's video resolution. Reducing the count may require a different capture target. Failures trigger configuration recovery attempts and report recovery errors.

### Release virtual screens when finished

Click **停止并释放虚拟屏** (Stop and release virtual screens), or exit the app normally. Its managed virtual-screen stream is ended and the virtual driver is disabled, removing the extra Windows desktop instead of leaving windows on an idle screen. It does not silently continue streaming the primary monitor. Exit cleanup may require UAC approval; cancellation or cleanup failure keeps the app open with an explanation.

Physical-monitor streaming can continue after closing the app. Force-terminating the process or losing power can bypass cleanup; reopen the app and use Stop and release virtual screens to recover.

## Quality and responsiveness

Configure this starting point **in Moonlight**:

```text
2560×1440 · 60 FPS · H.264 · HDR off · approximately 30 Mbps
```

This worked on the development TV, not necessarily every TV. If 4K is sharp but mouse response is delayed, inspect decoder time as well as network loss. The development TV accumulated a 4K H.264 decode backlog while 1440p60 worked better. HEVC/HDR compatibility depends on the client. The host app does not override Moonlight's codec preferences.

### Tested hardware and measurements (2026-10-30)

| Item | Test environment |
| --- | --- |
| Host GPU | **Intel Arc 130T integrated GPU**, using Intel Quick Sync hardware encoding; no discrete GPU required for this setup |
| TV | Xiaomi TV, ADB model **MiTV_ASTP0**; Amlogic hardware decoder `OMX.amlogic.avc.decoder.awesome` |
| Client / host | Moonlight Android **12.2** / Sunshine **2026.1028.122642** |
| Network | Both devices on the same router's 5GHz Wi-Fi; TV reported a 150 Mbps link rate, not measured throughput |
| Encoding | H.264, HDR off; approximately 31 Mbps at 1440p and 63 Mbps at 4K |

| Scenario | Observations | Suggested use |
| --- | --- | --- |
| **1440p: 2560×1440 / 60 FPS** | Stable during roughly three minutes of continuous observation: approximately **59–60 rendered FPS**, **0% network frame loss**, usually **1–3 ms decoding**, with an occasional brief rise to about 20 ms followed by recovery; no sustained decode backlog observed | Interactive desktop use, office work, and presentations |
| **4K: 3840×2160 / target 60 FPS** | This TV's decoder became the bottleneck, causing decode-queue overflows, queued frames, and mouse latency. One sample showed **43.54 received FPS**, **39.63 rendered FPS**, and approximately **161 ms average decoding**, with **0% network frame loss** | May be tried for **non-interactive, latency-tolerant content** if delayed output is acceptable; prefer 1440p60 for interaction |

These are short observations of one integrated-GPU PC/TV combination, not a universal benchmark. The observed 4K bottleneck was the TV decoder; it does not establish that integrated graphics are inherently insufficient. The tested Sunshine version differs from the version pinned in the Offline package, whose fresh-install combination requires separate validation.

## Diagnostics and refresh

- State is read at startup, on manual Refresh, and after management operations. **No periodic system polling.**
- A centered modal overlay keeps the existing page layout in place during operations.
- Sunshine logs remain local and are not uploaded automatically.
- Optional TV sampling: install Android platform-tools separately, authorize TV debugging, set TV address/ADB path in **应用设置**, enable Moonlight's performance overlay, then click **读取电视性能**.
- ADB is optional and not bundled. Sampling is manual; recent decode-error logs can belong to earlier sessions.

## Paths, permissions and limitations

- Sunshine: `%ProgramFiles%\Sunshine`, service `SunshineService`.
- Supported VDD settings: `C:\VirtualDisplayDriver\vdd_settings.xml`.
- App settings/install logs: `%LOCALAPPDATA%\HDScreenCast`.
- Backups sit beside the original: `sunshine.conf.backup-<timestamp>` / `vdd_settings.xml.backup-<timestamp>`.
- Normal exit releases virtual displays managed by this app. Physical-monitor streaming may continue. The Stop button stops Sunshine and releases virtual displays.
- Privileged changes require UAC approval. Windows may require a manual restart; the app never reboots automatically.
- Current GUI labels are Chinese; documentation is available in English and Chinese.
- No separate login session, multi-user input isolation, macOS backend, tray icon, automatic updates, or automatic third-party uninstaller yet.
- The app is not Authenticode-signed. Third-party files are unmodified official distributions with their own signatures/licenses.

## Development

Windows x64, a recent Rust toolchain, and Microsoft C++ build tools are required.

```text
cargo check --locked
cargo fmt --check
cargo build --release --locked
python -X utf8 scripts/prepare_packages.py --with-sources
python -X utf8 scripts/package_release.py
```

The preparation script downloads pinned official packages and verifies hashes without installing anything. Packaging uses an existing release executable; it does not run a build or install components.

`src/app.rs` contains the GUI; `src/platform/windows.rs` contains system integration; `src/platform/windows/setup.rs` implements initialization; `src/virtual_config.rs` edits driver XML while preserving unrelated options. `packages-manifest.json` pins dependency versions and hashes.

## Validation

Type/format checks and Windows release builds are run before publication. Screen detection, navigation, configuration display, and modal refresh have been checked on the development PC. On 2026-11-02, a live system-level start/stop cycle verified virtual-display creation, Sunshine binding to a 2560×1440 virtual screen, and removal of that screen with the service stopped afterward; TV frame rate was not remeasured in this cycle. Clean Windows VM installation, installer prompts/reboots, and all privileged recovery paths still require broader validation. This is an early release, not a claim of full hardware coverage.

## License and credits

Application: [MIT](LICENSE). Components remain separate software under their own licenses; see [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) and release `licenses/`.

- [Sunshine](https://github.com/LizardByte/Sunshine): GPL-3.0; unmodified official MSI. Corresponding source is offered on the same release page.
- [Virtual Display Driver](https://github.com/VirtualDrivers/Virtual-Display-Driver): MIT; official signed driver.
- [Moonlight Android](https://github.com/moonlight-stream/moonlight-android): installed separately on the TV.
- Microsoft Visual C++ Redistributable: Microsoft's license terms, unmodified prerequisite installer.

Independent community project; not an official Sunshine, Moonlight, Microsoft, or VirtualDrivers product.
