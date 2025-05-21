use crate::AppNotice;
use anyhow::anyhow;
use minio_rsc::Minio;
use minio_rsc::provider::StaticProvider;
use tokio::fs;
use tokio::io::AsyncReadExt;

const ACCESS_KEY: &str = "GMP4t8c8hhvLg0Yp";
const SECRET_KEY: &str = "HIl2Rq4coFPSMLPdupsbw7RsFjdgIfeK";
const END_POINT: &str = "192.168.1.17:9000";

const COS_KEY_FORMATTER: &str = "/global/data/csv/";

pub async fn minio_upload(
    input_dir: String,
    bucket_name: String,
    sender: tokio::sync::mpsc::UnboundedSender<AppNotice>,
) -> Result<(), Box<dyn std::error::Error + Send>> {
    let provider = StaticProvider::new(ACCESS_KEY, SECRET_KEY, None);
    let minio = Minio::builder()
        .endpoint(END_POINT)
        .provider(provider)
        .region("ap-chengdu")
        .secure(false)
        .build()
        .map_err(|e| anyhow!("Error building minio provider: {}", e))?;

    let have = minio
        .bucket_exists(&bucket_name)
        .await
        .map_err(|e| anyhow!("Error checking bucket existence: {}", e))?;
    // Check if bucket exists
    if !have {
        minio
            .make_bucket(&bucket_name, true)
            .await
            .map_err(|e| anyhow!("Error make bucket: {}", e))?;
    }

    // 先统计csv文件总数
    let mut total_csv_files = 0;
    let mut count_entries = match fs::read_dir(input_dir.clone()).await {
        Ok(entries) => entries,
        Err(e) => {
            sender
                .send(AppNotice::Toast((
                    format!("Failed to read directory {}: {}", input_dir, e),
                    5,
                )))
                .unwrap();
            return Err(anyhow::anyhow!("Failed to read directory {}", input_dir).into());
        }
    };

    while let Some(entry) = count_entries
        .next_entry()
        .await
        .map_err(|e| anyhow!("Failed to read directory entry: {}", e))?
    {
        if entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "csv") {
            total_csv_files += 1;
        }
    }

    sender
        .send(AppNotice::SyncServerProgress(
            0,
            total_csv_files,
            String::default(),
        ))
        .unwrap();

    let mut current_file = 0;
    let mut entries = match fs::read_dir(input_dir.clone()).await {
        Ok(entries) => entries,
        Err(e) => {
            sender
                .send(AppNotice::Toast((
                    format!("Failed to read directory {}: {}", input_dir, e),
                    5,
                )))
                .unwrap();
            return Err(anyhow::anyhow!("Failed to read directory {}", input_dir).into());
        }
    };

    while let Some(entry) = match entries.next_entry().await {
        Ok(entry) => entry,
        Err(e) => {
            sender
                .send(AppNotice::Toast((
                    format!("Failed to read directory entry: {}", e),
                    5,
                )))
                .unwrap();
            return Err(anyhow::anyhow!("Failed to read directory entry").into());
        }
    } {
        let path = entry.path();
        if path.is_file() {
            let file_name = path.file_name().unwrap().to_string_lossy().into_owned();
            let dest_path = path.to_string_lossy().into_owned();
            let src_key = format!("{}{}", COS_KEY_FORMATTER, file_name);

            let mut file = match fs::File::open(&path).await {
                Ok(file) => file,
                Err(e) => {
                    sender
                        .send(AppNotice::Toast((
                            format!("Failed to open file {}: {}", dest_path, e),
                            5,
                        )))
                        .unwrap();
                    continue;
                }
            };

            let mut contents = vec![];
            if let Err(e) = file.read_to_end(&mut contents).await {
                sender
                    .send(AppNotice::Toast((
                        format!("Failed to read file {}: {}", dest_path, e),
                        5,
                    )))
                    .unwrap();
                continue;
            }
            if path.extension().map_or(false, |ext| ext == "csv") {
                current_file += 1;
                sender
                    .send(AppNotice::SyncServerProgress(
                        current_file,
                        total_csv_files,
                        dest_path,
                    ))
                    .unwrap();
            }
            minio
                .put_object(&bucket_name, &src_key, contents.into())
                .await
                .map_err(|e| anyhow!("{}", e))?;
        }
    }

    Ok(())
}
