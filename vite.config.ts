import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { readFileSync } from "node:fs";
import path from "node:path";

const host = process.env.TAURI_DEV_HOST;
const screenshotMode = process.env.GSM_SCREENSHOT === "1";
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
      "@": path.resolve(__dirname, "src"),
      "@shared": path.resolve(__dirname, "shared"),
      ...(screenshotMode
        ? {
            "@tanstack/react-virtual": path.resolve(
              __dirname,
              "src/screenshot/mockReactVirtual.ts",
            ),
          }
        : {}),
    },
  },
  clearScreen: false,
  server: {
    port: screenshotMode ? 1433 : 1420,
    strictPort: true,
    host: host || false,
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
    rollupOptions: {
      input: {
        main: path.resolve(__dirname, "index.html"),
        screenshot: path.resolve(__dirname, "screenshot.html"),
      },
    },
  },
}));
