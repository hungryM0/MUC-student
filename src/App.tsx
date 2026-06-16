import { AlertCircle, Home, Moon, Settings, Sun, Wifi, Menu } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
import { Badge } from '$lib/components/ui/badge';
import { Button } from '$lib/components/ui/button';
import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
import { AccountDialog } from '$lib/features/accounts/AccountDialog';
import { HomePanel } from '$lib/features/home/HomePanel';
import { ConfirmDialog } from '$lib/features/shared/ConfirmDialog';
import { SettingsPanel } from '$lib/features/settings/SettingsPanel';
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
  const [collapsed, setCollapsed] = useState(() => localStorage.getItem('sidebar_collapsed') === 'true');

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

  const toggleCollapsed = () => {
    setCollapsed((prev) => {
      const next = !prev;
      localStorage.setItem('sidebar_collapsed', String(next));
      return next;
    });
  };

  const busy = Boolean(uiState.loadingMessage || appSnapshot?.loginState.running || appSnapshot?.refreshState.running);
  const pageTitle = useMemo(() => {
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
    <main className="flex h-full bg-background text-foreground select-none">
      <aside className={cn(
        "flex shrink-0 flex-col border-r border-border/40 bg-card/60 backdrop-blur-md transition-all duration-300 ease-in-out py-4",
        collapsed ? "w-14 px-2" : "w-56 px-3"
      )}>
        <div className={cn("flex items-center gap-2 px-2 pb-4", collapsed ? "justify-center" : "")}>
          <Button
            variant="ghost"
            size="icon"
            className="h-8 w-8 shrink-0 rounded-md hover:bg-muted/80 cursor-pointer"
            onClick={toggleCollapsed}
          >
            <Menu className="size-4" />
          </Button>
          {!collapsed && (
            <div className="flex flex-col min-w-0 transition-opacity duration-300">
              <h1 className="text-sm font-semibold tracking-tight text-primary leading-tight">MUC-student</h1>
              <p className="text-[10px] text-muted-foreground leading-tight">多账号网络认证</p>
            </div>
          )}
        </div>

        <Tabs value={uiState.activePage} onValueChange={(value) => setActivePage(value as typeof uiState.activePage)} orientation="vertical" className="flex-1 mt-2">
          <TabsList className="flex h-auto w-full flex-col items-stretch justify-start gap-1 bg-transparent p-0">
            <NavItem value="home" icon={Home} label="主页" active={uiState.activePage === 'home'} collapsed={collapsed} />
            <NavItem value="settings" icon={Settings} label="设置" active={uiState.activePage === 'settings'} collapsed={collapsed} />
          </TabsList>
        </Tabs>

        <Button
          variant="ghost"
          className={cn(
            'mt-auto h-9 transition-all duration-200 cursor-pointer',
            collapsed ? 'w-9 px-0 justify-center mx-auto' : 'w-full px-3 justify-start gap-2.5'
          )}
          onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}
        >
          {theme === 'dark' ? <Sun className="size-4 shrink-0" /> : <Moon className="size-4 shrink-0" />}
          {!collapsed && <span className="text-sm font-medium">{theme === 'dark' ? '浅色模式' : '深色模式'}</span>}
        </Button>
      </aside>

      <div className="flex min-w-0 flex-1 flex-col">
        <header className="flex h-14 shrink-0 items-center justify-between gap-3 bg-background/60 backdrop-blur-md px-8 pt-4 pb-2">
          <div>
            <h2 className="text-xl font-semibold tracking-tight">{pageTitle}</h2>
          </div>
          <div className="flex min-w-0 items-center gap-2">
            <Badge variant={appSnapshot.network.isOnline ? 'secondary' : 'destructive'} className="gap-1 rounded-full px-2.5 py-0.5 text-xs font-normal">
              <Wifi className="size-3" />
              {appSnapshot.network.statusText}
            </Badge>
            <Badge variant="outline" className="max-w-44 truncate font-mono rounded-full px-2.5 py-0.5 text-xs font-normal">
              IP: {appSnapshot.network.ip || 'unknown'}
            </Badge>
            {uiState.loadingMessage && (
              <Badge className="max-w-48 truncate rounded-full px-2.5 py-0.5 text-xs font-normal">
                {uiState.loadingMessage}
              </Badge>
            )}
          </div>
        </header>

        <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden px-8 pb-8 pt-2">
          {uiState.error && (
            <Alert variant="destructive" className="rounded-lg border-destructive/20 bg-destructive/5 text-destructive">
              <AlertCircle className="size-4 text-destructive" />
              <AlertTitle className="font-semibold">{uiState.error.message}</AlertTitle>
              <AlertDescription className="flex items-center justify-between gap-3 text-xs opacity-90">
                <span>{uiState.error.detail}</span>
                <Button size="sm" variant="ghost" className="h-7 hover:bg-destructive/10 text-destructive" onClick={clearError}>
                  关闭
                </Button>
              </AlertDescription>
            </Alert>
          )}

          {uiState.activePage === 'home' && (
            <HomePanel
              snapshot={appSnapshot}
              sortMode={uiState.sortMode}
              busy={busy}
              onSelectAccount={(accountId) => selectAccount(accountId).catch(() => undefined)}
              onLogin={() => loginSelectedAccount().catch(() => undefined)}
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
  collapsed: boolean;
};

function NavItem({ value, icon: Icon, label, active, collapsed }: NavItemProps) {
  return (
    <TabsTrigger
      value={value}
      className={cn(
        'relative flex h-9 items-center rounded-md px-3 text-sm font-medium transition-all duration-200 cursor-pointer select-none',
        'hover:bg-muted/60 data-[state=active]:bg-muted/80 data-[state=active]:shadow-none data-[state=active]:text-foreground',
        active ? 'text-foreground font-medium' : 'text-muted-foreground',
        collapsed ? 'justify-center px-0 w-9 mx-auto' : 'justify-start gap-2.5 w-full'
      )}
    >
      {active && (
        <span className={cn(
          'absolute left-0 top-1/2 -translate-y-1/2 w-[3px] h-4 rounded-full bg-primary transition-all duration-200',
          collapsed && 'left-1'
        )} />
      )}
      <Icon className="size-4 shrink-0" />
      <span className={cn('transition-all duration-200 truncate', collapsed ? 'w-0 opacity-0 pointer-events-none' : 'w-auto opacity-100')}>
        {label}
      </span>
    </TabsTrigger>
  );
}
