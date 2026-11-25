import { defineConfig } from "vite";

// Se publica en https://stellar.budi.lat/inscribete/
export default defineConfig({
  root: "web",
  base: "/inscribete/",
  build: { outDir: "dist", emptyOutDir: true, target: "es2022" },
});
