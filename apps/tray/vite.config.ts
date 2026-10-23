import preact from "@preact/preset-vite";
import { defineConfig } from "vite";

// The app loads the built files from inside itself; the dev server is only for `tauri dev`.
export default defineConfig({
  plugins: [preact()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    // WebKit on macOS and Linux, WebView2 on Windows.
    target: ["es2022", "safari16"],
    outDir: "dist",
    emptyOutDir: true,
  },
});
