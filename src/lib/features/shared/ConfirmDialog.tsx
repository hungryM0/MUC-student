import {
  Button,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  makeStyles,
  Text,
  tokens
} from '@fluentui/react-components';

type Props = {
  open: boolean;
  title: string;
  message: string;
  confirmText: string;
  danger?: boolean;
  busy: boolean;
  onClose: () => void;
  onConfirm: () => void;
};

const useStyles = makeStyles({
  surface: {
    maxWidth: '430px',
    backgroundColor: 'color-mix(in srgb, var(--colorNeutralBackground1) 94%, transparent)',
    borderRadius: tokens.borderRadiusXLarge
  },
  dangerButton: {
    backgroundColor: tokens.colorPaletteRedBackground3,
    color: tokens.colorNeutralForegroundInverted,
    ':hover': {
      backgroundColor: tokens.colorPaletteRedForeground1,
      color: tokens.colorNeutralForegroundInverted
    }
  }
});

export function ConfirmDialog({ open, title, message, confirmText, danger = false, busy, onClose, onConfirm }: Props) {
  const styles = useStyles();

  return (
    <Dialog open={open} onOpenChange={(_, data) => !data.open && onClose()}>
      <DialogSurface className={styles.surface}>
        <DialogBody>
          <DialogTitle>{title}</DialogTitle>
          <DialogContent>
            <Text>{message}</Text>
          </DialogContent>
          <DialogActions>
            <Button appearance="secondary" onClick={onClose}>
              取消
            </Button>
            <Button appearance={danger ? undefined : 'primary'} className={danger ? styles.dangerButton : undefined} disabled={busy} onClick={onConfirm}>
              {confirmText}
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}
