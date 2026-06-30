import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { logMessage } from "./api";

window.onerror = (message, source, lineno, colno, error) => {
  logMessage(`[JS ERROR] ${message} at ${source}:${lineno}:${colno} ${error?.stack || ""}`).catch(
    console.error
  );
};

window.onunhandledrejection = (event) => {
  logMessage(`[UNHANDLED REJECTION] ${event.reason}`).catch(console.error);
};

logMessage("Frontend script loaded");

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
