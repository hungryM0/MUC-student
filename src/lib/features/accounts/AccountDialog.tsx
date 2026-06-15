import { useEffect, useState } from 'react';
import { Alert, AlertDescription } from '$lib/components/ui/alert';
import { Button } from '$lib/components/ui/button';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
import { Input } from '$lib/components/ui/input';
import { Label } from '$lib/components/ui/label';
import type { AccountDto, AccountInput, AccountUpdateInput } from '$lib/types/app';

type Props = {
  open: boolean;
  mode: 'create' | 'edit';
  account?: AccountDto | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (input: AccountInput | AccountUpdateInput) => void;
};

export function AccountDialog({ open, mode, account = null, busy, onClose, onSubmit }: Props) {
  const [remarkName, setRemarkName] = useState('');
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [localError, setLocalError] = useState('');

  useEffect(() => {
    setRemarkName(account?.remarkName ?? '');
    setUsername(account?.username ?? '');
    setPassword('');
    setLocalError('');
  }, [account, open]);

  const submit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    setLocalError('');
    const remark = remarkName.trim();
    const user = username.trim();
    if (!remark || !user) {
      setLocalError('请填写备注名和账号');
      return;
    }
    if (mode === 'create' && !password) {
      setLocalError('请填写密码');
      return;
    }
    if (mode === 'edit' && account) {
      onSubmit({ accountId: account.id, remarkName: remark, username: user, password: password ? password : null });
      return;
    }
    onSubmit({ remarkName: remark, username: user, password });
  };

  return (
    <Dialog open={open} onOpenChange={(value) => !value && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>{mode === 'create' ? '添加账号' : '编辑账号'}</DialogTitle>
          <DialogDescription>{mode === 'create' ? '将新的校园网账号添加到号池。' : '修改已有账号。'}</DialogDescription>
        </DialogHeader>
        <form className="grid gap-4" onSubmit={submit}>
          <div className="grid gap-2">
            <Label htmlFor="remarkName">备注名</Label>
            <Input id="remarkName" value={remarkName} onChange={(event) => setRemarkName(event.target.value)} autoComplete="off" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="username">账号</Label>
            <Input id="username" value={username} onChange={(event) => setUsername(event.target.value)} autoComplete="username" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="password">密码</Label>
            <Input id="password" type="password" value={password} onChange={(event) => setPassword(event.target.value)} autoComplete={mode === 'create' ? 'new-password' : 'current-password'} placeholder={mode === 'edit' ? '不填则保留原密码' : ''} />
          </div>
          {localError && (
            <Alert variant="destructive">
              <AlertDescription>{localError}</AlertDescription>
            </Alert>
          )}
          <DialogFooter>
            <Button type="button" variant="ghost" onClick={onClose}>
              取消
            </Button>
            <Button type="submit" disabled={busy}>
              保存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
