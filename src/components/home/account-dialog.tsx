import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { AccountFormChange, AccountFormState } from "./types";

interface AccountDialogProps {
  form: AccountFormState | null;
  saving: boolean;
  onChange: AccountFormChange;
  onClose: () => void;
  onSave: () => void;
}

export function AccountDialog({
  form,
  saving,
  onChange,
  onClose,
  onSave,
}: AccountDialogProps) {
  const [localForm, setLocalForm] = useState<AccountFormState | null>(null);

  useEffect(() => {
    if (form) {
      setLocalForm(form);
      return;
    }

    setLocalForm(null);
  }, [form]);

  if (!localForm) {
    return null;
  }

  const isEditing = !!localForm.accountId;
  const canSave =
    !!localForm.remarkName.trim() &&
    !!localForm.username.trim() &&
    (isEditing || !!localForm.password.trim());

  function updateForm(nextForm: AccountFormState) {
    setLocalForm(nextForm);
    onChange(nextForm);
  }

  return (
    <Dialog open={!!form} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{isEditing ? "编辑账号" : "添加账号"}</DialogTitle>
        </DialogHeader>

        <div className="grid gap-3 py-1">
          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">备注名</span>
            <Input
              value={localForm.remarkName}
              onChange={(event) =>
                updateForm({
                  ...localForm,
                  remarkName: event.target.value,
                })
              }
              autoFocus
            />
          </label>

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">账号</span>
            <Input
              value={localForm.username}
              onChange={(event) =>
                updateForm({
                  ...localForm,
                  username: event.target.value,
                })
              }
            />
          </label>

          <label className="grid gap-1.5 text-sm">
            <span className="font-medium">密码</span>
            <Input
              type="password"
              value={localForm.password}
              placeholder={isEditing ? "留空则不修改" : ""}
              onChange={(event) =>
                updateForm({
                  ...localForm,
                  password: event.target.value,
                })
              }
            />
          </label>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={saving}>
            取消
          </Button>
          <Button onClick={onSave} disabled={saving || !canSave}>
            {saving ? "保存中" : "保存"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
