use crate::AppNotice;
use eframe::egui;

pub fn server_restart_ui(app: &mut crate::App, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("重启");
        ui.add_space(10.0);

        if app.server_restart {
            ui.spinner();
        }
    });

    ui.horizontal(|ui| {
        if ui.button("DEV").clicked() {
            let sender = app.notice_sender.clone().unwrap();
            tokio::spawn(async move {
                if let Err(e) = crate::ssh_utils::restart_server("dev", &sender).await {
                    sender
                        .send(AppNotice::ToastErr((format!("重启DEV失败:{:?}", e), 5)))
                        .unwrap();
                };
            });
        };
        ui.add_space(10.0);
        if ui.button("TEST").clicked() {
            let sender = app.notice_sender.clone().unwrap();
            tokio::spawn(async move {
                if let Err(e) = crate::ssh_utils::restart_server("test", &sender).await {
                    sender
                        .send(AppNotice::ToastErr((format!("重启TEST失败:{:?}", e), 5)))
                        .unwrap();
                };
            });
        };
    });
}

pub fn server_log_ui(app: &mut crate::App, ui: &mut egui::Ui) {
    ui.label("查看日志");
    ui.horizontal(|ui| {
        if ui.button("DEV").clicked() {
            let sender = app.notice_sender.clone().unwrap();
            tokio::spawn(async move {
                if let Err(e) = crate::ssh_utils::server_log("dev", &sender).await {
                    sender
                        .send(AppNotice::ToastErr((format!("查看DEV日志失败:{:?}", e), 5)))
                        .unwrap();
                };
            });
        };
        ui.add_space(10.0);
        if ui.button("TEST").clicked() {
            let sender = app.notice_sender.clone().unwrap();
            tokio::spawn(async move {
                if let Err(e) = crate::ssh_utils::server_log("test", &sender).await {
                    sender
                        .send(AppNotice::ToastErr((
                            format!("重启TEST日志失败:{:?}", e),
                            5,
                        )))
                        .unwrap();
                };
            });
        };
    });
}
