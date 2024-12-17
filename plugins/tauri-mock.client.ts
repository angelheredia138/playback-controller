import { defineNuxtPlugin } from "#app";

export default defineNuxtPlugin(() => {
  if (typeof window !== "undefined" && !window.__TAURI__) {
    window.__TAURI__ = {
      invoke: async (cmd: string) => {
        console.warn(`Mocking Tauri command: ${cmd}`);
        return "https://mock.spotify.auth.url";
      },
    };
  }
});
