use anyhow::Result;
use quinn::{Endpoint, ServerConfig};
use std::{net::SocketAddr, sync::Arc};
use chat::ChatMessage;
use warp::Filter;
use futures::{StreamExt, SinkExt};
use tokio::sync::mpsc;

mod chat;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let addr: SocketAddr = "0.0.0.0:4433".parse()?;
    let server_config = configure_server()?;
    
    let endpoint = Endpoint::server(server_config, addr)?;
    
    tracing::info!("QUIC chat server listening on {}", addr);
    
    let chat_state = Arc::new(chat::ChatState::new());
    let chat_state_ws = chat_state.clone();
    
    // WebSocket 路由
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let chat_state = chat_state_ws.clone();
            ws.on_upgrade(move |socket| async move {
                handle_ws_connection(socket, chat_state).await;
            })
        });
    
    // 启动 WebSocket 服务器
    let ws_addr = "127.0.0.1:8080";
    tokio::spawn(async move {
        warp::serve(ws_route)
            .run(ws_addr.parse::<SocketAddr>().unwrap())
            .await;
    });
    
    // QUIC 服务器
    while let Some(conn) = endpoint.accept().await {
        let chat_state = chat_state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(conn, chat_state).await {
                tracing::error!("Connection failed: {:?}", e);
            }
        });
    }
    
    Ok(())
}

async fn handle_ws_connection(ws: warp::ws::WebSocket, chat_state: Arc<chat::ChatState>) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut rx = chat_state.subscribe();
    let (tx, mut rx_ws) = mpsc::unbounded_channel::<String>();
    let ws_id = chat_state.register_ws(tx);
    let mut username: Option<String> = None;

    // 等待用户名
    if let Some(Ok(msg)) = ws_receiver.next().await {
        if let Ok(login) = serde_json::from_str::<serde_json::Value>(msg.to_str().unwrap()) {
            if let Some(name) = login.get("username").and_then(|u| u.as_str()) {
                let name = name.to_string();
                chat_state.add_user(name.clone());
                chat_state.broadcast_user_list();
                username = Some(name.clone());

                let message = ChatMessage {
                    username: name.clone(),
                    content: format!("{} 加入了聊天室", name),
                    timestamp: chrono::Utc::now(),
                };
                chat_state.broadcast_message(message);

                // 只给当前用户推送一次完整用户列表
                let users = chat_state.get_users();
                let _ = ws_sender.send(warp::ws::Message::text(
                    serde_json::to_string(&serde_json::json!({
                        "type": "userList",
                        "users": users
                    })).unwrap()
                )).await;

                loop {
                    let mut should_break = false;
                    tokio::select! {
                        msg = ws_receiver.next() => {
                            match msg {
                                Some(Ok(msg)) => {
                                    if let Ok(message) = serde_json::from_str::<serde_json::Value>(msg.to_str().unwrap()) {
                                        if let Some(msg_type) = message.get("type").and_then(|t| t.as_str()) {
                                            if msg_type == "logout" {
                                                chat_state.remove_user(&name);
                                                chat_state.broadcast_user_list();
                                                should_break = true;
                                                continue;
                                            }
                                        }
                                        if let Some(content) = message.get("content").and_then(|c| c.as_str()) {
                                            let message = ChatMessage {
                                                username: name.clone(),
                                                content: content.to_string(),
                                                timestamp: chrono::Utc::now(),
                                            };
                                            chat_state.broadcast_message(message);
                                        }
                                    }
                                },
                                _ => { should_break = true; },
                            }
                        },
                        Ok(broadcast_msg) = rx.recv() => {
                            if ws_sender.send(warp::ws::Message::text(
                                serde_json::to_string(&broadcast_msg).unwrap()
                            )).await.is_err() {
                                should_break = true;
                            }
                        },
                        Some(user_list_msg) = rx_ws.recv() => {
                            if ws_sender.send(warp::ws::Message::text(user_list_msg)).await.is_err() {
                                should_break = true;
                            }
                        }
                    }
                    if should_break { break; }
                }
            }
        }
    }
    // 只要连接断开就移除用户
    if let Some(name) = username {
        chat_state.remove_user(&name);
        chat_state.broadcast_user_list();
    }
    chat_state.unregister_ws(ws_id);
}

async fn handle_connection(
    conn: quinn::Connecting,
    chat_state: Arc<chat::ChatState>,
) -> Result<()> {
    let connection = conn.await?;
    tracing::info!("New connection: {}", connection.remote_address());
    
    while let Ok((mut send, mut recv)) = connection.accept_bi().await {
        let chat_state = chat_state.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_stream(&mut send, &mut recv, chat_state).await {
                tracing::error!("Stream handling failed: {:?}", e);
            }
        });
    }
    
    Ok(())
}

async fn handle_stream(
    _send: &mut quinn::SendStream,
    recv: &mut quinn::RecvStream,
    chat_state: Arc<chat::ChatState>,
) -> Result<()> {
    let mut buf = vec![0u8; 1024];
    let n = recv.read(&mut buf).await?;
    
    if let Some(n) = n {
        let username = String::from_utf8_lossy(&buf[..n]).to_string();
        chat_state.add_user(username.clone());
        
        let message = ChatMessage {
            username: username.clone(),
            content: format!("{} 加入了聊天室", username),
            timestamp: chrono::Utc::now(),
        };
        
        chat_state.broadcast_message(message);
        
        while let Ok(n) = recv.read(&mut buf).await {
            if let Some(n) = n {
                let content = String::from_utf8_lossy(&buf[..n]).to_string();
                let message = ChatMessage {
                    username: username.clone(),
                    content,
                    timestamp: chrono::Utc::now(),
                };
                
                chat_state.broadcast_message(message);
            }
        }
        
        chat_state.remove_user(&username);
        let message = ChatMessage {
            username: username.clone(),
            content: format!("{} 离开了聊天室", username),
            timestamp: chrono::Utc::now(),
        };
        
        chat_state.broadcast_message(message);
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
