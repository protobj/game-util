use crate::AppNotice;
use crate::xlsx2csv::Xlsx2CsvTool;
use eframe::egui;
use std::fs;

pub fn export_files_ui(app: &mut crate::App, ui: &mut egui::Ui) {
    ui.heading("导出文件");
    ui.add_space(10.0);

    // 文件列表区域

    ui.horizontal(|ui| {
        if ui.button("加载文件列表").clicked() && !app.excel_dir.is_empty() {
            // 从excel_dir读取实际的文件列表
            if let Ok(entries) = fs::read_dir(&app.excel_dir) {
                app.files.clear();
                for entry in entries {
                    if let Ok(entry) = entry {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name.ends_with(".xlsx") {
                                app.files.push(file_name.to_string());
                            }
                        }
                    }
                }
                app.files.sort(); // 按字母顺序排序
                app.files_loaded = true;
            } else {
                // 如果目录读取失败，显示错误信息
                app.files_loaded = false;
            }
        }

        if app.files_loaded {
            ui.add_space(20.0);
            if ui.checkbox(&mut app.select_all, "全选").clicked() {
                if app.select_all {
                    app.selected_files = app.files.clone();
                } else {
                    app.selected_files.clear();
                }
            }
        }
    });

    ui.add_space(10.0);

    // 导出按钮
    ui.horizontal(|ui| {
        if ui.button("开始导出").clicked() {
            let sender = app.notice_sender.clone().unwrap();
            let input_dir = app.excel_dir.clone();
            let output_dir = app.output_dir.clone();
            let selected_files = app.selected_files.clone();

            tokio::spawn(async move {
                let mut tool = Xlsx2CsvTool::new(input_dir, output_dir, selected_files);
                let progress_sender = sender.clone();
                tool.set_progress_callback(move |cur, total, text| {
                    match progress_sender.send(AppNotice::ExportProgress(cur, total, text)) {
                        Ok(_) => {}
                        Err(e) => {
                            progress_sender
                                .send(AppNotice::Toast((format!("发送进度失败:{}", e), 5)))
                                .unwrap();
                        }
                    }
                });
                match tool.exec() {
                    Ok(_) => sender
                        .send(AppNotice::Toast(("导出成功".to_string(), 5)))
                        .unwrap(),
                    Err(e) => sender
                        .send(AppNotice::Toast((format!("导出失败:{}", e), 5)))
                        .unwrap(),
                };
            });
        }
        ui.add_space(20.0);
        if let Some((cur, total, text)) = &app.export_progress {
            ui.add(
                egui::ProgressBar::new(*cur as f32 / *total as f32)
                    .text(text)
                    .show_percentage()
                    .animate(true),
            );
        }
    });

    ui.add_space(10.0);
    const NUM_COLUMNS: usize = 11; // 每行显示的列数
    // 使用流式布局显示文件列表
    if app.files_loaded {
        // 使用Grid布局显示文件列表
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("file_grid")
                .num_columns(NUM_COLUMNS) // 设置列数
                .spacing([10.0, 10.0]) // 设置间距
                .show(ui, |ui| {
                    for (i, file) in app.files.iter().enumerate() {
                        let mut is_selected = app.selected_files.contains(file);
                        if ui.checkbox(&mut is_selected, file).clicked() {
                            if is_selected {
                                app.selected_files.push(file.clone());
                            } else {
                                if let Some(pos) = app.selected_files.iter().position(|x| x == file)
                                {
                                    app.selected_files.remove(pos);
                                }
                            }
                            // 更新全选状态
                            app.select_all = app.selected_files.len() == app.files.len();
                        }
                        // 在每行的最后一个元素后换行
                        if (i + 1) % NUM_COLUMNS == 0 {
                            ui.end_row();
                        }
                    }
                    // 如果最后一行的元素数量不足列数，也需要换行
                    if app.files.len() % NUM_COLUMNS != 0 {
                        ui.end_row();
                    }
                });
        });
    } else {
        ui.label("请先加载文件列表");
    }
}
