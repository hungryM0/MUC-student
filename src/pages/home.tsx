import { useState } from "react";
import { CircleGauge, Settings as SettingsIcon, Users } from "lucide-react";
import { WindowFrame } from "@/components/window-frame";
import { MainTitleBar } from "@/components/main-title-bar";
import { SettingsDialog } from "@/components/settings-dialog";
import { AboutDialog } from "@/components/about-dialog";
import { MobileSettings } from "@/components/mobile-settings";
import { UpdaterDialog } from "@/components/updater-dialog";
import { AccountDialog } from "@/components/home/account-dialog";
import { AccountPoolDialog } from "@/components/home/account-pool-dialog";
import { AccountPoolSection } from "@/components/home/account-pool-section";
import { ActionLoadingOverlay } from "@/components/home/action-loading-overlay";
import { DeleteConfirmDialog } from "@/components/home/delete-confirm-dialog";
import { ErrorBanner } from "@/components/home/error-banner";
import { OverviewSection } from "@/components/home/overview-section";
import type { HomeTab } from "@/components/home/types";
import { useHomePageController } from "@/hooks/use-home-page-controller";
import { cn, isAndroid } from "@/lib/utils";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";

export default function HomePage() {
  const [activeTab, setActiveTab] = useState<HomeTab>("accounts");
  const controller = useHomePageController();

  return (
    <WindowFrame
      titleBar={
        !isAndroid() ? (
          <MainTitleBar
            onOpenSettings={controller.openSettings}
            onOpenAbout={controller.openAbout}
          />
        ) : null
      }
      contentClassName="flex flex-1 overflow-hidden"
    >
      <div className="relative flex min-h-0 flex-1 flex-col overflow-hidden">
        <ErrorBanner
          message={controller.errorText || controller.preferencesErrorText}
          onClose={controller.clearError}
        />

        <div
          className={cn(
            "shrink-0 px-4 pt-4 md:px-6 md:pt-6",
            isAndroid() && activeTab === "settings" && "hidden",
          )}
        >
          <div className="mx-auto w-full max-w-5xl xl:max-w-7xl 2xl:max-w-[1440px]">
            <header className="flex flex-col justify-between gap-2 border-b border-border/40 pb-3 sm:flex-row sm:items-center sm:gap-0 md:pb-4">
              <div className="flex items-center gap-3 md:gap-4">
                <h1 className="flex items-center gap-2.5 text-xl font-bold tracking-tight text-foreground">
                  <img
                    src={appIconUrl}
                    alt=""
                    className="h-7 w-7 shrink-0 rounded-md"
                  />
                  MUC 校园网拼车
                </h1>
                <div className="bg-border/60 hidden h-4 w-px sm:block" />
              </div>
              <span className="text-xs text-muted-foreground md:hidden">
                IP:{" "}
                {controller.snapshot?.network.ip &&
                controller.snapshot.network.ip !== "unknown"
                  ? controller.snapshot.network.ip
                  : "未知"}
              </span>
              <span className="hidden font-mono text-xs text-muted-foreground sm:inline">
                {controller.snapshot?.network.ip &&
                controller.snapshot.network.ip !== "unknown"
                  ? controller.snapshot.network.ip
                  : "IP 未识别"}
              </span>
            </header>
          </div>
        </div>

        <main
          className={cn(
            "min-w-0 flex-1 overflow-y-auto p-4 md:p-6",
            isAndroid() && "pb-24",
          )}
        >
          <div className="mx-auto flex w-full max-w-5xl flex-col gap-4 md:gap-6 xl:max-w-7xl 2xl:max-w-[1440px]">
            <div className="flex flex-col gap-4 md:flex-row md:gap-6">
              <div
                className={cn(
                  "min-w-0 flex-1",
                  isAndroid() && activeTab !== "accounts" && "hidden md:block",
                )}
              >
                <AccountPoolSection
                  snapshot={controller.snapshot}
                  loading={controller.loading}
                  isBusy={controller.isBusy}
                  selectingId={controller.selectingId}
                  loginAccountId={controller.loginAccountId}
                  deletingAccountId={controller.deletingAccountId}
                  runningAction={controller.runningAction}
                  onOpenImport={() => controller.openAccountPoolDialog("import")}
                  onOpenExport={() => controller.openAccountPoolDialog("export")}
                  onOpenAddAccount={controller.openAddAccountForm}
                  onRefresh={controller.refreshDashboard}
                  onEditAccount={controller.openEditAccountForm}
                  onDeleteAccount={controller.requestDeleteAccount}
                  onLoginAccount={controller.handleLoginAccount}
                />
              </div>

              <div
                className={cn(
                  "flex w-full shrink-0 flex-col gap-6 md:w-80",
                  isAndroid() && activeTab !== "overview" && "hidden md:flex",
                )}
              >
                <OverviewSection
                  snapshot={controller.snapshot}
                  canLogoutLocalDevice={controller.canLogoutLocalDevice}
                  isBusy={controller.isBusy}
                  onLogoutLocalDevice={controller.logoutLocalDevice}
                />
              </div>

              {isAndroid() && activeTab === "settings" && (
                <div className="w-full flex-1 md:hidden">
                  <MobileSettings
                    preferences={controller.snapshot?.preferences ?? null}
                    errorText={controller.preferencesErrorText}
                    saving={controller.preferencesSaving}
                    onTogglePreference={controller.togglePreference}
                  />
                </div>
              )}
            </div>
          </div>
        </main>

        {isAndroid() && (
          <div className="absolute right-0 bottom-0 left-0 z-40 border-t border-border/40 bg-background/95 pb-[env(safe-area-inset-bottom)] shadow-[0_-4px_16px_rgba(0,0,0,0.05)] backdrop-blur-md">
            <div className="flex h-16 items-center justify-around px-2">
              <button
                onClick={() => setActiveTab("accounts")}
                className={cn(
                  "flex h-full w-full flex-col items-center justify-center gap-1 transition-colors",
                  activeTab === "accounts"
                    ? "text-emerald-500"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <Users className="h-6 w-6" />
                <span className="text-[10px] font-medium">账号池</span>
              </button>
              <button
                onClick={() => setActiveTab("overview")}
                className={cn(
                  "flex h-full w-full flex-col items-center justify-center gap-1 transition-colors",
                  activeTab === "overview"
                    ? "text-emerald-500"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <CircleGauge className="h-6 w-6" />
                <span className="text-[10px] font-medium">概览</span>
              </button>
              <button
                onClick={() => setActiveTab("settings")}
                className={cn(
                  "flex h-full w-full flex-col items-center justify-center gap-1 transition-colors",
                  activeTab === "settings"
                    ? "text-emerald-500"
                    : "text-muted-foreground hover:text-foreground",
                )}
              >
                <SettingsIcon className="h-6 w-6" />
                <span className="text-[10px] font-medium">设置</span>
              </button>
            </div>
          </div>
        )}

        <ActionLoadingOverlay
          runningAction={controller.runningAction}
          loginAccountId={controller.loginAccountId}
          accounts={controller.snapshot?.accounts ?? []}
        />
      </div>

      <AccountDialog
        form={controller.accountForm}
        saving={controller.savingAccount}
        onChange={controller.setAccountForm}
        onClose={() => controller.setAccountForm(null)}
        onSave={controller.handleSaveAccount}
      />
      <AccountPoolDialog
        mode={controller.accountPoolMode}
        code={controller.accountPoolCode}
        passphrase={controller.accountPoolPassphrase}
        busy={controller.accountPoolBusy}
        resultText={controller.accountPoolResult}
        onCodeChange={controller.setAccountPoolCode}
        onPassphraseChange={controller.setAccountPoolPassphrase}
        onClose={controller.closeAccountPoolDialog}
        onExport={controller.handleExportAccountPool}
        onImport={controller.handleImportAccountPool}
        onCopy={controller.copyAccountPoolCode}
      />
      <SettingsDialog
        open={controller.settingsOpen}
        onClose={controller.closeSettings}
        preferences={controller.snapshot?.preferences ?? null}
        errorText={controller.preferencesErrorText}
        saving={controller.preferencesSaving}
        onTogglePreference={controller.togglePreference}
      />
      <AboutDialog open={controller.aboutOpen} onClose={controller.closeAbout} />
      <UpdaterDialog />
      <DeleteConfirmDialog
        account={controller.accountToDelete}
        deleting={!!controller.deletingAccountId}
        onClose={controller.closeDeleteAccountDialog}
        onConfirm={controller.confirmDeleteAccount}
      />
    </WindowFrame>
  );
}
