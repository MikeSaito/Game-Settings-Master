import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { readFileSync } from "node:fs";
import path from "node:path";

const rootDir = import.meta.dirname;
const host = process.env.TAURI_DEV_HOST;
const screenshotMode = process.env.GSM_SCREENSHOT === "1";
const e2eMode = process.env.GSM_E2E === "1";
const appVersion = JSON.parse(
  readFileSync(new URL("./package.json", import.meta.url), "utf8"),
).version as string;

export default defineConfig(async () => ({
  plugins: [react(), tailwindcss()],
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
  },
  resolve: {
    alias: {
      "@": path.resolve(rootDir, "src"),
      "@shared": path.resolve(rootDir, "shared"),
      ...(screenshotMode || e2eMode
        ? {
            "@tanstack/react-virtual": path.resolve(
              rootDir,
              "src/screenshot/mockReactVirtual.ts",
            ),
          }
        : {}),
      ...(e2eMode
        ? {
            "@tauri-apps/api/core": path.resolve(rootDir, "src/e2e/tauriCoreMock.ts"),
            "@tauri-apps/api/window": path.resolve(rootDir, "src/e2e/tauriWindowMock.ts"),
            "@tauri-apps/plugin-process": path.resolve(rootDir, "src/e2e/tauriProcessMock.ts"),
            "@tauri-apps/plugin-updater": path.resolve(rootDir, "src/e2e/tauriUpdaterMock.ts"),
            "@tauri-apps/plugin-dialog": path.resolve(rootDir, "src/e2e/tauriDialogMock.ts"),
          }
        : {}),
    },
  },
  clearScreen: false,
  server: {
    port: e2eMode ? 1434 : screenshotMode ? 1433 : 1420,
    strictPort: true,
    // Explicit IPv4 loopback — Vite's `host: false` can bind [::1] only on Windows,
    // which breaks Cursor browser / tools that resolve localhost to 127.0.0.1.
    host: host || "127.0.0.1",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    rolldownOptions: {
      checks: {
        pluginTimings: false,
      },
    },
    rollupOptions: {
      input: {
        main: path.resolve(rootDir, "index.html"),
        ...(screenshotMode || e2eMode
          ? {
              screenshot: path.resolve(rootDir, "screenshot.html"),
              e2e: path.resolve(rootDir, "e2e.html"),
            }
          : {}),
      },
    },
  },
}));
