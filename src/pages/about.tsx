import { useEffect, useState } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { getVersion } from "@tauri-apps/api/app";
import { GitFork } from "lucide-react";
import { Button } from "@/components/ui/button";
import { TitleBar } from "@/components/title-bar";
import { WindowFrame } from "@/components/window-frame";
import { openUrl } from "@tauri-apps/plugin-opener";
import { cancelDestroyWindow, destroyWindow } from "@/lib/window";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";

export default function AboutPage() {
  const [version, setVersion] = useState("");

  useEffect(() => {
    void getVersion().then(setVersion);
  }, []);

  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();

    const unlistenClose = appWindow.onCloseRequested(async (event) => {
      event.preventDefault();
      await destroyWindow(appWindow.label, 5000);
    });

    const unlistenFocusChanged = appWindow.onFocusChanged(
      ({ payload: focused }) => {
        if (focused) {
          cancelDestroyWindow(appWindow.label);
        }
      },
    );

    return () => {
      unlistenClose.then((fn) => fn());
      unlistenFocusChanged.then((fn) => fn());
    };
  }, []);

  const handleOpenGithub = async () => {
    await openUrl("https://github.com/hungryM0/MUC-student");
  };

  return (
    <WindowFrame
      titleBar={
        <TitleBar title="关于" showMinimize={false} showMaximize={false} />
      }
      contentClassName="flex flex-1 items-center justify-center overflow-hidden"
    >
      <div className="w-full max-w-xs space-y-6">
        <div className="space-y-3 text-center">
          <img
            src={appIconUrl}
            alt=""
            className="mx-auto h-14 w-14 rounded-xl"
          />
          <h2 className="text-2xl font-bold">MUC-student</h2>
        </div>

        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">版本</span>
            <span className="font-medium">{version}</span>
          </div>
        </div>

        <Button onClick={handleOpenGithub} className="w-full" variant="outline">
          <GitFork className="mr-2 h-4 w-4" />
          GitHub
        </Button>
      </div>
    </WindowFrame>
  );
}
