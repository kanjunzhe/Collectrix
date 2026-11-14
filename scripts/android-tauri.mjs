import { spawn } from "node:child_process";
import { delimiter, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const rootDir = resolve(scriptDir, "..");
const cliPath = join(
  rootDir,
  "node_modules",
  "@tauri-apps",
  "cli",
  "tauri.js",
);

const env = { ...process.env };

// Tauri's Android Gradle plugin invokes `npm.bat` on Windows for
// beforeBuildCommand/beforeDevCommand. A standard Node install only ships
// npm.cmd/npm.ps1, so expose a small shim while Gradle is running.
if (process.platform === "win32") {
  const shimDir = join(scriptDir, "bin");
  const pathKey =
    Object.keys(env).find((key) => key.toLowerCase() === "path") ?? "PATH";
  env[pathKey] = `${shimDir}${delimiter}${env[pathKey] ?? ""}`;
}

const child = spawn(
  process.execPath,
  [cliPath, "android", ...process.argv.slice(2)],
  {
    cwd: rootDir,
    env,
    stdio: "inherit",
  },
);

child.on("error", (error) => {
  console.error(error);
  process.exit(1);
});
child.on("exit", (code, signal) => {
  if (signal) {
    console.error(`Android build terminated by ${signal}`);
    process.exit(1);
  }
  process.exit(code ?? 1);
});
