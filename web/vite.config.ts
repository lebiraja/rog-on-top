import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const ROGD = "http://127.0.0.1:7777";

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    proxy: {
      "/api": {
        target: ROGD,
        // rogd rejects foreign Origin headers, so present the dev server as the daemon itself.
        configure: (proxy) => proxy.on("proxyReq", (req) => req.setHeader("origin", ROGD)),
      },
    },
  },
});
