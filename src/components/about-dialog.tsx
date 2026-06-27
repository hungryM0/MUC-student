import { GitFork, Info } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { APP_DISPLAY_VERSION } from "@/lib/app-version";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";

interface AboutDialogProps {
  open: boolean;
  onClose: () => void;
}

export function AboutDialog({ open, onClose }: AboutDialogProps) {
  const handleOpenGithub = async () => {
    await openUrl("https://github.com/hungryM0/MUC-student");
  };

  return (
    <Dialog open={open} onOpenChange={(val) => !val && onClose()}>
      <DialogContent className="max-w-xs">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Info className="h-4.5 w-4.5 text-sky-500" />
            关于
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-5 py-2">
          <div className="space-y-2 text-center">
            <img
              src={appIconUrl}
              alt=""
              className="mx-auto h-12 w-12 rounded-xl"
            />
            <h3 className="text-lg font-bold tracking-tight">MUC-student</h3>
            <p className="text-muted-foreground text-[10px]">
              MUC 校园网多账号管理工具
            </p>
          </div>

          <div className="space-y-2 text-xs border border-border/60 bg-muted/20 rounded-lg p-3">
            <div className="flex justify-between items-center">
              <span className="text-muted-foreground">版本</span>
              <span className="font-semibold">{APP_DISPLAY_VERSION}</span>
            </div>
          </div>

          <Button
            onClick={handleOpenGithub}
            className="w-full h-8.5 text-xs gap-1.5"
            variant="outline"
          >
            <GitFork className="h-3.5 w-3.5" />
            GitHub 仓库
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
