use crate::AppNotice;
use myssh::client::{AuthMethod, Client, CommandExecutedResult, ServerCheckMethod};
use std::error::Error;
use tokio::sync::mpsc::UnboundedSender;

pub struct SshClient {
    client: Client,
}

#[derive(Debug)]
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

impl SshClient {
    pub async fn connect(config: SshConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let auth_method = if let Some(key_path) = config.key_path {
            AuthMethod::with_key_file(&key_path, config.password.as_deref())
        } else if let Some(password) = config.password {
            AuthMethod::with_password(&password)
        } else {
            return Err("需要提供密码或密钥文件".into());
        };

        let client = Client::connect(
            format!("{host}:{port}", host = config.host, port = config.port),
            &config.username,
            auth_method,
            ServerCheckMethod::NoCheck,
        )
        .await?;
        let ssh_client = Self { client };

        Ok(ssh_client)
    }
}

pub async fn restart_server(
    env: &str,
    sender: &UnboundedSender<AppNotice>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_ssh_client().await?;

    let commands: Vec<(String, String)> = vec![
        (
            "cd /xd-workspace/xd-server".to_string(),
            "执行cd命令".to_string(),
        ),
        (
            "git switch xd-20250304-dev".to_string(),
            "切换到分支".to_string(),
        ),
        ("git pull".to_string(), "执行git pull命令".to_string()),
        (
            format!("./upload_image_onekey_linux.sh {}", env).to_string(),
            "执行上传脚本".to_string(),
        ),
        (
            format!("cd /docker/xd-server-{}", env),
            "进入运行目录".to_string(),
        ),
        ("docker compose up -d".to_string(), "执行重启".to_string()),
    ];
    sender.send(AppNotice::ServerRestartStart)?;

    let cmd = commands
        .into_iter()
        .map(|x| format!(" {} ", x.0))
        .collect::<Vec<String>>()
        .join("&&");
    let rs = client.client.execute(&cmd).await?;

    show_res(sender, &rs, 5);
    sender.send(AppNotice::ServerRestartComplete)?;
    sender.send(AppNotice::Toast((
        format!("所有命令执行完成:{}", rs.exit_status),
        5,
    )))?;
    Ok(())
}

async fn create_ssh_client() -> Result<SshClient, Box<dyn Error>> {
    let client = SshClient::connect(SshConfig {
        host: "192.168.1.17".to_string(),
        port: 22,
        username: "root".to_string(),
        password: Some("debian123".to_string()),
        key_path: None,
    })
    .await?;
    Ok(client)
}

fn show_res(sender: &UnboundedSender<AppNotice>, rs: &CommandExecutedResult, duration: u64) {
    let stdout = rs.stdout.split("\n").collect::<Vec<&str>>();
    let stderr = rs.stderr.split("\n").collect::<Vec<&str>>();

    // 处理标准输出，每10行组合成一个消息
    let mut count = 0;
    for chunk in stdout.chunks(10) {
        count += 1;
        if !chunk.is_empty() {
            let message = chunk.join("\n");
            sender
                .send(AppNotice::Toast((message, duration * count)))
                .unwrap();
        }
    }
    count = 0;
    // 处理错误输出，每10行组合成一个消息
    for chunk in stderr.chunks(10) {
        count += 1;
        if !chunk.is_empty() {
            let message = chunk.join("\n");
            sender
                .send(AppNotice::Toast((message, duration * count)))
                .unwrap();
        }
    }
}

pub async fn server_log(
    env: &str,
    sender: &UnboundedSender<AppNotice>,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = create_ssh_client().await?;

    let rs = client
        .client
        .execute(&format!(
            "tail -100 /docker/xd-server-{}/xd/log/xd-gs.log | grep '[E]'",
            env
        ))
        .await?;
    show_res(sender, &rs, 5);
    Ok(())
}
