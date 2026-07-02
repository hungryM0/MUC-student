import { useCallback, useState } from "react";
import {
  checkForUpdates,
  downloadAndInstall,
  UpdateProgress,
  UpdateCheckResult,
  AndroidUpdate,
} from "@/lib/updater";
import type { Update } from "@tauri-apps/plugin-updater";

export function useUpdater() {
  const [update, setUpdate] = useState<Update | AndroidUpdate | null>(null);
  const [checking, setChecking] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState<UpdateProgress | null>(null);

  const checkUpdate = useCallback(async (): Promise<UpdateCheckResult> => {
    setChecking(true);
    try {
      const result = await checkForUpdates();
      setUpdate(
        result.status === "available" || result.status === "android-available"
          ? result.update
          : null,
      );
      return result;
    } finally {
      setChecking(false);
    }
  }, []);

  const installUpdate = useCallback(
    async (target?: Update | AndroidUpdate) => {
      setDownloading(true);
      try {
        return await downloadAndInstall(
          target ?? update ?? undefined,
          (progressEvent) => {
            setProgress(progressEvent);
          },
        );
      } catch {
        return false;
      } finally {
        setDownloading(false);
      }
    },
    [update],
  );

  return {
    update,
    checking,
    downloading,
    progress,
    checkUpdate,
    installUpdate,
  };
}
