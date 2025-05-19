export interface Message {
  username: string;
  content: string;
  timestamp: Date;
}

export interface User {
  username: string;
  last_seen: string;
}

class WebSocketService {
  private socket: WebSocket | null = null;
  private messageHandlers: ((message: Message) => void)[] = [];
  private userListHandlers: ((users: User[]) => void)[] = [];
  private username: string = "";

  public connect(username: string) {
    this.username = username;
    this.socket = new WebSocket("ws://localhost:8080/ws");

    this.socket.onopen = () => {
      // 登录
      this.socket?.send(JSON.stringify({ username }));
    };

    this.socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        // 处理 userList
        if (data.type === "userList") {
          this.userListHandlers.forEach((handler) => handler(data.users));
          return;
        }
        // 过滤 system userList 消息
        if (data.username === "system" && typeof data.content === "string") {
          try {
            const contentObj = JSON.parse(data.content);
            if (contentObj.type === "userList") {
              // 是 userList，忽略
              return;
            }
          } catch {
            // 不是 JSON，继续往下
          }
        }
        // 普通聊天消息
        if (data.username && data.content && data.timestamp) {
          this.messageHandlers.forEach((handler) => handler(data));
        }
      } catch {
        // 不是 JSON，忽略
      }
    };

    this.socket.onclose = () => {
      // 断开处理
    };

    this.socket.onerror = (err) => {
      // 错误处理
    };
  }

  public sendMessage(message: string) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ content: message }));
    }
  }

  public sendLogout() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ type: "logout" }));
    }
  }

  public onMessage(handler: (message: Message) => void) {
    this.messageHandlers.push(handler);
    return () => {
      this.messageHandlers = this.messageHandlers.filter((h) => h !== handler);
    };
  }

  public onUserList(handler: (users: User[]) => void) {
    this.userListHandlers.push(handler);
    return () => {
      this.userListHandlers = this.userListHandlers.filter(
        (h) => h !== handler
      );
    };
  }

  public disconnect() {
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
  }
}

export const webSocketService = new WebSocketService();
