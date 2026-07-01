import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { AccountDto } from "@/lib/muc";

interface DeleteConfirmDialogProps {
  account: AccountDto | null;
  deleting: boolean;
  onClose: () => void;
  onConfirm: () => void;
}

export function DeleteConfirmDialog({
  account,
  deleting,
  onClose,
  onConfirm,
}: DeleteConfirmDialogProps) {
  if (!account) {
    return null;
  }

  return (
    <Dialog open={!!account} onOpenChange={(open) => !open && onClose()}>
      <DialogContent className="max-w-xs">
        <DialogHeader>
          <DialogTitle>删除确认</DialogTitle>
        </DialogHeader>

        <div className="py-2 text-sm text-muted-foreground">
          确定要删除账号“
          <span className="font-semibold text-foreground">
            {account.remarkName}
          </span>
          ”吗？
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            size="sm"
            onClick={onClose}
            disabled={deleting}
          >
            取消
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={onConfirm}
            disabled={deleting}
          >
            {deleting ? "删除中" : "确认删除"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
