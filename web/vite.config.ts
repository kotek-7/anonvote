import react from "@vitejs/plugin-react";
import { fileURLToPath } from "url";
import { defineConfig, loadEnv } from "vite";

export default defineConfig(({ mode }) => {
  const envDir = fileURLToPath(new URL("../", import.meta.url))
  const env = loadEnv(mode, envDir, '');
  return {
    plugins: [react()],
    server: {
      host: env.WEB_HOST ?? "127.0.0.1",
      port: Number(env.WEB_PORT ?? "8080"),
      strictPort: true,
      proxy: {
        "/api": env.API_URL ?? "http://127.0.0.1:8081",
      },
    },
  };
});
