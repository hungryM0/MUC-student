import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Github } from "lucide-react";
import { Button } from "@/components/ui/button";
import { TitleBar } from "@/components/title-bar";
import { WindowFrame } from "@/components/window-frame";
import { openUrl } from "@tauri-apps/plugin-opener";
import { cancelDestroyWindow, destroyWindow } from "@/lib/window";
import packageJson from "../../package.json";

const techVersions = {
  tauri: packageJson.dependencies["@tauri-apps/api"].replace(/^\^/, "v"),
  react: packageJson.dependencies.react.replace(/^\^/, "v"),
  typescript: packageJson.devDependencies.typescript.replace(/^~/, "v"),
};

export default function AboutPage() {
  const [appVersion, setAppVersion] = useState("");

  useEffect(() => {
    void getVersion().then(setAppVersion);
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
        <div className="text-center">
          <h2 className="text-2xl font-bold">MUC-student</h2>
        </div>

        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">版本</span>
            <span className="font-medium">{appVersion}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">Tauri</span>
            <span className="font-medium">{techVersions.tauri}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">React</span>
            <span className="font-medium">{techVersions.react}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-muted-foreground">TypeScript</span>
            <span className="font-medium">{techVersions.typescript}</span>
          </div>
        </div>

        <Button onClick={handleOpenGithub} className="w-full" variant="outline">
          <Github className="mr-2 h-4 w-4" />
          GitHub
        </Button>
      </div>
    </WindowFrame>
  );
}
