import { Copy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { AccountPoolDialogMode } from "./types";

interface AccountPoolDialogProps {
  mode: AccountPoolDialogMode | null;
  code: string;
  passphrase: string;
  busy: boolean;
  resultText: string;
  onCodeChange: (value: string) => void;
  onPassphraseChange: (value: string) => void;
  onClose: () => void;
  onExport: () => void;
  onImport: () => void;
  onCopy: () => void;
}

export function AccountPoolDialog({
  mode,
  code,
  passphrase,
  busy,
  resultText,
  onCodeChange,
  onPassphraseChange,
  onClose,
  onExport,
  onImport,
  onCopy,
}: AccountPoolDialogProps) {
  if (!mode) {
    return null;
  }

  const isExport = mode === "export";
  const canSubmit = !!passphrase.trim() && (isExport || !!code.trim()) && !busy;

  return (
    <Dialog open={!!mode} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-xl">
        <DialogHeader>
          <DialogTitle>{isExport ? "导出号池" : "导入号池"}</DialogTitle>
        </DialogHeader>

        <div className="grid gap-3 py-1">
          {!isExport && (
            <label className="grid gap-1.5 text-sm">
              <span className="font-medium">号池码</span>
              <textarea
                value={code}
                onChange={(event) => onCodeChange(event.target.value)}
                className="min-h-28 w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm shadow-xs outline-none transition-[color,box-shadow] placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30"
              />
            </label>
          )}

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">加密令牌</span>
            <Input
              type="password"
              value={passphrase}
              onChange={(event) => onPassphraseChange(event.target.value)}
              autoFocus
            />
          </label>

          {isExport && (
            <label className="grid gap-1.5 text-sm">
              <span className="font-medium">号池码</span>
              <div className="grid grid-cols-[1fr_36px] gap-2">
                <textarea
                  value={code}
                  readOnly
                  className="min-h-28 w-full resize-none rounded-md border border-input bg-muted/30 px-3 py-2 text-sm shadow-xs outline-none"
                />
                <Button
                  type="button"
                  variant="outline"
                  size="icon-sm"
                  disabled={!code.trim()}
                  onClick={onCopy}
                  aria-label="复制号池码"
                  className="h-9 w-9"
                >
                  <Copy className="h-4 w-4" />
                </Button>
              </div>
            </label>
          )}

          {resultText && (
            <div className="rounded-md border border-emerald-500/20 bg-emerald-500/10 px-3 py-2 text-sm text-emerald-700 dark:text-emerald-300">
              {resultText}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={busy}>
            关闭
          </Button>
          <Button
            onClick={isExport ? onExport : onImport}
            disabled={!canSubmit}
          >
            {busy ? "处理中" : isExport ? "生成" : "导入"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
