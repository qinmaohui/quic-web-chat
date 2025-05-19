// src/bin/chat_client.rs
use anyhow::{Context, Result};
use quinn::{ClientConfig, Endpoint};
use std::{net::SocketAddr, sync::Arc};
use tokio::io::BufReader;
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> Result<()> {
    let server_addr: SocketAddr = "127.0.0.1:4433".parse()?;
    let endpoint = create_client_endpoint("0.0.0.0:0")?;
    
    println!("Connecting to server at {}", server_addr);
    let connection = endpoint
        .connect(server_addr, "localhost")?
        .await
        .context("Failed to connect")?;
    
    println!("Connected. Please enter your username:");
    let mut username = String::new();
    let mut stdin = BufReader::new(tokio::io::stdin());
    stdin.read_line(&mut username).await?;
    let username = username.trim();
    
    // 打开双向流
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // 发送用户名
    send.write_all(username.as_bytes()).await?;
    
    // 启动接收消息任务
    let _recv_task = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match recv.read(&mut buf).await {
                Ok(Some(n)) if n > 0 => {
                    let msg = String::from_utf8_lossy(&buf[..n]);
                    println!("{}", msg);
                }
                Ok(Some(0)) | Ok(None) => break,
                Err(_) => break,
                _ => continue,
            }
        }
        Ok::<_, anyhow::Error>(())
    });
    
    // 处理用户输入
    let mut input = String::new();
    loop {
        input.clear();
        stdin.read_line(&mut input).await?;
        send.write_all(input.trim().as_bytes()).await?;
    }
}

fn create_client_endpoint(bind_addr: &str) -> Result<Endpoint> {
    let client_cfg = configure_client()?;
    let mut endpoint = Endpoint::client(bind_addr.parse()?)?;
    endpoint.set_default_client_config(client_cfg);
    Ok(endpoint)
}

fn configure_client() -> Result<ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(Arc::new(danger::NoCertificateVerification))
        .with_no_client_auth();
    
    let mut cfg = ClientConfig::new(Arc::new(crypto));
    
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
    cfg.transport_config(Arc::new(transport));
    
    Ok(cfg)
}

mod danger {
    use rustls::client::{ServerCertVerifier, ServerCertVerified};
    use rustls::{Certificate, Error};
    
    pub struct NoCertificateVerification;
    
    impl ServerCertVerifier for NoCertificateVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &Certificate,
            _intermediates: &[Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<ServerCertVerified, Error> {
            Ok(ServerCertVerified::assertion())
        }
    }
}