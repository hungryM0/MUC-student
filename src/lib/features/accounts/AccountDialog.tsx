import { useEffect, useState } from 'react';
import {
  Button,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  Field,
  Input,
  MessageBar,
  MessageBarBody,
  makeStyles,
  tokens
} from '@fluentui/react-components';
import type { AccountDto, AccountInput, AccountUpdateInput } from '$lib/types/app';

type Props = {
  open: boolean;
  mode: 'create' | 'edit';
  account?: AccountDto | null;
  busy: boolean;
  onClose: () => void;
  onSubmit: (input: AccountInput | AccountUpdateInput) => void;
};

const useStyles = makeStyles({
  surface: {
    maxWidth: '460px',
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground1) 94%, transparent)',
    borderRadius: tokens.borderRadiusXLarge
  },
  form: {
    display: 'flex',
    flexDirection: 'column',
    gap: '16px'
  }
});

export function AccountDialog({ open, mode, account = null, busy, onClose, onSubmit }: Props) {
  const styles = useStyles();
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

  const submit = (event?: React.FormEvent) => {
    if (event) event.preventDefault();
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
    <Dialog open={open} onOpenChange={(_, data) => !data.open && onClose()}>
      <DialogSurface className={styles.surface}>
        <form onSubmit={submit}>
          <DialogBody>
            <DialogTitle>{mode === 'create' ? '添加账号' : '编辑账号'}</DialogTitle>
            <DialogContent className={styles.form}>
              <Field label="备注名" required>
                <Input value={remarkName} onChange={(event) => setRemarkName(event.target.value)} autoComplete="off" />
              </Field>
              <Field label="账号" required>
                <Input value={username} onChange={(event) => setUsername(event.target.value)} autoComplete="username" />
              </Field>
              <Field label={mode === 'create' ? '密码' : '密码 (不填则保留原密码)'} required={mode === 'create'}>
                <Input
                  type="password"
                  value={password}
                  onChange={(event) => setPassword(event.target.value)}
                  autoComplete={mode === 'create' ? 'new-password' : 'current-password'}
                />
              </Field>
              {localError && (
                <MessageBar intent="error">
                  <MessageBarBody>{localError}</MessageBarBody>
                </MessageBar>
              )}
            </DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={onClose}>
                取消
              </Button>
              <Button appearance="primary" type="submit" disabled={busy}>
                保存
              </Button>
            </DialogActions>
          </DialogBody>
        </form>
      </DialogSurface>
    </Dialog>
  );
}
