use std::time::Duration;

use eframe::egui;
use rfd::FileDialog;

pub fn settings_ui(app: &mut crate::App, ui: &mut egui::Ui) {
    ui.heading("设置");
    ui.add_space(10.0);

    // 文件夹选择区域
    ui.vertical_centered(|ui| {
        // Excel目录
        ui.horizontal(|ui| {
            ui.label("Excel目录:(配置表/数据表)");
            ui.add_space(10.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.excel_dir)
                    .desired_width(ui.available_width() - 100.0),
            );
            if ui.button("选择").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    app.excel_dir = path.display().to_string();
                }
            }
        });

        ui.add_space(5.0);

        // 输出目录
        ui.horizontal(|ui| {
            ui.label("输出目录:(导表工具)");
            ui.add_space(10.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.output_dir)
                    .desired_width(ui.available_width() - 100.0),
            );
            if ui.button("选择").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    app.output_dir = path.display().to_string();
                }
            }
        });

        ui.add_space(5.0);

        // 客户端目录
        ui.horizontal(|ui| {
            ui.label("客户端目录:(xd-client)");
            ui.add_space(10.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.client_dir)
                    .desired_width(ui.available_width() - 100.0),
            );
            if ui.button("选择").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    app.client_dir = path.display().to_string();
                }
            }
        });

        ui.add_space(5.0);

        // 服务器目录
        ui.horizontal(|ui| {
            ui.label("服务器目录:(xd-server)");
            ui.add_space(10.0);
            ui.add(
                egui::TextEdit::singleline(&mut app.server_dir)
                    .desired_width(ui.available_width() - 100.0),
            );
            if ui.button("选择").clicked() {
                if let Some(path) = FileDialog::new().pick_folder() {
                    app.server_dir = path.display().to_string();
                }
            }
        });

        ui.add_space(10.0);
        if ui.button("保存配置").clicked() {
            if let Err(e) = app.save_config() {
                app.toasts.info(format!("保存配置失败: {}", e)).duration(Duration::from_secs(5).into());
            }else{
                app.toasts.info("保存配置成功").duration(Duration::from_secs(5).into());
            }
        }
    });
}