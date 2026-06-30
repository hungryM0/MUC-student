import { existsSync } from "node:fs";
import { spawnSync } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, "..");

const command = process.argv[2];
if (!command || !["dev", "check", "build"].includes(command)) {
  console.error("用法：node scripts/tauri-android.mjs <dev|check|build>");
  process.exit(1);
}

const env = { ...process.env };
const androidSdk = findAndroidSdk();
const ndkHome = findAndroidNdk(androidSdk);
const javaHome = findJavaHome();
const ndkBin = path.join(
  ndkHome,
  "toolchains",
  "llvm",
  "prebuilt",
  "windows-x86_64",
  "bin",
);
const originalPath = env.PATH ?? env.Path ?? "";

env.ANDROID_HOME = androidSdk;
env.ANDROID_SDK_ROOT = androidSdk;
env.ANDROID_NDK_HOME = ndkHome;
env.NDK_HOME = ndkHome;
env.JAVA_HOME = javaHome;
env.PATH = [path.join(javaHome, "bin"), ndkBin, originalPath]
  .filter(Boolean)
  .join(path.delimiter);
env.Path = env.PATH;

for (const [target, triple] of [
  ["aarch64_linux_android", "aarch64-linux-android"],
  ["armv7_linux_androideabi", "armv7a-linux-androideabi"],
  ["i686_linux_android", "i686-linux-android"],
  ["x86_64_linux_android", "x86_64-linux-android"],
]) {
  env[`CC_${target}`] = path.join(ndkBin, `${triple}24-clang.cmd`);
  env[`CXX_${target}`] = path.join(ndkBin, `${triple}24-clang++.cmd`);
}

const result =
  command === "check"
    ? spawnSync("cargo", ["check", "--target", "aarch64-linux-android"], {
        cwd: rootDir,
        env,
        stdio: "inherit",
        shell: process.platform === "win32",
      })
    : spawnSync("pnpm", ["tauri", "android", command], {
        cwd: rootDir,
        env,
        stdio: "inherit",
        shell: process.platform === "win32",
      });

process.exit(result.status ?? 1);

function findAndroidSdk() {
  const candidates = [
    process.env.ANDROID_HOME,
    process.env.ANDROID_SDK_ROOT,
    path.join(os.homedir(), "AppData", "Local", "Android", "Sdk"),
  ].filter(Boolean);

  const sdk = candidates.find((item) =>
    existsSync(path.join(item, "platform-tools", "adb.exe")),
  );
  if (!sdk) {
    throw new Error(
      "找不到 Android SDK。请安装 Android SDK，或设置 ANDROID_HOME。",
    );
  }
  return sdk;
}

function findAndroidNdk(androidSdk) {
  const ndkRoot = path.join(androidSdk, "ndk");
  if (!existsSync(ndkRoot)) {
    throw new Error("找不到 Android NDK。请安装 NDK。");
  }

  const candidates = spawnSync(
    "powershell",
    [
      "-NoProfile",
      "-Command",
      `Get-ChildItem -LiteralPath '${ndkRoot.replaceAll("'", "''")}' -Directory | Sort-Object Name -Descending | Select-Object -First 1 -ExpandProperty FullName`,
    ],
    { encoding: "utf8" },
  )
    .stdout.trim()
    .split(/\r?\n/)
    .filter(Boolean);

  const ndk = candidates.find((item) =>
    existsSync(
      path.join(
        item,
        "toolchains",
        "llvm",
        "prebuilt",
        "windows-x86_64",
        "bin",
      ),
    ),
  );
  if (!ndk) {
    throw new Error("找不到可用 Android NDK toolchain。");
  }
  return ndk;
}

function findJavaHome() {
  const candidates = [
    process.env.JAVA_HOME,
    path.join("C:", "Program Files", "Android", "Android Studio", "jbr"),
    path.join("C:", "Program Files", "Java", "jdk-17.0.2"),
    path.join("C:", "Program Files", "Java", "jdk-21"),
    path.join("C:", "Program Files", "Java", "jdk-25"),
  ].filter(Boolean);

  const javaHome = candidates.find((item) =>
    existsSync(path.join(item, "bin", "java.exe")),
  );
  if (!javaHome) {
    throw new Error("找不到 Java。请安装 JDK 17+，或设置 JAVA_HOME。");
  }
  return javaHome;
}
