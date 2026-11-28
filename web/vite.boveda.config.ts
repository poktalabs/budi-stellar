import { resolve } from "node:path";
import { defineConfig } from "vite";

// La bóveda se publica sola en https://stellar.budi.lat/boveda/ (base propia, sin tocar el resto del sitio).
export default defineConfig({
  root: "web",
  base: "/boveda/",
  build: {
    outDir: "dist-boveda",
    emptyOutDir: true,
    target: "es2022",
    rollupOptions: {
      input: {
        boveda: resolve(import.meta.dirname, "boveda/index.html"),
        mainnet: resolve(import.meta.dirname, "boveda/mainnet/index.html"),
      },
    },
  },
});
