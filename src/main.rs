use eframe::egui;
use egui_notify::Toasts;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;

// 嵌入字体文件
static FONT_DATA: &[u8] = include_bytes!("../fonts/NotoSansMonoCJKsc-Regular.otf");

#[tokio::main]
async fn main() -> eframe::Result<()> {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(egui::vec2(1600.0, 900.0));
    eframe::run_native(
        "XdUtilApp",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct App {
    #[serde(skip)]
    current_tab: usize,
    excel_dir: String,
    output_dir: String,
    client_dir: String,
    server_dir: String,
    #[serde(skip)]
    files: Vec<String>,
    #[serde(skip)]
    selected_files: Vec<String>,
    #[serde(skip)]
    select_all: bool,
    #[serde(skip)]
    files_loaded: bool,
    #[serde(skip)]
    toasts: Toasts,
    #[serde(skip)]
    notice_receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<AppNotice>>>>,
    #[serde(skip)]
    notice_sender: Option<mpsc::UnboundedSender<AppNotice>>,
    #[serde(skip)]
    export_progress: Option<(i32, i32, String)>,
    #[serde(skip)]
    sync_server_progress: Option<(i32, i32, String)>,
    #[serde(skip)]
    server_restart: bool,
}

pub enum AppNotice {
    Toast((String, u64)),
    ToastErr((String, u64)),
    ExportProgress(i32, i32, String),
    SyncServerProgress(i32, i32, String),
    ServerRestartStart,
    ServerRestartComplete,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 配置中文字体支持
        Self::configure_fonts(&cc.egui_ctx);
        let (tx, rx) = mpsc::unbounded_channel();
        let mut app = Self::load_config().unwrap_or_else(|e| {
            eprintln!("加载配置失败: {}", e);
            Self::default()
        });
        app.notice_receiver = Arc::new(Mutex::new(Some(rx)));
        app.notice_sender = tx.into();
        app
    }

    fn configure_fonts(ctx: &eframe::egui::Context) -> Option<()> {
        // 使用开源的Noto Sans CJK字体
        let font_name = "Noto Sans CJK";

        // 使用嵌入的字体数据
        let font_data = eframe::egui::FontData::from_owned(FONT_DATA.to_vec());

        // 创建字体定义
        let mut font_def = eframe::egui::FontDefinitions::default();
        font_def
            .font_data
            .insert(font_name.to_string(), font_data.into());

        // 将字体添加到字体族中
        let font_family = eframe::epaint::FontFamily::Proportional;
        font_def
            .families
            .get_mut(&font_family)?
            .insert(0, font_name.to_string());

        // 应用字体设置
        ctx.set_fonts(font_def);
        Some(())
    }
}
mod export_files;
mod file_utils;
mod minio_uploader;
mod server;
mod ssh_utils;
mod sync_client;
mod sync_server;
mod xlsx2csv;

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理来自异步任务的消息
        if let Ok(mut receiver_guard) = self.notice_receiver.lock() {
            if let Some(receiver) = receiver_guard.as_mut() {
                while let Ok(msg) = receiver.try_recv() {
                    match msg {
                        AppNotice::Toast(msg) => {
                            self.toasts
                                .info(msg.0)
                                .duration(Duration::from_secs(msg.1).into());
                        }
                        AppNotice::ToastErr(msg) => {
                            self.toasts
                                .error(msg.0)
                                .duration(Duration::from_secs(msg.1).into());
                        }
                        AppNotice::ExportProgress(cur, total, text) => {
                            self.export_progress = Some((cur, total, text));
                        }
                        AppNotice::SyncServerProgress(cur, total, text) => {
                            self.sync_server_progress = Some((cur, total, text));
                        }
                        AppNotice::ServerRestartStart => {
                            self.server_restart = true;
                        }
                        AppNotice::ServerRestartComplete => {
                            self.server_restart = false;
                        }
                    };
                    ctx.request_repaint();
                }
            }
        }

        self.toasts.show(ctx);
        // 中央面板
        egui::CentralPanel::default().show(ctx, |ui| {
            // 创建一个固定高度的顶部区域用于放置Tab栏
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                // 增加Tab按钮的大小和间距
                let tab_size = egui::vec2(100.0, 30.0);
                let tab_spacing = 10.0;

                ui.add_space(tab_spacing);
                if ui
                    .add(
                        egui::Button::new("配置")
                            .selected(self.current_tab == 0)
                            .min_size(tab_size),
                    )
                    .clicked()
                {
                    self.current_tab = 0;
                }
                ui.add_space(tab_spacing);
                if ui
                    .add(
                        egui::Button::new("服务器")
                            .selected(self.current_tab == 1)
                            .min_size(tab_size),
                    )
                    .clicked()
                {
                    self.current_tab = 1;
                }
            });
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            match self.current_tab {
                0 => {
                    settings::settings_ui(self, ui);
                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);
                    export_files::export_files_ui(self, ui);
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    sync_client::sync_client_ui(self, ui);
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                    sync_server::sync_server_ui(self, ui);
                }
                1 => {
                    server::server_restart_ui(self, ui,true);
                    ui.separator();
                    ui.add_space(10.0);
                    server::server_log_ui(self, ui);
                    ui.separator();
                    ui.add_space(10.0);
                    server::server_restart_ui(self, ui,false);
                }
                _ => unreachable!(),
            }
        });
    }
}
impl App {
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = "xd-util.json";
        let json_string = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, json_string)?;
        Ok(())
    }

    fn load_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = "xd-util.json";
        let json_string = std::fs::read_to_string(config_path)?;
        let app: Self = serde_json::from_str(&json_string)?;
        Ok(app)
    }
}

mod settings;
