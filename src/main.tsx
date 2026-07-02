import React, { Suspense, useEffect } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isAndroid } from "@/lib/utils";
import "./index.css";

import HomePage from "./pages/home";

function AppWrapper() {
  useEffect(() => {
    // Show window after React is ready
    getCurrentWindow().show();
    if (isAndroid()) {
      document.documentElement.classList.add("platform-android");
    }
  }, []);

  return <HomePage />;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <Suspense fallback={null}>
      <AppWrapper />
    </Suspense>
  </React.StrictMode>,
);
