use calamine::{Reader, Xlsx, open_workbook};
use serde_json;
use std::{
    fs,
    io::{self, Error},
    path::{self, PathBuf},
};

pub struct Xlsx2CsvTool {
    pub input_dir: String,
    pub output_server_dir: PathBuf,
    pub output_client_dir: PathBuf,
    pub files: Vec<String>,
    pub progress_callback: Option<Box<dyn Fn(i32, i32, String) + Send>>,
}

impl Xlsx2CsvTool {
    pub fn new(input_dir: String, output_dir: String, files: Vec<String>) -> Self {
        Self {
            input_dir,
            output_server_dir: PathBuf::from(&output_dir).join("server"),
            output_client_dir: PathBuf::from(&output_dir).join("client"),
            files,
            progress_callback: None,
        }
    }

    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(i32, i32, String) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
    }

    pub fn exec(self) -> io::Result<()> {
        // 清理输出目录
        if self.output_server_dir.exists() {
            fs::remove_dir_all(&self.output_server_dir)?;
        }
        if self.output_client_dir.exists() {
            fs::remove_dir_all(&self.output_client_dir)?;
        }

        // 创建输出目录
        fs::create_dir_all(&self.output_server_dir)?;
        fs::create_dir_all(&self.output_client_dir)?;

        // 读取输入目录
        let entries = fs::read_dir(&self.input_dir)?;

        // 处理每个xlsx文件
        let total_files = self.files.len() as i32;
        let mut processed_files = 0i32;

        // 初始化进度
        if let Some(ref callback) = self.progress_callback {
            callback(processed_files, total_files, String::from(""));
        }

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            // 跳过隐藏文件和非xlsx文件
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.starts_with(".") || file_name.starts_with("~") {
                    continue;
                }

                if !self.files.is_empty() && !self.files.contains(&file_name.to_string()) {
                    continue;
                }

                if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
                    if extension.eq_ignore_ascii_case("xlsx") {
                        processed_files += 1;
                        if let Some(ref callback) = self.progress_callback {
                            callback(
                                processed_files,
                                total_files,
                                path.to_owned().to_string_lossy().to_string(),
                            );
                        }

                        self.process_xlsx(&path)
                            .map_err(|e| Error::new(io::ErrorKind::Other, e.to_string()))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn process_xlsx(&self, xlsx_path: &path::Path) -> io::Result<()> {
        let mut workbook: Xlsx<_> = open_workbook(xlsx_path).map_err(|e| {
            Error::new(
                io::ErrorKind::Other,
                format!("Failed to open workbook: {}", e),
            )
        })?;

        let base_name = xlsx_path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::new(io::ErrorKind::Other, "Invalid file name"))?;

        // 处理每个工作表
        for sheet_name in workbook.sheet_names().to_owned() {
            if let Some(Ok(range)) = workbook.worksheet_range(&sheet_name) {
                self.process_sheet(base_name, &sheet_name, range)?;
            }
        }

        Ok(())
    }

    fn process_sheet(
        &self,
        base_name: &str,
        sheet_name: &str,
        range: calamine::Range<calamine::DataType>,
    ) -> io::Result<()> {
        let rows: Vec<_> = range.rows().collect();
        if rows.is_empty() {
            return Ok(());
        }

        // 解析标题头和输出类型
        let mut for_server = false;
        let mut for_client = false;
        let mut line_limit = 0;

        if let Some(first_cell) = rows[0].first() {
            let meta = first_cell.to_string();
            let metas: Vec<&str> = meta.split('#').collect();
            let csv_name = metas[0].to_string();

            if metas.len() == 1 {
                for_server = true;
                for_client = true;
            } else {
                if metas.len() > 1 && metas[1] == "1" {
                    for_server = true;
                }
                if metas.len() > 2 && metas[2] == "1" {
                    for_client = true;
                }
                if metas.len() > 3 {
                    line_limit = metas[3].parse().unwrap_or(0);
                }
            }

            if line_limit > 0 && line_limit < 3 {
                return Err(Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "CHECK SERVER CSV FILE: <<{}>> - {} ERROR: 行数限制不能小于3, 前3行是必须的头; 不限制请设置为0",
                        base_name, sheet_name
                    ),
                ));
            }

            let strip_quot = csv_name == "Language" || csv_name == "BadWords";
            let ts_export = csv_name == "Language";

            let server_csv = self.output_server_dir.join(format!("{}.csv", csv_name));
            let client_csv = self.output_client_dir.join(format!("{}.csv", csv_name));

            let mut server_content = String::new();
            let mut client_content = String::new();
            let mut ts_text = Vec::new();

            let mut unexport_cell_index = std::collections::HashSet::new();
            let mut title_mapping = std::collections::HashMap::new();

            for (index, row) in rows.iter().enumerate() {
                if line_limit > 0 && index + 1 > line_limit {
                    break;
                }

                let mut types_for_client = Vec::new();
                let mut contents = Vec::new();
                let mut data_mapping = if ts_export && index > 2 {
                    Some(std::collections::HashMap::new())
                } else {
                    None
                };
                let mut row_key = String::new();

                for (cell_index, cell) in row.iter().enumerate() {
                    let mut value = cell.to_string();

                    if index == 1 {
                        title_mapping.insert(cell_index, value.clone());
                    }

                    if index == 0 {
                        value = value
                            .replace(",", " ")
                            .replace("\r\n", " ")
                            .replace("\n", " ");

                        if value.starts_with("UNEXPORT_") || value.trim().is_empty() {
                            unexport_cell_index.insert(cell_index);
                            continue;
                        }
                    } else {
                        value = value
                            .replace(",", "，")
                            .replace("\r\n", "\n")
                            .replace("\n", "\\n");
                    }

                    if unexport_cell_index.contains(&cell_index) {
                        continue;
                    }

                    if index == 1 && value.trim().is_empty() {
                        unexport_cell_index.insert(cell_index);
                        continue;
                    }

                    if index > 2 && cell_index == 0 {
                        row_key = value.clone();
                    }

                    if index > 2 && ts_export && cell_index > 0 {
                        if let Some(ref mut mapping) = data_mapping {
                            if let Some(title) = title_mapping.get(&cell_index) {
                                mapping.insert(title.clone(), value.clone());
                            }
                        }
                    }

                    if index == 2 {
                        if value.contains("#") {
                            let values: Vec<&str> = value.split('#').collect();
                            types_for_client.push(values[1].to_string());
                            value = values[0].to_string();
                        } else {
                            types_for_client.push(value.clone());
                        }
                    }

                    if csv_name == "BadWords" {
                        value = value.replace("\"", "");
                    }

                    if strip_quot
                        && (value.contains("\\n") || value.contains(",") || value.contains("，"))
                    {
                        value = value.replace("\"", "\"\"");
                        value = format!("\"{}\"", value);
                    }

                    contents.push(value);
                }

                let row_content = contents.join(",") + "\n";
                if !row_content.starts_with(",") {
                    server_content.push_str(&row_content);
                }
                if index > 0 {
                    let client_row = if !types_for_client.is_empty() {
                        types_for_client.join(",") + "\n"
                    } else {
                        row_content.clone()
                    };
                    if !client_row.starts_with(",") {
                        client_content.push_str(&client_row);
                    }
                }

                if ts_export && index > 2 {
                    if let Some(mapping) = data_mapping {
                        if let Ok(json_str) = serde_json::to_string(&mapping) {
                            let json_str = json_str.replace("\\\\n", "\\n");
                            ts_text.push(format!("\"{}\": {},", row_key, json_str));
                        }
                    }
                }
            }

            if for_server {
                fs::write(&server_csv, server_content)?;
            }

            if for_client {
                fs::write(&client_csv, client_content)?;
            }

            if ts_export && !ts_text.is_empty() {
                let ts_content = format!(
                    "let CXTranslationText: Record<string, Record<string, string>> = {{\n{}\n}};\nexport {{ CXTranslationText }};\n",
                    ts_text.join("\n")
                );
                let ts_file = PathBuf::from(&self.output_client_dir).join("CXTranslationText.ts");
                fs::write(ts_file, ts_content)?;
            }
        }

        Ok(())
    }
}
