/// <reference types="vitest/config" />
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { readFileSync } from "node:fs";

// The version the About dialog shows: the app's, from tauri.conf.json (the build's).
const version = JSON.parse(readFileSync(new URL("./src-tauri/tauri.conf.json", import.meta.url), "utf8")).version as string;

// Tauri expects a fixed port in dev and no network exposure.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  define: { __APP_VERSION__: JSON.stringify(version) },
  // ../licenses: the GPL text the About dialog bundles (?raw import)
  server: { port: 5173, strictPort: true, host: "localhost", fs: { allow: [".", "../licenses"] } },
  build: {
    target: "es2022",
    sourcemap: false,
    // licence summary of bundled packages (the project's internal review notes 9.5)
    license: true,
  },
  resolve: process.env.VITEST ? { conditions: ["browser"] } : undefined,
  test: {
    environment: "happy-dom",
    include: ["tests/**/*.test.ts"],
  },
});
