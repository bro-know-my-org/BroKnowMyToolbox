import vue from "@vitejs/plugin-vue";
import { readFileSync } from "node:fs";
import { defineConfig } from "vitest/config";

const packageJson = JSON.parse(
  readFileSync(new URL("./package.json", import.meta.url), "utf8"),
) as { version: string };

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  define: {
    __BKMT_VERSION__: JSON.stringify(packageJson.version),
  },
  build: {
    target:
      process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari13",
    chunkSizeWarningLimit: 700,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes("@bro-know-my/spark-analyzer"))
            return "spark-analyzer";
          if (id.includes("/naive-ui/")) return "naive-ui";
          if (id.includes("/marked/")) return "markdown";
        },
      },
    },
  },
  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
  },
  test: {
    environment: "jsdom",
  },
});
