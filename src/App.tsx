import { Activity, AlertCircle, Home, Moon, Settings, Sun, Wifi } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
import { AccountDialog } from '$lib/features/accounts/AccountDialog';
import { HomePanel } from '$lib/features/home/HomePanel';
import { ConfirmDialog } from '$lib/features/shared/ConfirmDialog';
import { SettingsPanel } from '$lib/features/settings/SettingsPanel';
import { StatusPanel } from '$lib/features/status/StatusPanel';
import { useAppStore } from '$lib/hooks/use-app-store';
import {
  bootstrapApp,
  clearError,
  closeDialog,
  createAccount,
  deleteAccount,
  disposeTauriBridge,
  findAccount,
  loginSelectedAccount,
  logoutLocalDevice,
  openCreateAccountDialog,
  openDeleteConfirm,
  openEditAccountDialog,
  openLogoutConfirm,
  refreshDashboard,
  selectAccount,
  setActivePage,
  setSortMode,
  updateAccount,
  updatePreferences
} from '$lib/stores/app';
import type { AccountInput, AccountUpdateInput, PreferenceInput } from '$lib/types/app';
import { cn } from '$lib/utils';

export default function App() {
  const { appSnapshot, uiState, dialogState } = useAppStore();
  const [theme, setTheme] = useState<'dark' | 'light'>('dark');

  useEffect(() => {
    bootstrapApp().catch(() => undefined);
    if (window.matchMedia?.('(prefers-color-scheme: light)').matches) {
      setTheme('light');
    }
    return () => disposeTauriBridge();
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
  }, [theme]);

  const busy = Boolean(uiState.loadingMessage || appSnapshot?.loginState.running || appSnapshot?.refreshState.running);
  const pageTitle = useMemo(() => {
    if (uiState.activePage === 'status') return '状态';
    if (uiState.activePage === 'settings') return '设置';
    return '主页';
  }, [uiState.activePage]);

  const submitAccount = async (input: AccountInput | AccountUpdateInput) => {
    try {
      if ('accountId' in input) {
        await updateAccount(input);
      } else {
        await createAccount(input);
      }
      closeDialog();
    } catch {
      return;
    }
  };

  const confirmDelete = async (accountId: string) => {
    try {
      await deleteAccount(accountId);
      closeDialog();
    } catch {
      return;
    }
  };

  const confirmLogout = async () => {
    try {
      await logoutLocalDevice();
      closeDialog();
    } catch {
      return;
    }
  };

  const savePreferences = async (preferences: PreferenceInput) => {
    try {
      await updatePreferences(preferences);
    } catch {
      return;
    }
  };

  if (!appSnapshot) {
    return (
      <main className="grid h-full place-items-center bg-background p-6">
        <div className="w-full max-w-sm rounded-lg border border-border bg-card p-6 text-center shadow-lg">
          <h1 className="text-2xl font-semibold text-primary">MUC-student</h1>
          <p className="mt-3 text-sm text-muted-foreground">{uiState.loadingMessage || '启动中...'}</p>
          {uiState.error && (
            <Alert variant="destructive" className="mt-4 text-left">
              <AlertTitle>{uiState.error.message}</AlertTitle>
              <AlertDescription>{uiState.error.detail}</AlertDescription>
            </Alert>
          )}
        </div>
      </main>
    );
  }

  return (
    <main className="flex h-full bg-background text-foreground">
      <aside className="flex w-64 shrink-0 flex-col border-r border-border bg-card px-4 py-5">
        <div className="px-2">
          <h1 className="text-2xl font-semibold tracking-normal text-primary">MUC</h1>
          <p className="mt-1 text-sm text-muted-foreground">校园网认证客户端</p>
        </div>

        <Tabs value={uiState.activePage} onValueChange={(value) => setActivePage(value as typeof uiState.activePage)} orientation="vertical" className="mt-8 flex-1">
          <TabsList className="flex h-auto w-full flex-col items-stretch justify-start gap-1 bg-transparent p-0">
            <NavItem value="home" icon={Home} label="主页" active={uiState.activePage === 'home'} />
            <NavItem value="status" icon={Activity} label="状态" active={uiState.activePage === 'status'} />
            <NavItem value="settings" icon={Settings} label="设置" active={uiState.activePage === 'settings'} />
          </TabsList>
        </Tabs>

        <Button variant="ghost" className="justify-start" onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>
          {theme === 'dark' ? <Sun /> : <Moon />}
          {theme === 'dark' ? '浅色' : '深色'}
        </Button>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-16 shrink-0 items-center justify-between gap-3 border-b border-border bg-background/95 px-6">
          <h2 className="text-lg font-semibold">{pageTitle}</h2>
          <div className="flex min-w-0 items-center gap-2">
            <Badge variant={appSnapshot.network.isOnline ? 'secondary' : 'destructive'} className="gap-1">
              <Wifi className="size-3" />
              {appSnapshot.network.statusText}
            </Badge>
            <Badge variant="outline" className="max-w-44 truncate font-mono">
              IP: {appSnapshot.network.ip || 'unknown'}
            </Badge>
            {uiState.loadingMessage && <Badge className="max-w-48 truncate">{uiState.loadingMessage}</Badge>}
          </div>
        </header>

        <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden p-6">
          {uiState.error && (
            <Alert variant="destructive">
              <AlertCircle className="size-4" />
              <AlertTitle>{uiState.error.message}</AlertTitle>
              <AlertDescription className="flex items-center justify-between gap-3">
                <span>{uiState.error.detail}</span>
                <Button size="sm" variant="ghost" onClick={clearError}>
                  关闭
                </Button>
              </AlertDescription>
            </Alert>
          )}

          {uiState.activePage === 'home' && (
            <HomePanel
              snapshot={appSnapshot}
              busy={busy}
              onSelectAccount={(accountId) => selectAccount(accountId).catch(() => undefined)}
              onLogin={() => loginSelectedAccount().catch(() => undefined)}
            />
          )}
          {uiState.activePage === 'status' && (
            <StatusPanel
              snapshot={appSnapshot}
              sortMode={uiState.sortMode}
              busy={busy}
              onSortMode={setSortMode}
              onAdd={openCreateAccountDialog}
              onEdit={openEditAccountDialog}
              onDelete={openDeleteConfirm}
              onLogout={openLogoutConfirm}
              onRefresh={() => refreshDashboard().catch(() => undefined)}
            />
          )}
          {uiState.activePage === 'settings' && <SettingsPanel snapshot={appSnapshot} busy={busy} onUpdatePreferences={savePreferences} />}
        </div>
      </div>

      {dialogState.type === 'account' && (
        <AccountDialog
          open
          mode={dialogState.mode}
          account={dialogState.mode === 'edit' ? findAccount(appSnapshot, dialogState.accountId) : null}
          busy={busy}
          onClose={closeDialog}
          onSubmit={submitAccount}
        />
      )}

      {dialogState.type === 'confirmDelete' && (
        <ConfirmDialog
          open
          title="删除账号"
          message="确定删除这个账号吗？删掉后需要重新添加。"
          confirmText="删除"
          danger
          busy={busy}
          onClose={closeDialog}
          onConfirm={() => confirmDelete(dialogState.accountId)}
        />
      )}

      {dialogState.type === 'confirmLogout' && (
        <ConfirmDialog
          open
          title="下线本机设备"
          message="下线后本机会断网，需要重新登录认证。"
          confirmText="确认下线"
          busy={busy}
          onClose={closeDialog}
          onConfirm={confirmLogout}
        />
      )}
    </main>
  );
}

type NavItemProps = {
  value: string;
  icon: React.ElementType;
  label: string;
  active: boolean;
};

function NavItem({ value, icon: Icon, label, active }: NavItemProps) {
  return (
    <TabsTrigger
      value={value}
      className={cn(
        'h-10 justify-start gap-2 rounded-md px-3 data-[state=active]:bg-primary data-[state=active]:text-primary-foreground',
        active ? 'text-primary-foreground' : 'text-muted-foreground'
      )}
    >
      <Icon className="size-4" />
      {label}
    </TabsTrigger>
  );
}
