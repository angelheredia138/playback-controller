<template>
  <div class="flex items-center justify-center h-screen bg-gray-900 text-white">
    <div class="text-center">
      <h1 class="text-3xl font-bold mb-6">Spotify Playback Controller</h1>
      <button
        @click="redirectToSpotify"
        class="bg-green-500 hover:bg-green-400 text-white font-bold py-2 px-4 rounded"
      >
        Login with Spotify
      </button>
      <p v-if="errorMessage" class="mt-4 text-red-500">{{ errorMessage }}</p>
    </div>
  </div>
</template>

<script setup>
import { ref } from "vue";

const errorMessage = ref("");

const redirectToSpotify = async () => {
  try {
    if (typeof window !== "undefined" && window.__TAURI__) {
      // Use the correct import path for invoke
      const { invoke } = await import("@tauri-apps/api/core");

      const authUrl = await invoke("get_spotify_auth_url");
      window.location.href = authUrl;
    } else {
      throw new Error("Tauri runtime is not available.");
    }
  } catch (error) {
    console.error("Error redirecting to Spotify:", error);
    errorMessage.value = error.message;
  }
};
</script>
