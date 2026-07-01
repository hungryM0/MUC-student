import { check } from "@tauri-apps/plugin-updater";
import type { Update } from "@tauri-apps/plugin-updater";
import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";
import { isAndroid } from "@/lib/utils";

const UPDATE_FEED_URL = "https://student.hungrym0.com/latest.json";

export interface UpdateProgress {
  event: "Started" | "Progress" | "Finished";
  data?: {
    contentLength?: number;
    chunkLength?: number;
    downloaded?: number;
  };
}

export type UpdateCheckResult =
  | { status: "available"; update: Update }
  | { status: "android-available"; update: AndroidUpdate }
  | { status: "up-to-date" }
  | { status: "error"; error: unknown };

export interface AndroidUpdate {
  version: string;
  url: string;
  notes?: string;
}

interface UpdateFeed {
  version?: string;
  notes?: string;
  android?: {
    version?: string;
    url?: string;
    notes?: string;
  };
}

export async function checkForUpdates(): Promise<UpdateCheckResult> {
  try {
    if (isAndroid()) {
      const update = await checkAndroidUpdate();
      return update
        ? { status: "android-available", update }
        : { status: "up-to-date" };
    }

    const update = await check();
    if (update) {
      return { status: "available", update };
    }

    return { status: "up-to-date" };
  } catch (error) {
    return { status: "error", error };
  }
}

async function checkAndroidUpdate(): Promise<AndroidUpdate | null> {
  const [currentVersion, feed] = await Promise.all([
    getVersion(),
    fetchUpdateFeed(),
  ]);
  const android = feed.android;
  const version = android?.version ?? feed.version;
  const url = android?.url;

  if (!version || !url || compareVersions(version, currentVersion) <= 0) {
    return null;
  }

  return {
    version,
    url,
    notes: android?.notes ?? feed.notes,
  };
}

async function fetchUpdateFeed(): Promise<UpdateFeed> {
  const response = await fetch(UPDATE_FEED_URL, { cache: "no-store" });
  if (!response.ok) {
    throw new Error(`检查更新失败：${response.status}`);
  }
  return (await response.json()) as UpdateFeed;
}

function compareVersions(left: string, right: string) {
  const leftParts = parseVersionParts(left);
  const rightParts = parseVersionParts(right);
  const maxLength = Math.max(leftParts.length, rightParts.length);

  for (let index = 0; index < maxLength; index += 1) {
    const diff = (leftParts[index] ?? 0) - (rightParts[index] ?? 0);
    if (diff !== 0) {
      return diff;
    }
  }

  return 0;
}

function parseVersionParts(value: string) {
  return value
    .trim()
    .replace(/^v/i, "")
    .split(/[.-]/)
    .map((part) => Number.parseInt(part, 10))
    .map((part) => (Number.isFinite(part) ? part : 0));
}

export async function downloadAndInstall(
  update?: Update | AndroidUpdate,
  onProgress?: (progress: UpdateProgress) => void,
) {
  if (isAndroid()) {
    if (!update || !("url" in update)) {
      return false;
    }
    await openUrl(update.url);
    return true;
  }

  const desktopUpdate =
    update && "downloadAndInstall" in update ? update : await check();

  if (!desktopUpdate) {
    return false;
  }

  let downloaded = 0;
  let contentLength = 0;

  await desktopUpdate.downloadAndInstall((event) => {
    switch (event.event) {
      case "Started":
        contentLength = event.data.contentLength!;
        onProgress?.({
          event: "Started",
          data: { ...event.data, downloaded: 0 },
        });
        break;
      case "Progress":
        downloaded += event.data.chunkLength;
        onProgress?.({
          event: "Progress",
          data: { ...event.data, contentLength, downloaded },
        });
        break;
      case "Finished":
        onProgress?.({
          event: "Finished",
          data: { contentLength, downloaded },
        });
        break;
    }
  });

  await relaunch();
  return true;
}
