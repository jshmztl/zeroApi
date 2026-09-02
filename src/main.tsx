import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
// 自托管可变字体：离线优先，契合 Local-first 定位（无需运行时联网下载）
import "@fontsource-variable/saira";
import "@fontsource-variable/chivo";
import "@fontsource-variable/jetbrains-mono";
import "./styles/globals.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
