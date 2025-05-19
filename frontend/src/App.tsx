import React, { useState } from "react";
import Chat from "./components/Chat";
import Login from "./components/Login";
import { Toaster } from "react-hot-toast";

const App: React.FC = () => {
  const [username, setUsername] = useState<string | null>(null);

  const handleLogin = (name: string) => {
    setUsername(name);
  };

  return (
    <div className="h-screen">
      <Toaster position="top-right" />
      {username ? (
        <Chat username={username} onLogout={() => setUsername(null)} />
      ) : (
        <Login onLogin={handleLogin} />
      )}
    </div>
  );
};

export default App;
