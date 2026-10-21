import { defineConfig } from "vite";

export default defineConfig({
  server: { port: 1420, strictPort: true },
  clearScreen: false,
  build: {
    rolldownOptions: {
      onwarn(warning, defaultHandler) {
        // Tauri embeds a client-only SPA; Mantine's React Server Components marker has no server boundary here.
        if (warning.code === "MODULE_LEVEL_DIRECTIVE" && warning.message.includes('"use client"') && warning.id?.replaceAll("\\", "/").includes("/@mantine/")) return;
        defaultHandler(warning);
      },
    },
  },
});
