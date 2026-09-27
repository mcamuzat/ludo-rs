import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const NGROK_HOSTS = [".ngrok-free.dev", ".ngrok-free.app"];

export default defineConfig({
  // Relative paths: the build works from any folder (GitHub Pages project site).
  base: "./",
  plugins: [react()],
  worker: { format: "es" },
  // The site may be shared through an ngrok tunnel (any subdomain): `pnpm share`.
  server: { allowedHosts: NGROK_HOSTS },
  preview: { allowedHosts: NGROK_HOSTS },
});
