use crate::file_utils::{check_dir_path, check_file_exist, copy_dir_files_no_subdir, copy_file};
use eframe::egui;
use std::path::PathBuf;
use std::time::Duration;

pub fn sync_client_ui(app: &mut crate::App, ui: &mut egui::Ui) {
    ui.label("同步客户端");
    ui.add_space(5.0);

    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("同步客户端CSV&ts").min_size(egui::vec2(150.0, 30.0)))
            .clicked()
        {
            let csv_src_dir = PathBuf::from(&app.output_dir).join("client");
            if let Err(err) = check_dir_path(&csv_src_dir) {
                app.toasts
                    .error(err.to_string())
                    .duration(Duration::from_secs(5).into());
                return;
            }

            let ts_src_file = PathBuf::from(&csv_src_dir).join("CXTranslationText.ts");
            if !check_file_exist(&ts_src_file) {
                app.toasts
                    .error("ts文件不存在")
                    .duration(Duration::from_secs(5).into());
                return;
            }

            let ts_dst_file = PathBuf::from(&app.client_dir)
                .join("assets/scripts/framework/cx18n/CXTranslationText.ts");
            if let Err(err) = copy_file(&ts_src_file, &ts_dst_file) {
                app.toasts
                    .error(format!("复制ts文件失败: {}", err))
                    .duration(Duration::from_secs(5).into());
                return;
            }

            let csv_dst_dir = PathBuf::from(&app.client_dir).join("assets/csv");
            if let Err(err) = copy_dir_files_no_subdir(&csv_src_dir, &csv_dst_dir, |entry| {
                entry.path().extension().map_or(true, |ext| ext != "csv")
            }) {
                app.toasts
                    .error(format!("复制csv文件失败: {}", err))
                    .duration(Duration::from_secs(5).into());
                return;
            }

            app.toasts
                .success("同步完成")
                .duration(Duration::from_secs(5).into());
        }
    });
}
