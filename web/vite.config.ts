import { resolve } from "node:path";
import { defineConfig } from "vite";

// Se publica en https://stellar.budi.lat: /aula/ (el entorno de práctica) y /inscribete/.
export default defineConfig({
  root: "web",
  base: "/",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    target: "es2022",
    rollupOptions: {
      input: {
        aula: resolve(import.meta.dirname, "aula/index.html"),
        inscribete: resolve(import.meta.dirname, "inscribete/index.html"),
      },
    },
  },
});
