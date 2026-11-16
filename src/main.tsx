import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import AddWindowApp from "./addWindow/AddWindowApp";

/** The Tauri window this page runs in ("main", or "add" for the add-downloads window). */
function windowLabel(): string {
  const internals = (window as { __TAURI_INTERNALS__?: { metadata?: { currentWindow?: { label?: string } } } }).__TAURI_INTERNALS__;
  return internals?.metadata?.currentWindow?.label ?? "main";
}

// React 19's development build records every component render with performance.measure() for
// DevTools. The timeline buffer is unbounded: with progress re-rendering the table four times a
// second it held over a million entries and crashed the WebView after about two hours. The app
// reads no measures; production React records none.
if (import.meta.env.DEV) {
  setInterval(() => performance.clearMeasures(), 10_000);
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    {windowLabel() === "add" ? <AddWindowApp /> : <App />}
  </React.StrictMode>,
);
