import React, { Suspense, useEffect } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isAndroid } from "@/lib/utils";
import "./index.css";

import HomePage from "./pages/home";

function AppWrapper() {
  useEffect(() => {
    void invoke<boolean>("should_show_main_window_on_launch").then(
      (shouldShow) => {
        if (shouldShow) {
          return getCurrentWindow().show();
        }
      },
      () => getCurrentWindow().show(),
    );
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
