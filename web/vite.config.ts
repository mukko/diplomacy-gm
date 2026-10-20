import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// Dev: the API server runs separately on 127.0.0.1:8080 (cargo run in server/).
// Build: `npm run build` writes dist/, which the API server serves at / (architecture.md § 7).
const api = "http://127.0.0.1:8080";

export default defineConfig({
  plugins: [react()],
  server: {
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
    proxy: { "/games": api, "/maps": api },
  },
});
