import React, { useState, useEffect, useRef } from "react";
import { Message, User, webSocketService } from "../services/WebSocketService";
import { toast } from "react-hot-toast";
import { UserGroupIcon } from "@heroicons/react/24/outline";

interface ChatProps {
  username: string;
  onLogout: () => void;
}

const Chat: React.FC<ChatProps> = ({ username, onLogout }) => {
  const [messages, setMessages] = useState<Message[]>([]);
  const [inputMessage, setInputMessage] = useState("");
  const [users, setUsers] = useState<User[]>([]);
  const messagesEndRef = useRef<null | HTMLDivElement>(null);

  useEffect(() => {
    webSocketService.connect(username);

    const messageUnsubscribe = webSocketService.onMessage((message) => {
      setMessages((prev) => [...prev, message]);
      if (message.username !== username) {
        toast(`${message.username}: ${message.content}`, {
          duration: 3000,
        });
      }
    });

    const userListUnsubscribe = webSocketService.onUserList((userList) => {
      setUsers(userList);
    });

    return () => {
      messageUnsubscribe();
      userListUnsubscribe();
      webSocketService.disconnect();
    };
  }, [username]);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (inputMessage.trim()) {
      webSocketService.sendMessage(inputMessage.trim());
      setInputMessage("");
    }
  };

  return (
    <div className="flex h-screen bg-gray-100">
      {/* 用户列表侧边栏 */}
      <div className="w-64 bg-white border-r p-4">
        <div className="flex items-center mb-4">
          <UserGroupIcon className="h-6 w-6 text-gray-500 mr-2" />
          <h2 className="text-lg font-semibold">在线用户</h2>
        </div>
        <div className="space-y-2">
          {users.map((user) => (
            <div
              key={user.username}
              className="flex items-center space-x-2 p-2 rounded hover:bg-gray-100"
            >
              <div className={`w-2 h-2 rounded-full bg-green-500`} />
              <span className="text-sm">{user.username}</span>
            </div>
          ))}
        </div>
      </div>

      {/* 聊天主区域 */}
      <div className="flex-1 flex flex-col">
        {/* 顶部栏，添加登出按钮 */}
        <div className="flex justify-end items-center p-2 bg-white border-b">
          <button
            onClick={() => {
              webSocketService.sendLogout();
              setTimeout(() => {
                webSocketService.disconnect();
                onLogout();
              }, 100);
            }}
            className="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600"
          >
            退出
          </button>
        </div>
        <div className="flex-1 p-4 overflow-y-auto">
          <div className="space-y-4">
            {messages.map((message, index) => (
              <div
                key={index}
                className={`flex ${
                  message.username === username
                    ? "justify-end"
                    : "justify-start"
                }`}
              >
                <div
                  className={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                    message.username === username
                      ? "bg-blue-500 text-white"
                      : "bg-white text-gray-800"
                  }`}
                >
                  <div className="font-bold">{message.username}</div>
                  <div>{message.content}</div>
                  <div className="text-xs opacity-75">
                    {new Date(message.timestamp).toLocaleTimeString()}
                  </div>
                </div>
              </div>
            ))}
            <div ref={messagesEndRef} />
          </div>
        </div>

        <form onSubmit={handleSubmit} className="p-4 bg-white border-t">
          <div className="flex space-x-4">
            <input
              type="text"
              value={inputMessage}
              onChange={(e) => setInputMessage(e.target.value)}
              placeholder="输入消息..."
              className="flex-1 p-2 border rounded-lg focus:outline-none focus:border-blue-500"
            />
            <button
              type="submit"
              className="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 focus:outline-none"
            >
              发送
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};

export default Chat;
