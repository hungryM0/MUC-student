import type { AccountDto, AppSnapshotDto } from "@/lib/muc";

export type RunningAction = "login" | "refresh" | "logout";
export type HomeTab = "accounts" | "overview" | "settings";
export type AccountPoolDialogMode = "export" | "import";
export type Preferences = AppSnapshotDto["preferences"];

export type AccountFormState = {
  accountId: string;
  remarkName: string;
  username: string;
  password: string;
};

export type AccountFormChange = (form: AccountFormState | null) => void;
export type AccountAction = (account: AccountDto) => void;
