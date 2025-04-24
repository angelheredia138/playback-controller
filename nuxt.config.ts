import { defineNuxtConfig } from "nuxt/config";
import * as fs from "graceful-fs";
fs.gracefulify(require("fs"));

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  typescript: {
    strict: true,
  },
  vite: {
    define: {
      __TAURI__: typeof process !== "undefined" && !!process.env.TAURI,
    },
    optimizeDeps: {
      exclude: ["@tauri-apps/api"], // Ensure Vite doesn't pre-bundle Tauri API
    },
    resolve: {
      alias: {
        "@tauri-apps/api": "@tauri-apps/api", // Ensure it resolves cleanly
      },
    },
  },
  css: ["~/assets/css/main.css"],
  compatibilityDate: "2024-11-01",
  devtools: { enabled: true },
  postcss: {
    plugins: {
      tailwindcss: {},
      autoprefixer: {},
    },
  },

  target: 'static',
  ssr:false,
  generate: {
    dir:'dist',
},
});
