use crate::{
    model::{Action, DisplayMode, Preferences, Snapshot, VirtualConfig},
    platform,
};
use eframe::egui::{self, Color32, FontData, FontDefinitions, FontFamily, RichText, Stroke, Vec2};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

const GREEN: Color32 = Color32::from_rgb(119, 226, 174);
const MUTED: Color32 = Color32::from_rgb(150, 166, 183);
const PANEL: Color32 = Color32::from_rgb(24, 33, 44);

enum Event {
    Snapshot(Snapshot),
    Action(Result<String, String>, Snapshot),
    Metrics(Result<String, String>),
}

#[derive(Clone, Copy, PartialEq)]
enum Page {
    Setup,
    Screens,
    Virtual,
    Diagnostics,
    Settings,
}

pub struct ControlApp {
    state: Snapshot,
    preferences: Preferences,
    sender: Sender<Event>,
    receiver: Receiver<Event>,
    busy: bool,
    task: String,
    page: Page,
    notice: Option<(bool, String)>,
    metrics: String,
    has_snapshot: bool,
    virtual_draft: Option<VirtualConfig>,
    virtual_dirty: bool,
    applying_virtual: bool,
    install_sunshine: bool,
    install_driver: bool,
}

impl ControlApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = FontDefinitions::default();
        for path in [
            "C:/Windows/Fonts/msyh.ttc",
            "C:/Windows/Fonts/msyh.ttf",
            "C:/Windows/Fonts/simhei.ttf",
        ] {
            if let Ok(data) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert("chinese".into(), FontData::from_owned(data).into());
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .insert(0, "chinese".into());
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .push("chinese".into());
                break;
            }
        }
        cc.egui_ctx.set_fonts(fonts);
        cc.egui_ctx.set_theme(egui::ThemePreference::Dark);
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(15, 22, 31);
        visuals.window_fill = PANEL;
        visuals.selection.bg_fill = Color32::from_rgb(34, 89, 69);
        visuals.selection.stroke = Stroke::new(1.0, GREEN);
        cc.egui_ctx.set_visuals_of(egui::Theme::Dark, visuals);
        cc.egui_ctx.style_mut_of(egui::Theme::Dark, |style| {
            style.spacing.item_spacing = Vec2::new(12.0, 12.0);
            style.spacing.button_padding = Vec2::new(14.0, 9.0);
            for (kind, size) in [
                (egui::TextStyle::Body, 16.0),
                (egui::TextStyle::Button, 15.0),
                (egui::TextStyle::Small, 12.0),
                (egui::TextStyle::Heading, 22.0),
            ] {
                style
                    .text_styles
                    .insert(kind, egui::FontId::proportional(size));
            }
        });
        let (sender, receiver) = mpsc::channel();
        let mut app = Self {
            state: Snapshot::default(),
            preferences: platform::load_preferences(),
            sender,
            receiver,
            busy: false,
            task: String::new(),
            page: Page::Screens,
            notice: None,
            metrics: String::new(),
            has_snapshot: false,
            virtual_draft: None,
            virtual_dirty: false,
            applying_virtual: false,
            install_sunshine: true,
            install_driver: true,
        };
        app.refresh(&cc.egui_ctx);
        app
    }

    fn refresh(&mut self, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.task = "读取系统状态".into();
        let sender = self.sender.clone();
        let context = ctx.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::Snapshot(platform::snapshot()));
            context.request_repaint();
        });
    }

    fn act(&mut self, ctx: &egui::Context, action: Action, label: &str) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.task = label.into();
        self.notice = None;
        self.applying_virtual = matches!(action, Action::ConfigureVirtualDisplay { .. });
        let sender = self.sender.clone();
        let context = ctx.clone();
        thread::spawn(move || {
            let result = platform::execute(action);
            let snapshot = platform::snapshot();
            let _ = sender.send(Event::Action(result, snapshot));
            context.request_repaint();
        });
    }

    fn read_metrics(&mut self, ctx: &egui::Context) {
        if self.busy {
            return;
        }
        self.busy = true;
        self.task = "读取电视性能信息".into();
        let preferences = self.preferences.clone();
        let sender = self.sender.clone();
        let context = ctx.clone();
        thread::spawn(move || {
            let _ = sender.send(Event::Metrics(platform::television_metrics(&preferences)));
            context.request_repaint();
        });
    }

    fn opened(&mut self, target: &str) {
        if let Err(error) = platform::open(target) {
            self.notice = Some((false, error));
        }
    }

    fn sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("navigation")
            .exact_width(204.0)
            .show(ctx, |ui| {
                ui.add_space(25.0);
                ui.label(RichText::new("高清投屏").color(GREEN).strong().size(23.0));
                ui.label(RichText::new("串流控制台").color(MUTED).size(14.0));
                ui.add_space(35.0);
                for (page, label) in [
                    (Page::Setup, "首次初始化"),
                    (Page::Screens, "屏幕与串流"),
                    (Page::Virtual, "虚拟屏配置"),
                    (Page::Diagnostics, "运行诊断"),
                    (Page::Settings, "应用设置"),
                ] {
                    if ui
                        .add_sized(
                            [176.0, 45.0],
                            egui::Button::new(label).selected(self.page == page),
                        )
                        .clicked()
                    {
                        self.page = page;
                    }
                }
                ui.add_space(30.0);
                ui.separator();
                ui.label(RichText::new("WINDOWS EDITION").color(MUTED).small());
                ui.label(
                    RichText::new(concat!(env!("CARGO_PKG_VERSION"), " · 本地管理"))
                        .color(MUTED)
                        .small(),
                );
                ui.add_space(12.0);
                ui.label(
                    RichText::new("电视继续使用 Moonlight 连接。屏幕切换后需重新连接串流。")
                        .color(MUTED)
                        .small(),
                );
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(15.0);
                    if ui.button("打开 Sunshine 管理页面").clicked() {
                        self.opened("https://localhost:47990");
                    }
                    if ui.button("电视配对").clicked() {
                        self.opened("https://localhost:47990/pin");
                    }
                });
            });
    }

    fn header(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(match self.page {
                    Page::Setup => "让新电脑准备好高清投屏。",
                    Page::Screens => "把合适的屏幕，交给电视。",
                    Page::Virtual => "为电视准备合适的独立桌面。",
                    Page::Diagnostics => "看清串流的运行状态。",
                    Page::Settings => "让控制台适合你的设备。",
                });
                ui.label(RichText::new("HD Screen Cast / Windows").color(MUTED));
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(!self.busy, egui::Button::new("刷新状态"))
                    .clicked()
                {
                    self.refresh(ctx);
                }
            });
        });
        ui.add_space(14.0);
        if let Some((ok, message)) = &self.notice {
            egui::Frame::new()
                .fill(if *ok {
                    Color32::from_rgb(23, 57, 45)
                } else {
                    Color32::from_rgb(74, 40, 37)
                })
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label(message);
                });
        }
        if let Some(error) = &self.state.error {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
        ui.add_space(10.0);
    }

    fn setup(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("安装所需组件");
        ui.label("完整离线包已经包含安装组件。初始化会申请管理员权限，并打开必要的 Windows 安装确认窗口。");
        ui.add_space(12.0);
        egui::Frame::new().fill(PANEL).inner_margin(18.0).corner_radius(10.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_enabled(!self.state.installed && !self.busy, egui::Checkbox::new(&mut self.install_sunshine, "Sunshine 串流服务"));
                ui.colored_label(if self.state.installed { GREEN } else { MUTED }, if self.state.installed { "已安装，不会覆盖" } else { "需要安装" });
            });
            ui.horizontal(|ui| {
                ui.add_enabled(!self.state.virtual_installed && !self.busy, egui::Checkbox::new(&mut self.install_driver, "Virtual Display Driver 虚拟屏驱动"));
                ui.colored_label(if self.state.virtual_installed { GREEN } else { MUTED }, if self.state.virtual_installed { "已安装，不会覆盖" } else { "可选，独立桌面需要" });
            });
            ui.label(RichText::new("安装前会校验组件 SHA-256；同时安装或修复 Microsoft Visual C++ x64 运行库。现有账号、配对和配置不会复制到其他电脑。").color(MUTED));
            ui.label(RichText::new("驱动需注册到 Windows，应用不会自动导入根证书或关闭签名检查。如 Windows 提示重启，请手动重启。").color(MUTED));
        });
        ui.add_space(12.0);
        if !self.state.packages_available {
            ui.colored_label(
                Color32::YELLOW,
                "当前目录缺少完整安装组件。请下载并完整解压 Offline 离线包，保留 packages 文件夹。",
            );
        }
        let needed = (self.install_sunshine && !self.state.installed)
            || (self.install_driver && !self.state.virtual_installed);
        if ui
            .add_enabled(
                !self.busy && self.has_snapshot && self.state.packages_available && needed,
                egui::Button::new("一键初始化所选组件"),
            )
            .clicked()
        {
            self.act(
                ctx,
                Action::Initialize {
                    sunshine: self.install_sunshine,
                    virtual_driver: self.install_driver,
                },
                "安装组件，请完成 Windows 安装提示",
            );
        }
        ui.add_space(18.0);
        ui.heading("接下来怎么用");
        ui.label("1. 电脑和电视连接同一路由器，优先使用有线网络或 5GHz Wi-Fi。\n2. 打开 Sunshine 管理页面，设置账号和密码。\n3. 电视安装 Moonlight，选择电脑，将 PIN 输入 Sunshine 配对页面。\n4. 在本应用选择实体屏或虚拟屏，在电视中启动 Desktop。\n5. 先从 2K / 60 FPS / H.264 / HDR 关闭开始，再根据解码性能调整。");
        ui.horizontal_wrapped(|ui| {
            if ui.button("设置 Sunshine 账号").clicked() {
                self.opened("https://localhost:47990");
            }
            if ui.button("电视配对").clicked() {
                self.opened("https://localhost:47990/pin");
            }
            if ui.button("Moonlight 安卓 APK 下载").clicked() {
                self.opened("https://github.com/moonlight-stream/moonlight-android/releases");
            }
        });
    }

    fn screens(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let ready = self.state.installed && !self.busy;
        egui::Frame::new()
            .fill(PANEL)
            .inner_margin(20.0)
            .corner_radius(12.0)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        let (label, color) = match self.state.service.as_str() {
                            "Running" => ("Sunshine 正在运行", GREEN),
                            "Stopped" => ("Sunshine 已停止", MUTED),
                            "Missing" => ("未检测到 Sunshine 服务", Color32::LIGHT_RED),
                            "" if !self.has_snapshot => ("正在检测 Sunshine", MUTED),
                            _ => ("Sunshine 状态待确认", Color32::YELLOW),
                        };
                        ui.label(RichText::new(label).color(color).strong().size(19.0));
                        let target = self
                            .state
                            .target_display()
                            .map(|display| {
                                format!(
                                    "当前共享目标：{} · {} × {}",
                                    display.name, display.width, display.height
                                )
                            })
                            .unwrap_or_else(|| {
                                if self.state.target.is_empty() {
                                    "共享目标：跟随 Windows 主屏".into()
                                } else {
                                    "共享目标未连接，请选择可用屏幕".into()
                                }
                            });
                        ui.label(RichText::new(target).color(MUTED));
                        ui.label(
                            RichText::new("服务运行不代表电视已连接；实际画面以 Moonlight 为准。")
                                .color(MUTED)
                                .small(),
                        );
                    });
                });
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            ready && self.state.service == "Stopped",
                            egui::Button::new("启动串流服务"),
                        )
                        .clicked()
                    {
                        self.act(ctx, Action::Start, "启动 Sunshine");
                    }
                    if ui
                        .add_enabled(
                            ready && self.state.running(),
                            egui::Button::new("停止串流服务"),
                        )
                        .clicked()
                    {
                        self.act(ctx, Action::Stop, "停止 Sunshine");
                    }
                    if ui
                        .add_enabled(ready && self.state.running(), egui::Button::new("重启服务"))
                        .clicked()
                    {
                        self.act(ctx, Action::Restart, "重启 Sunshine");
                    }
                });
            });
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.heading("选择共享屏幕");
            ui.label(RichText::new(format!("{} 块已连接", self.state.displays.len())).color(MUTED));
        });
        ui.label(
            RichText::new("应用只修改 Sunshine 的捕获目标，不会更改你的 Windows 主屏。")
                .color(MUTED),
        );
        let displays = self.state.displays.clone();
        for display in displays {
            let selected = self.state.selected(&display);
            egui::Frame::new()
                .fill(PANEL)
                .stroke(Stroke::new(
                    1.0,
                    if selected {
                        GREEN
                    } else {
                        Color32::from_rgb(43, 55, 69)
                    },
                ))
                .inner_margin(16.0)
                .corner_radius(9.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(if display.name.is_empty() {
                                        &display.device_name
                                    } else {
                                        &display.name
                                    })
                                    .strong()
                                    .size(17.0),
                                );
                                if display.virtual_display {
                                    ui.colored_label(GREEN, "独立虚拟屏");
                                }
                                if display.primary {
                                    ui.colored_label(MUTED, "主屏");
                                }
                                if selected {
                                    ui.colored_label(GREEN, "当前目标");
                                }
                            });
                            ui.label(
                                RichText::new(format!(
                                    "{} × {}  /  {} Hz",
                                    display.width, display.height, display.refresh_hz
                                ))
                                .color(MUTED),
                            );
                            if display.id.is_none() {
                                ui.label(
                                    RichText::new("Sunshine 尚未识别此屏幕，请重启服务后刷新。")
                                        .color(Color32::YELLOW)
                                        .small(),
                                );
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(
                                    ready && !selected && display.id.is_some(),
                                    egui::Button::new("共享此屏幕"),
                                )
                                .clicked()
                            {
                                self.act(
                                    ctx,
                                    Action::SelectDisplay {
                                        id: display.id.clone().unwrap_or_default(),
                                    },
                                    "切换共享屏幕",
                                );
                            }
                        });
                    });
                });
        }
        ui.add_space(12.0);
        egui::Frame::new().fill(PANEL).inner_margin(18.0).corner_radius(10.0).show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.heading("独立虚拟屏");
            let description = if !self.state.virtual_installed { "尚未安装 MTT Virtual Display Driver。安装后可在此启用和关闭。" } else if self.state.virtual_enabled { "驱动已启用。选择上面的虚拟屏，即可把窗口单独移到电视。" } else { "驱动已安装，当前关闭。启用后会新增一块扩展屏。" };
            ui.label(description);
            ui.label(RichText::new("关闭正在共享的虚拟屏时，会先将共享目标切回主屏。鼠标、键盘与登录会话共用。").color(MUTED).small());
            ui.horizontal(|ui| {
                if ui.add_enabled(!self.busy && self.state.virtual_installed && !self.state.virtual_enabled, egui::Button::new("启用虚拟屏")).clicked() { self.act(ctx, Action::SetVirtualDisplay { enabled: true }, "启用虚拟屏"); }
                if ui.add_enabled(!self.busy && self.state.virtual_installed && self.state.virtual_enabled, egui::Button::new("关闭虚拟屏")).clicked() { self.act(ctx, Action::SetVirtualDisplay { enabled: false }, "关闭虚拟屏"); }
                if ui.button("Windows 显示设置").clicked() { self.opened("ms-settings:display"); }
                if ui.button("数量与画面配置").clicked() { self.page = Page::Virtual; }
                if ui.add_enabled(ready && !self.state.target.is_empty(), egui::Button::new("跟随主屏")).clicked() { self.act(ctx, Action::SelectDisplay { id: String::new() }, "恢复主屏共享"); }
            });
        });
        ui.add_space(10.0);
        ui.label(RichText::new("已验证的电视端配置：1440p / 60 FPS / H.264 / HDR 关闭 / 约 31 Mbps。请在 Moonlight 中设置，主机不会覆盖这些参数。").color(MUTED));
    }

    fn virtual_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("虚拟屏数量");
        ui.label("增加的屏幕会作为 Windows 扩展桌面；Sunshine 每次共享其中一块屏幕。");
        if let Some(error) = &self.state.virtual_config_error {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
        let Some(mut draft) = self.virtual_draft.clone() else {
            ui.label("读取到受支持的虚拟屏驱动配置后，可以在这里编辑。");
            return;
        };
        let previous = draft.clone();
        ui.add_enabled_ui(!self.busy, |ui| {
            ui.horizontal(|ui| {
                ui.label("数量");
                ui.add(egui::DragValue::new(&mut draft.count).range(1..=4).suffix(" 块"));
                ui.label(RichText::new("此版本支持 1～4 块").color(MUTED));
            });
            ui.add_space(18.0);
            ui.heading("可用画面模式");
            ui.label("这些模式供所有虚拟屏使用。每块屏的当前分辨率和刷新率，在 Windows 显示设置中选择。");
            ui.label(RichText::new("先保留当前的 2560 × 1440 / 60Hz，再按需要添加其他模式。电视端画质仍在 Moonlight 中设置。").color(MUTED));
            let mut remove = None;
            let can_remove = draft.modes.len() > 1;
            for (index, mode) in draft.modes.iter_mut().enumerate() {
                ui.push_id(index, |ui| {
                    egui::Frame::new().fill(PANEL).inner_margin(12.0).corner_radius(8.0).show(ui, |ui| {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("模式 {}", index + 1));
                            ui.add(egui::DragValue::new(&mut mode.width).range(640..=7680).speed(8.0));
                            ui.label("×");
                            ui.add(egui::DragValue::new(&mut mode.height).range(480..=4320).speed(8.0));
                            ui.add(egui::DragValue::new(&mut mode.refresh_hz).range(24..=240).suffix(" Hz"));
                            if ui.add_enabled(can_remove, egui::Button::new("移除")).clicked() { remove = Some(index); }
                        });
                    });
                });
            }
            if let Some(index) = remove { draft.modes.remove(index); }
            ui.horizontal_wrapped(|ui| {
                for (label, width, height) in [("添加 1080p / 60Hz", 1920, 1080), ("添加 2K / 60Hz", 2560, 1440), ("添加 4K / 60Hz", 3840, 2160)] {
                    let mode = DisplayMode { width, height, refresh_hz: 60 };
                    if ui.add_enabled(draft.modes.len() < 12 && !draft.modes.contains(&mode), egui::Button::new(label)).clicked() { draft.modes.push(mode); }
                }
            });
        });
        if draft != previous {
            self.virtual_dirty = true;
        }
        self.virtual_draft = Some(draft.clone());
        ui.add_space(16.0);
        ui.label(RichText::new("保存前会备份配置。驱动启用时，应用会重启虚拟屏和正在运行的 Sunshine，串流将短暂断开；虚拟屏数量减少时，会重新选择可用目标。其他驱动选项保持不变。").color(MUTED));
        let valid = draft.validate();
        if let Err(error) = &valid {
            ui.colored_label(Color32::LIGHT_RED, error);
        }
        ui.horizontal(|ui| {
            let changed = self.state.virtual_config.as_ref() != Some(&draft);
            if ui
                .add_enabled(
                    !self.busy && self.state.virtual_installed && changed && valid.is_ok(),
                    egui::Button::new("保存并应用配置"),
                )
                .clicked()
            {
                self.act(
                    ctx,
                    Action::ConfigureVirtualDisplay { config: draft },
                    "应用虚拟屏配置",
                );
            }
            if ui
                .add_enabled(
                    !self.busy && self.virtual_dirty,
                    egui::Button::new("撤销未保存修改"),
                )
                .clicked()
            {
                self.virtual_dirty = false;
                self.virtual_draft.clone_from(&self.state.virtual_config);
            }
            if ui.button("Windows 显示设置").clicked() {
                self.opened("ms-settings:display");
            }
        });
    }

    fn diagnostics(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading("电视性能采样（可选）");
        ui.label("通过已授权的 ADB 读取 Moonlight 性能信息，不会清除日志或修改电视画质。");
        ui.label(
            RichText::new("先在应用设置填写电视地址，在 Moonlight 中开启性能信息，并进入串流。")
                .color(MUTED),
        );
        if ui
            .add_enabled(!self.busy, egui::Button::new("读取电视性能"))
            .clicked()
        {
            self.read_metrics(ctx);
        }
        if !self.metrics.is_empty() {
            egui::Frame::new()
                .fill(PANEL)
                .inner_margin(15.0)
                .show(ui, |ui| {
                    ui.label(&self.metrics);
                });
        }
        ui.add_space(15.0);
        ui.heading("Sunshine 最近日志");
        ui.label(
            RichText::new(
                "日志不会自动上传，也不会定时读取。点击刷新状态获取最新日志；电视性能为手动采样。",
            )
            .color(MUTED),
        );
        let mut log = self.state.log.as_str();
        ui.add(
            egui::TextEdit::multiline(&mut log)
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(20),
        );
    }

    fn settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("电视诊断连接");
        ui.label("电视 IP 地址（可带端口）");
        ui.add(
            egui::TextEdit::singleline(&mut self.preferences.tv_address)
                .hint_text("192.168.1.100:5555")
                .desired_width(420.0),
        );
        ui.label("ADB 程序路径（留空使用系统 PATH）");
        ui.add(
            egui::TextEdit::singleline(&mut self.preferences.adb_path)
                .hint_text("C:\\Android\\platform-tools\\adb.exe")
                .desired_width(600.0),
        );
        if ui.button("保存设置").clicked() {
            self.notice = Some(match platform::save_preferences(&self.preferences) {
                Ok(()) => (true, "设置已保存。".into()),
                Err(error) => (false, error),
            });
        }
        ui.add_space(20.0);
        ui.heading("工作方式");
        ui.label("• Sunshine 使用默认安装位置：Program Files\\Sunshine。\n• 应用常规运行无需管理员权限，初始化或修改服务、驱动和捕获目标时才申请授权。\n• 每次切换共享目标都会在 Sunshine 配置目录保留备份。\n• 完整离线包支持首次初始化；已安装组件会跳过。\n• 虚拟屏分辨率由驱动及 Windows 显示设置管理。\n• 关闭应用会保留当前串流和屏幕配置。停止服务请使用首页按钮。");
        ui.add_space(12.0);
        if ui.button("打开本地设置目录").clicked() {
            self.opened(&platform::data_dir().to_string_lossy());
        }
    }
}

impl eframe::App for ControlApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(event) = self.receiver.try_recv() {
            self.busy = false;
            match event {
                Event::Snapshot(state) => {
                    if !self.has_snapshot && (!state.installed || !state.virtual_installed) {
                        self.page = Page::Setup;
                    }
                    if !self.virtual_dirty {
                        self.virtual_draft.clone_from(&state.virtual_config);
                    }
                    self.state = state;
                    self.has_snapshot = true;
                }
                Event::Action(result, state) => {
                    if self.applying_virtual && result.is_ok() {
                        self.virtual_dirty = false;
                    }
                    self.applying_virtual = false;
                    if !self.virtual_dirty {
                        self.virtual_draft.clone_from(&state.virtual_config);
                    }
                    self.state = state;
                    self.has_snapshot = true;
                    self.notice = Some(match result {
                        Ok(message) => (true, message),
                        Err(error) => (false, error),
                    });
                }
                Event::Metrics(result) => match result {
                    Ok(metrics) => self.metrics = metrics,
                    Err(error) => self.notice = Some((false, error)),
                },
            }
        }
        self.sidebar(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(18.0);
                self.header(ui, ctx);
                match self.page {
                    Page::Setup => self.setup(ui, ctx),
                    Page::Screens => self.screens(ui, ctx),
                    Page::Virtual => self.virtual_settings(ui, ctx),
                    Page::Diagnostics => self.diagnostics(ui, ctx),
                    Page::Settings => self.settings(ui),
                }
                ui.add_space(20.0);
            });
        });
        if self.busy {
            egui::Modal::new(egui::Id::new("operation-progress")).show(ctx, |ui| {
                ui.set_min_width(290.0);
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(format!("{}…", self.task));
                });
                ui.add_space(8.0);
                ui.label(RichText::new("完成后会自动更新状态。").color(MUTED));
            });
        }
    }
}
