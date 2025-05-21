use crate::AppNotice;
use eframe::egui;

pub const SERVER_MAP: &[(&str, &str)] = &[("olddev", "老服"), ("dev", "dev服"), ("cqdev", "CQ服")];

pub fn sync_server_ui(app: &mut crate::App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("同步服务器");
        ui.add_space(20.0);
        if let Some((cur, total, text)) = &app.sync_server_progress {
            ui.add(
                egui::ProgressBar::new(*cur as f32 / *total as f32)
                    .text(text)
                    .show_percentage()
                    .animate(true),
            );
        }
    });
    ui.horizontal(|ui| {
        SERVER_MAP.iter().for_each(|(server, label)| {
            if ui
                .add(
                    egui::Button::new(format!("[{}] {}", *server, *label))
                        .min_size(egui::vec2(150.0, 30.0)),
                )
                .clicked()
            {
                let output_dir = app.output_dir.clone();
                let server_name = server.to_string();
                let sender = app.notice_sender.clone().unwrap();
                let input_dir = format!("{}/server", output_dir);
                tokio::spawn(async move {
                    match crate::minio_uploader::minio_upload(
                        input_dir,
                        server_name.clone(),
                        sender.clone(),
                    )
                    .await
                    {
                        Ok(_) => {
                            let _ = sender.send(AppNotice::Toast((
                                format!("Upload to {} finished successfully.", server_name),
                                5,
                            )));
                        }
                        Err(e) => {
                            let _ = sender.send(AppNotice::Toast((
                                format!("Upload to {} failed: {}", server_name, e),
                                5,
                            )));
                        }
                    };
                });
            }
        });
    });
}
