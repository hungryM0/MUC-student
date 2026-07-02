import { useState, useRef } from "react";
import {
  CircleGauge,
  Settings as SettingsIcon,
  Users,
  Upload,
  Download,
} from "lucide-react";
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
import { Button } from "@/components/ui/button";
import { cn, isAndroid } from "@/lib/utils";
import appIconUrl from "../../src-tauri/icons/icon.svg?url";

export default function HomePage() {
  const [activeTab, setActiveTab] = useState<HomeTab>("accounts");
  const [slideDirection, setSlideDirection] = useState<"left" | "right">(
    "right",
  );
  const [navScaleX, setNavScaleX] = useState(1);
  const scaleTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const TAB_INDEX = {
    accounts: 0,
    overview: 1,
    settings: 2,
  };

  const handleTabChange = (tab: HomeTab) => {
    if (tab === activeTab) return;

    const currentIdx = TAB_INDEX[tab];
    const prevIdx = TAB_INDEX[activeTab];
    setSlideDirection(currentIdx > prevIdx ? "right" : "left");

    setNavScaleX(1.25);
    setActiveTab(tab);

    if (scaleTimeoutRef.current) {
      clearTimeout(scaleTimeoutRef.current);
    }

    scaleTimeoutRef.current = setTimeout(() => {
      setNavScaleX(0.95);
      scaleTimeoutRef.current = setTimeout(() => {
        setNavScaleX(1);
      }, 80);
    }, 180);
  };

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
            <header className="flex items-center justify-between border-b border-border/40 pb-3 md:pb-4">
              <div
                className={cn(
                  "flex gap-3",
                  isAndroid()
                    ? "flex-row items-start"
                    : "items-center md:gap-4",
                )}
              >
                <img
                  src={appIconUrl}
                  alt=""
                  className={cn(
                    "shrink-0 rounded-md",
                    isAndroid() ? "h-9 w-9 mt-0.5" : "h-7 w-7",
                  )}
                />
                <div className="flex flex-col min-w-0">
                  <h1 className="text-lg sm:text-xl font-bold tracking-tight text-foreground whitespace-nowrap shrink-0">
                    MUC 校园网拼车
                  </h1>
                  {isAndroid() && (
                    <span className="font-mono text-[10px] text-muted-foreground whitespace-nowrap shrink-0 mt-0.5 flex items-center gap-1.5">
                      <span
                        className={cn(
                          "h-1.5 w-1.5 rounded-full",
                          controller.snapshot?.network.ip &&
                            controller.snapshot.network.ip !== "unknown"
                            ? "bg-emerald-500 animate-pulse"
                            : "bg-amber-500",
                        )}
                      />
                      IP:{" "}
                      {controller.snapshot?.network.ip &&
                      controller.snapshot.network.ip !== "unknown"
                        ? controller.snapshot.network.ip
                        : "未识别"}
                    </span>
                  )}
                </div>
                {!isAndroid() && (
                  <div className="bg-border/60 hidden h-4 w-px sm:block" />
                )}
              </div>
              <div className="flex items-center gap-2 sm:gap-3">
                {!isAndroid() && (
                  <>
                    <span className="font-mono text-xs text-muted-foreground whitespace-nowrap shrink-0">
                      IP:{" "}
                      {controller.snapshot?.network.ip &&
                      controller.snapshot.network.ip !== "unknown"
                        ? controller.snapshot.network.ip
                        : "未识别"}
                    </span>
                    <div className="bg-border/40 h-3 w-px" />
                  </>
                )}
                <div className="flex items-center gap-0.5 sm:gap-1">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => controller.openAccountPoolDialog("import")}
                    disabled={controller.isBusy}
                    className="h-7 w-7 p-0 md:w-auto md:h-7 md:px-2 md:gap-1 text-xs border-border bg-background shadow-none text-foreground hover:bg-accent hover:text-accent-foreground"
                    title="导入号池"
                  >
                    <Upload className="h-3.5 w-3.5" />
                    <span className="hidden md:inline">导入</span>
                  </Button>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => controller.openAccountPoolDialog("export")}
                    disabled={
                      controller.isBusy || !controller.snapshot?.accounts.length
                    }
                    className="h-7 w-7 p-0 md:w-auto md:h-7 md:px-2 md:gap-1 text-xs border-border bg-background shadow-none text-foreground hover:bg-accent hover:text-accent-foreground"
                    title="导出号池"
                  >
                    <Download className="h-3.5 w-3.5" />
                    <span className="hidden md:inline">导出</span>
                  </Button>
                </div>
              </div>
            </header>
          </div>
        </div>

        <main
          className={cn(
            "min-w-0 flex-1 overflow-y-auto p-4 md:p-6",
            isAndroid() && "pb-[calc(env(safe-area-inset-bottom)+5.5rem)]",
          )}
        >
          <div className="mx-auto flex w-full max-w-5xl flex-col gap-4 md:gap-6 xl:max-w-7xl 2xl:max-w-[1440px]">
            <div className="flex flex-col gap-4 md:flex-row md:gap-6">
              {isAndroid() ? (
                <div
                  key={activeTab}
                  className={cn(
                    "w-full min-w-0 flex-1",
                    slideDirection === "right"
                      ? "animate-slide-from-right"
                      : "animate-slide-from-left",
                  )}
                >
                  {activeTab === "accounts" && (
                    <AccountPoolSection
                      snapshot={controller.snapshot}
                      loading={controller.loading}
                      isBusy={controller.isBusy}
                      selectingId={controller.selectingId}
                      loginAccountId={controller.loginAccountId}
                      deletingAccountId={controller.deletingAccountId}
                      runningAction={controller.runningAction}
                      onOpenAddAccount={controller.openAddAccountForm}
                      onRefresh={controller.refreshDashboard}
                      onEditAccount={controller.openEditAccountForm}
                      onDeleteAccount={controller.requestDeleteAccount}
                      onLoginAccount={controller.handleLoginAccount}
                    />
                  )}
                  {activeTab === "overview" && (
                    <OverviewSection
                      snapshot={controller.snapshot}
                      canLogoutLocalDevice={controller.canLogoutLocalDevice}
                      isBusy={controller.isBusy}
                      onLogoutLocalDevice={controller.logoutLocalDevice}
                    />
                  )}
                  {activeTab === "settings" && (
                    <MobileSettings
                      preferences={controller.snapshot?.preferences ?? null}
                      errorText={controller.preferencesErrorText}
                      saving={controller.preferencesSaving}
                      onTogglePreference={controller.togglePreference}
                    />
                  )}
                </div>
              ) : (
                <>
                  <div className="min-w-0 flex-1">
                    <AccountPoolSection
                      snapshot={controller.snapshot}
                      loading={controller.loading}
                      isBusy={controller.isBusy}
                      selectingId={controller.selectingId}
                      loginAccountId={controller.loginAccountId}
                      deletingAccountId={controller.deletingAccountId}
                      runningAction={controller.runningAction}
                      onOpenAddAccount={controller.openAddAccountForm}
                      onRefresh={controller.refreshDashboard}
                      onEditAccount={controller.openEditAccountForm}
                      onDeleteAccount={controller.requestDeleteAccount}
                      onLoginAccount={controller.handleLoginAccount}
                    />
                  </div>

                  <div className="flex w-full shrink-0 flex-col gap-6 md:w-80">
                    <OverviewSection
                      snapshot={controller.snapshot}
                      canLogoutLocalDevice={controller.canLogoutLocalDevice}
                      isBusy={controller.isBusy}
                      onLogoutLocalDevice={controller.logoutLocalDevice}
                    />
                  </div>
                </>
              )}
            </div>
          </div>
        </main>

        {isAndroid() && (
          <div className="absolute right-0 bottom-0 left-0 z-40 border-t border-border/40 bg-background/95 pb-[env(safe-area-inset-bottom)] shadow-[0_-4px_16px_rgba(0,0,0,0.05)] backdrop-blur-md">
            <div className="mx-auto flex h-16 max-w-md items-center px-6">
              <div className="relative flex w-full h-full items-center justify-around">
                {/* Fluent Design Slide Indicator (绿色小胶囊) */}
                <div
                  className="absolute top-[7px] h-8 w-16 rounded-full bg-emerald-500/8 dark:bg-emerald-500/15 pointer-events-none transition-[left,transform] duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]"
                  style={{
                    left:
                      activeTab === "accounts"
                        ? "16.67%"
                        : activeTab === "overview"
                          ? "50%"
                          : "83.33%",
                    transform: `translateX(-50%) scaleX(${navScaleX})`,
                  }}
                />

                <button
                  onClick={() => handleTabChange("accounts")}
                  className="flex h-full w-full flex-col items-center justify-center gap-1 outline-none"
                >
                  <div className="relative flex h-8 w-16 items-center justify-center rounded-full">
                    <Users
                      className={cn(
                        "h-5 w-5 transition-all duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]",
                        activeTab === "accounts"
                          ? "text-emerald-600 dark:text-emerald-400 scale-110"
                          : "text-muted-foreground scale-100",
                      )}
                    />
                  </div>
                  <span
                    className={cn(
                      "text-[10px] font-medium transition-colors duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]",
                      activeTab === "accounts"
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-muted-foreground",
                    )}
                  >
                    账号池
                  </span>
                </button>

                <button
                  onClick={() => handleTabChange("overview")}
                  className="flex h-full w-full flex-col items-center justify-center gap-1 outline-none"
                >
                  <div className="relative flex h-8 w-16 items-center justify-center rounded-full">
                    <CircleGauge
                      className={cn(
                        "h-5 w-5 transition-all duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]",
                        activeTab === "overview"
                          ? "text-emerald-600 dark:text-emerald-400 scale-110"
                          : "text-muted-foreground scale-100",
                      )}
                    />
                  </div>
                  <span
                    className={cn(
                      "text-[10px] font-medium transition-colors duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]",
                      activeTab === "overview"
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-muted-foreground",
                    )}
                  >
                    概览
                  </span>
                </button>

                <button
                  onClick={() => handleTabChange("settings")}
                  className="flex h-full w-full flex-col items-center justify-center gap-1 outline-none"
                >
                  <div className="relative flex h-8 w-16 items-center justify-center rounded-full">
                    <SettingsIcon
                      className={cn(
                        "h-5 w-5 transition-all duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]",
                        activeTab === "settings"
                          ? "text-emerald-600 dark:text-emerald-400 scale-110"
                          : "text-muted-foreground scale-100",
                      )}
                    />
                  </div>
                  <span
                    className={cn(
                      "text-[10px] font-medium transition-colors duration-300 ease-[cubic-bezier(0.1,0.9,0.2,1)]",
                      activeTab === "settings"
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-muted-foreground",
                    )}
                  >
                    设置
                  </span>
                </button>
              </div>
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
      <AboutDialog
        open={controller.aboutOpen}
        onClose={controller.closeAbout}
      />
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
