// src/bin/web_server.rs
use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use std::{net::SocketAddr, sync::Arc};
use tokio::fs;

#[tokio::main]
async fn main() -> Result<()> {
    let addr: SocketAddr = "0.0.0.0:4433".parse()?;
    let server_config = configure_server()?;
    
    let endpoint = Endpoint::server(server_config, addr)?;
    println!("HTTP/3 server listening on {}", addr);
    
    while let Some(conn) = endpoint.accept().await {
        tokio::spawn(async move {
            if let Err(e) = handle_http3_connection(conn).await {
                eprintln!("Connection failed: {:?}", e);
            }
        });
    }
    
    Ok(())
}

async fn handle_http3_connection(conn: quinn::Connecting) -> Result<()> {
    let connection = conn.await?;
    
    loop {
        match connection.accept_bi().await {
            Ok((mut send, mut recv)) => {
                let mut buf = [0u8; 1024];
                match recv.read(&mut buf).await {
                    Ok(Some(n)) => {
                        let request = String::from_utf8_lossy(&buf[..n]);
                        
                        let response = if request.starts_with("GET / ") {
                            let content = fs::read_to_string("static/index.html").await?;
                            format!(
                                "HTTP/3 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                                content.len(),
                                content
                            )
                        } else {
                            "HTTP/3 404 Not Found\r\n\r\n".to_string()
                        };
                        
                        send.write_all(response.as_bytes()).await?;
                    }
                    Ok(None) => break,
                    Err(_) => break,
                }
            }
            Err(quinn::ConnectionError::ApplicationClosed { .. }) => break,
            Err(e) => return Err(e.into()),
        }
    }
    
    Ok(())
}

fn configure_server() -> Result<ServerConfig> {
    let cert = std::fs::read("cert.der")?;
    let key = std::fs::read("key.der")?;
    
    let certificate = rustls::Certificate(cert);
    let private_key = rustls::PrivateKey(key);
    
    let mut server_config = ServerConfig::with_single_cert(vec![certificate], private_key)?;
    
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
    server_config.transport = Arc::new(transport_config);
    
    Ok(server_config)
}