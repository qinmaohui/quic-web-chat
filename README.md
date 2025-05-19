# QUIC Web Chat

一个基于 QUIC 协议和 WebSocket 的实时聊天应用。

## 功能特点

- 使用 QUIC 协议进行安全通信
- WebSocket 支持实时消息推送
- 用户在线状态显示
- 实时消息通知
- 响应式界面设计

## 技术栈

### 后端

- Rust
- Quinn (QUIC 实现)
- Warp (WebSocket 服务器)
- Tokio (异步运行时)

### 前端

- React
- TypeScript
- Tailwind CSS
- WebSocket API

## 快速开始

### 生成证书

首先需要生成自签名证书：

```bash
cargo run --bin generate_cert
```

### 运行服务器

```bash
cargo run
```

### 运行前端

```bash
cd frontend
npm install
npm start
```

## 项目结构

```
.
├── src/                # Rust 后端代码
│   ├── main.rs        # 主程序入口
│   ├── chat.rs        # 聊天功能实现
│   └── bin/           # 二进制程序
│       ├── chat_client.rs    # QUIC 客户端
│       └── generate_cert.rs  # 证书生成工具
├── frontend/          # React 前端代码
│   ├── src/
│   │   ├── components/      # React 组件
│   │   └── services/        # 服务层
│   └── package.json
└── Cargo.toml         # Rust 项目配置
```

## 许可证

MIT
