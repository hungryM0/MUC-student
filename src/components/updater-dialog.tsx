import { useEffect, useState } from "react";
import { useUpdater } from "@/hooks/use-updater";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Progress } from "@/components/ui/progress";
import { toast } from "sonner";

interface UpdaterDialogProps {
  manualCheck?: boolean;
  onCheckComplete?: () => void;
}

export function UpdaterDialog({
  manualCheck = false,
  onCheckComplete,
}: UpdaterDialogProps) {
  const {
    update,
    checking,
    downloading,
    progress,
    checkUpdate,
    installUpdate,
  } = useUpdater();
  const [open, setOpen] = useState(false);

  useEffect(() => {
    if (!manualCheck) {
      void checkUpdate();
    }
  }, [manualCheck, checkUpdate]);

  useEffect(() => {
    if (update) {
      setOpen(true);
      onCheckComplete?.();
    } else if (manualCheck && !checking) {
      onCheckComplete?.();
    }
  }, [update, checking, manualCheck, onCheckComplete]);

  const handleInstall = () => {
    void installUpdate();
  };

  const handleCancel = () => {
    setOpen(false);
  };

  const getProgressPercentage = () => {
    if (!progress || progress.event === "Started") return 0;
    const { downloaded, contentLength } = progress.data || {};
    if (!contentLength) return 0;
    if (progress.event === "Finished") return 100;
    return Math.round(((downloaded ?? 0) / contentLength) * 100);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>
            {downloading ? "正在下载更新" : "发现新版本"}
          </DialogTitle>
          <DialogDescription>
            {downloading ? (
              <div className="space-y-2">
                <p>正在安装版本 {update?.version}...</p>
                <Progress value={getProgressPercentage()} />
              </div>
            ) : (
              <div className="space-y-2">
                <p>版本 {update?.version} 可用。</p>
                {update?.body && (
                  <div className="bg-muted mt-2 rounded-md p-3 text-sm">
                    <p className="font-semibold">更新说明：</p>
                    <p className="mt-1 whitespace-pre-wrap">{update.body}</p>
                  </div>
                )}
              </div>
            )}
          </DialogDescription>
        </DialogHeader>
        {!downloading && (
          <DialogFooter>
            <Button variant="outline" onClick={handleCancel}>
              稍后
            </Button>
            <Button onClick={handleInstall}>立即安装</Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  );
}

export function useManualUpdateCheck() {
  const { checkUpdate, checking, update } = useUpdater();
  const [showNoUpdate, setShowNoUpdate] = useState(false);

  const handleCheckUpdate = async () => {
    setShowNoUpdate(false);
    const result = await checkUpdate();

    if (result.status === "up-to-date") {
      setShowNoUpdate(true);
      return;
    }

    if (result.status === "error") {
      toast.error("检查更新失败。");
    }
  };

  return {
    checkUpdate: handleCheckUpdate,
    checking,
    hasUpdate: !!update,
    showNoUpdate,
    dismissNoUpdate: () => setShowNoUpdate(false),
  };
}
